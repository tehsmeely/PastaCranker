use crate::core_elements::{CountStore, IncrSprite};
use crate::dough_store::DoughStore;
use crate::flour_pile::FlourPile;
use crate::{dough_store, helpers, SpriteType};
use alloc::format;
use alloc::vec::Vec;
use crankstart::graphics::{Bitmap, Graphics};
use crankstart::sprite::{Sprite, SpriteManager};
use crankstart::system::System;
use crankstart_sys::LCDBitmapFlip;

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
        sprite.set_tag(SpriteType::MachineCrank as u8).unwrap();
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

    pub fn update(&mut self) -> bool {
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

pub struct PastaMachineState {
    crank: MachineCrank,
    body_sprite: Sprite,
    top_dough: IncrSprite,
    bottom_dough: IncrSprite,
    dough_store: DoughStore,
    completed_pasta: CountStore,
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
            sprite.set_tag(SpriteType::MachineBody as u8).unwrap();
            sprite_manager.add_sprite(&sprite).unwrap();
            sprite
        };
        let crank_x = x + 38.0 + 11.0;
        let crank = MachineCrank::new((crank_x, y - 15.0));
        let top_dough = IncrSprite::new(
            (x - 1.0, y - 33.0),
            "res/roller_dough/roller_dough",
            4,
            SpriteType::MachineDough,
        );
        let bottom_dough = IncrSprite::new(
            (x - 1.0, y + 21.0),
            "res/roller_dough/roller_dough_bottom",
            4,
            SpriteType::MachineDough,
        );
        let mut dough_store = DoughStore::new((280.0, 160.0));
        dough_store.set_dough_count(3);

        let completed_pasta = CountStore::new();

        Self {
            crank,
            body_sprite,
            top_dough,
            bottom_dough,
            dough_store,
            completed_pasta,
        }
    }
    pub fn update_crank(&mut self) {
        let crank_ticked = self.crank.update();
        if crank_ticked {
            let top_dough_pre = self.top_dough.get_idx();
            // TODO: Don't like this both should always be in sync so should treat them this way
            self.top_dough.incr(false);
            self.bottom_dough.incr(false);

            // TODO: Emit something when dough is completed. i.e. when top_dough transitions from
            // 3 to None
            if top_dough_pre == Some(3) && self.top_dough.get_idx().is_none() {
                self.completed_pasta.add(1);
            }

            if !self.top_dough.is_active() {
                // try and replenish
                if self.dough_store.take_one() {
                    self.top_dough.reset();
                    self.bottom_dough.reset();
                }
            }
        }
    }
    pub fn update(&mut self, pile: &mut FlourPile) {
        if pile.is_full() {
            self.dough_store.add_one();
            pile.reset();
        }
    }
}
