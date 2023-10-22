#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use alloc::{format};
use crankstart::graphics::Bitmap;
use crankstart::sprite::{Sprite, SpriteManager};
use crankstart_sys::LCDBitmapFlip;
use {
    alloc::boxed::Box,
    anyhow::Error,
    crankstart::{
        crankstart_game,
        graphics::{Graphics, LCDColor, LCDSolidColor},
        system::System,
        Game, Playdate,
    },
};

mod helpers {
    pub fn wrap(val: f32, min: f32, max: f32) -> f32 {
        // Note: if val is bigger than min-max overshot, it'll not be returned in that range
        // i.e. if you pass in   -200, 0, 100, you'll get back -100 which is not what you expect
        if val < min {
            max + (val - min)
        } else if val > max {
            min + (val - max)
        } else {
            val
        }
    }
    pub enum WrapResult {
        NoWrap,
        Underflow,
        Overflow,
    }
    pub fn wrap_with_info(val: f32, min: f32, max: f32) -> (f32, WrapResult) {
        // Note: if val is bigger than min-max overshot, it'll not be returned in that range
        // i.e. if you pass in   -200, 0, 100, you'll get back -100 which is not what you expect
        if val < min {
            (max + (val - min), WrapResult::Underflow)
        } else if val > max {
            (min + (val - max), WrapResult::Overflow)
        } else {
            (val, WrapResult::NoWrap)
        }
    }
}

#[repr(u8)]
enum SpriteType {
    MachineCrank,
    MachineBody,
}

impl From<u8> for SpriteType {
    fn from(val: u8) -> Self {
        match val {
            0 => Self::MachineCrank,
            1 => Self::MachineBody,
            _ => panic!("Unknown sprite type {}", val),
        }
    }
}

struct IncrSprite {
    images: Vec<Bitmap>,
    sprite: Sprite,
    idx: usize,
}
impl IncrSprite {
    fn new(pos: (f32, f32), base_name: &str, num_images: usize) -> Self {
        let graphics = Graphics::get();
        let images: Vec<Bitmap> = (0..num_images)
            .flat_map(|idx| graphics.load_bitmap(&format!("{}{}", base_name, idx)))
            .collect();
        let sprite_manager = SpriteManager::get_mut();
        let mut sprite = sprite_manager.new_sprite().unwrap();
        sprite
            .set_image(images[0].clone(), LCDBitmapFlip::kBitmapUnflipped)
            .unwrap();
        sprite.set_tag(SpriteType::MachineCrank as u8);
        let (x, y) = pos;
        sprite.move_to(x, y).unwrap();
        sprite_manager.add_sprite(&sprite).unwrap();
        Self {
            images,
            sprite,
            idx: 0,
        }
    }

    fn incr(&mut self) {
        self.idx = (self.idx + 1) % self.images.len();
        self.sprite
            .set_image(
                self.images[self.idx].clone(),
                LCDBitmapFlip::kBitmapUnflipped,
            )
            .unwrap();
    }
}

struct CrankTracker {
    crank_progress: f32,
    progress_per_tick: f32,
}

impl CrankTracker {
    fn new(progress_per_tick: f32) -> Self {
        Self {
            crank_progress: 0.0,
            progress_per_tick,
        }
    }

    fn update(&mut self, crank_diff: f32) -> bool {
        // Using only positive cranking, need to see if there's a more intuitive way of supporting
        // continual cranking in either direction, or at least indicating to player
        if crank_diff < 0.0 {
            return false;
        }

        self.crank_progress += crank_diff;

        if self.crank_progress > self.progress_per_tick {
            self.crank_progress -= self.progress_per_tick;
            true
        } else {
            false
        }
    }
}

struct MachineCrank {
    images: Vec<Bitmap>,
    sprite: Sprite,
    pos: f32,
    crank_tracker: CrankTracker,
}

impl MachineCrank {
    fn new(pos: (f32, f32)) -> Self {
        System::log_to_console("Machine crank new");
        let graphics = Graphics::get();
        let images: Vec<Bitmap> = (0..14)
            .flat_map(|idx| graphics.load_bitmap(&format!("res/crank/crank{}", idx)))
            .collect();
        let sprite_manager = SpriteManager::get_mut();
        let mut sprite = sprite_manager.new_sprite().unwrap();
        sprite
            .set_image(images[0].clone(), LCDBitmapFlip::kBitmapUnflipped)
            .unwrap();
        sprite.set_tag(SpriteType::MachineCrank as u8);
        let (x, y) = pos;
        sprite.move_to(x, y).unwrap();
        sprite_manager.add_sprite(&sprite).unwrap();
        Self {
            images,
            sprite,
            pos: 0.0,
            crank_tracker: CrankTracker::new(360.0),
        }
    }

    fn get_idx(&self) -> usize {
        // 360.0 / 14.0 = 25.71428571428571
        let macro_idx = (self.pos / (360.0 / 28.0)) as usize;
        if macro_idx >= 14 {
            27 - macro_idx
        } else {
            macro_idx
        }
    }

    fn update(&mut self) -> bool {
        let system = System::get();
        let crank_change = system.get_crank_change().unwrap_or(0.0);
        self.pos = helpers::wrap(self.pos + crank_change, 0.0, 360.0);
        let idx = self.get_idx();
        self.sprite
            .set_image(self.images[idx].clone(), LCDBitmapFlip::kBitmapUnflipped)
            .unwrap();
        self.crank_tracker.update(crank_change)
    }
}

struct PastaMachineState {
    crank: MachineCrank,
    body_sprite: Sprite,
    top_dough: IncrSprite,
    bottom_dough: IncrSprite,
    last: f32,
}

impl PastaMachineState {
    pub fn new() -> Self {
        let y = 71.0;
        let x = 284.0;
        let body_sprite = {
            let sprite_manager = SpriteManager::get_mut();
            let mut sprite = sprite_manager.new_sprite().unwrap();
            let image = Graphics::get().load_bitmap("res/machine_body").unwrap();
            sprite
                .set_image(image, LCDBitmapFlip::kBitmapUnflipped)
                .unwrap();
            sprite.move_to(x, y).unwrap();
            sprite.set_tag(SpriteType::MachineBody as u8);
            sprite_manager.add_sprite(&sprite).unwrap();
            sprite
        };
        let crank_x = x + 38.0 + 11.0;
        let crank = MachineCrank::new((crank_x, y - 15.0));
        let top_dough = IncrSprite::new((x - 1.0, y - 33.0), "res/roller_dough/roller_dough", 4);
        let bottom_dough = IncrSprite::new(
            (x - 1.0, y + 21.0),
            "res/roller_dough/roller_dough_bottom",
            4,
        );

        Self {
            crank,
            body_sprite,
            top_dough,
            bottom_dough,
            last: System::get().get_elapsed_time().unwrap(),
        }
    }
    fn update_crank(&mut self) {
        let crank_ticked = self.crank.update();
        if crank_ticked {
            self.top_dough.incr();
            self.bottom_dough.incr();
        }
    }
    fn update(&mut self) {}
}
struct State {
    pasta_machine: PastaMachineState,
}

impl State {
    pub fn new(_playdate: &Playdate) -> Result<Box<Self>, Error> {
        crankstart::display::Display::get().set_refresh_rate(20.0)?;
        Ok(Box::new(Self {
            pasta_machine: PastaMachineState::new(),
        }))
    }
}

impl Game for State {
    fn update(&mut self, _playdate: &mut Playdate) -> Result<(), Error> {
        let graphics = Graphics::get();
        graphics.clear(LCDColor::Solid(LCDSolidColor::kColorWhite))?;

        System::get().draw_fps(0, 0)?;

        Ok(())
    }

    fn update_sprite(&mut self, sprite: &mut Sprite, _playdate: &mut Playdate) -> Result<(), Error> {
        let sprite_type: SpriteType = sprite.get_tag()?.into();
        match sprite_type {
            SpriteType::MachineCrank => self.pasta_machine.update_crank(),
            SpriteType::MachineBody => self.pasta_machine.update(),
        }
        Ok(())
    }
}

crankstart_game!(State);
