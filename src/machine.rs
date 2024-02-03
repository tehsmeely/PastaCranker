use crate::audio_events::{AudioEvent, AudioEventChannel};
use crate::core_elements::{AutoTicker, IncrSprite};
use crate::dough_store::DoughStore;
use crate::game_value::GameUInt;
use crate::helpers::load_sprite_at;
use crate::{helpers, CoreParameters, CoreState, SpriteType};
use alloc::format;
use alloc::vec::Vec;
use crankstart::graphics::{Bitmap, Graphics};
use crankstart::sprite::{Sprite, SpriteManager};
use crankstart::system::System;
use crankstart_sys::LCDBitmapFlip;
use num_traits::Zero;

#[derive(Debug)]
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

#[derive(Debug)]
struct MachineCrank {
    images: Vec<Bitmap>,
    sprite: Sprite,
    pos: f32,
    crank_tracker: CrankTracker,
    auto_crank: AutoTicker,
    prev_crank_angle: f32,
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
            auto_crank: AutoTicker::new(40.0),
            prev_crank_angle: System::get().get_crank_angle().unwrap_or(0.0),
        }
    }

    fn get_idx(&self) -> usize {
        // 360.0 / 14.0 = 25.71428571428571
        let macro_idx = (self.pos / (360.0 / 28.0)) as usize;
        if macro_idx >= 14 {
            if macro_idx >= 27 {
                0
            } else {
                27 - macro_idx
            }
        } else {
            macro_idx
        }
    }

    pub fn update(&mut self, parameters: &CoreParameters) -> bool {
        let system = System::get();
        let angle_from_autocrank = self.auto_crank.poll(parameters.auto_crank_level);
        // There is extra paranoia here because in theory "get_crank_change" resets the value after
        // each call, but I've observed this being not true, as least in the simulator
        // As a result we monitor the crank angle and only update if it has changed
        let crank_change = system.get_crank_change().unwrap_or(0.0);
        let crank_angle = system.get_crank_angle().unwrap_or(0.0);
        let angle_changed = crank_angle != self.prev_crank_angle;
        self.prev_crank_angle = crank_angle;
        let crank_move = match (angle_changed, angle_from_autocrank) {
            (true, _) => Some(crank_change),
            (false, 0.0) => None,
            (false, _) => Some(angle_from_autocrank),
        };
        if let Some(crank_by_angle) = crank_move {
            self.pos = helpers::wrap(self.pos + crank_by_angle, 0.0, 360.0);
            let idx = self.get_idx();
            self.sprite
                .set_image(self.images[idx].clone(), LCDBitmapFlip::kBitmapUnflipped)
                .unwrap();
            self.crank_tracker.update(crank_by_angle)
        } else {
            false
        }
    }
}

#[derive(Debug)]
pub struct Gearbox {
    sprite: Sprite,
    visible: bool,
}

impl Gearbox {
    fn new(x: f32, y: f32) -> Self {
        let mut sprite = load_sprite_at("res/crank-gearbox", x, y, None);
        sprite.set_visible(false).unwrap();
        Self {
            sprite,
            visible: false,
        }
    }

    fn set_visible(&mut self, visible: bool) {
        if self.visible != visible {
            self.sprite.set_visible(visible).unwrap();
            self.visible = visible;
        }
    }
}

#[derive(Debug)]
pub struct PastaMachineState {
    crank: MachineCrank,
    gearbox: Gearbox,
    body_sprite: Sprite,
    top_dough: IncrSprite,
    bottom_dough: IncrSprite,
    dough_store: DoughStore,
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
        let gear_x = x - 38.0 - 5.0;
        let crank = MachineCrank::new((crank_x, y - 15.0));
        let gearbox = Gearbox::new(gear_x, y - 15.0);
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
        Self {
            crank,
            gearbox,
            body_sprite,
            top_dough,
            bottom_dough,
            dough_store,
        }
    }
    pub fn update_crank(
        &mut self,
        state: &mut CoreState,
        parameters: &CoreParameters,
        events: &mut AudioEventChannel,
    ) {
        let crank_ticked = self.crank.update(parameters);
        if crank_ticked {
            System::log_to_console("Crank ticked");
            let top_dough_pre = self.top_dough.get_idx();
            // TODO: Don't like this. Both should always be in sync so should treat them this way
            self.top_dough.incr(false);
            self.bottom_dough.incr(false);

            if top_dough_pre == Some(3) && self.top_dough.get_idx().is_none() {
                state.add_money_big(parameters.pasta_price.clone());
                events.push(AudioEvent::MoneyGained);
            }

            if !self.top_dough.is_active() {
                // try and replenish
                if !state.dough_balls.is_zero() {
                    state.dough_balls -= GameUInt::one();
                    self.top_dough.reset();
                    self.bottom_dough.reset();
                }
            }
        }
    }
    pub fn update(&mut self, state: &mut CoreState, parameters: &CoreParameters) {
        self.dough_store.update(state);
        self.gearbox.set_visible(parameters.auto_crank_level > 0)
    }
}
