use crate::game_value::GameUInt;
use crate::SpriteType;
use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::{Debug, Formatter};
use core::ops::{Add, Not};
use crankstart::graphics::{Bitmap, Graphics, LCDColor};
use crankstart::sprite::{Sprite, SpriteManager, TextSprite};
use crankstart::system;
use crankstart::system::System;
use crankstart_sys::{LCDBitmapFlip, LCDSolidColor};
use euclid::Size2D;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameMode {
    LevelSelect,
    Game,
    Menu,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Core parameters that may be changed/upgraded and impact how other things behave
pub struct CoreParameters {
    /// How much each knead tick increases the fill bar
    pub(crate) knead_tick_size: f32,
    /// How much each pasta is worth
    pub(crate) pasta_price: GameUInt,
    /// How much autocranking occurs
    pub(crate) auto_crank_level: usize,
    /// How much autokneading occurs
    pub(crate) auto_knead_level: usize,
}

impl Default for CoreParameters {
    fn default() -> Self {
        Self {
            knead_tick_size: 0.02,
            pasta_price: GameUInt::from(20usize),
            auto_crank_level: 0,
            auto_knead_level: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Core state of the game, including things that change/increase over time
pub struct CoreState {
    // TODO: Quantities here will need to be able to grow larger than usize
    pub(crate) money: GameUInt,
    pub(crate) diamonds: GameUInt,
    pub(crate) dough_balls: GameUInt,
}

impl Default for CoreState {
    #[cfg(feature = "starting_money")]
    fn default() -> Self {
        Self {
            money: GameUInt::from(15000000usize),
            diamonds: GameUInt::from(42usize),
            dough_balls: GameUInt::from(5usize),
        }
    }
    #[cfg(not(feature = "starting_money"))]
    fn default() -> Self {
        Self {
            money: GameUInt::from(0usize),
            diamonds: GameUInt::from(0usize),
            dough_balls: GameUInt::from(0usize),
        }
    }
}

impl CoreState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_money(&mut self, amount: usize) {
        self.money += amount;
    }
    pub fn add_money_big(&mut self, amount: GameUInt) {
        self.money += amount;
    }
}

#[derive(Debug)]
pub struct IncrSprite {
    images: Vec<Bitmap>,
    sprite: Sprite,
    idx: Option<usize>,
    empty_bitmap: Bitmap,
}

impl IncrSprite {
    pub fn new(
        pos: (f32, f32),
        base_name: &str,
        num_images: usize,
        sprite_tag: SpriteType,
    ) -> Self {
        let graphics = Graphics::get();
        let images: Vec<Bitmap> = (0..num_images)
            .flat_map(|idx| graphics.load_bitmap(&format!("{}{}", base_name, idx)))
            .collect();
        let sprite_manager = SpriteManager::get_mut();
        let mut sprite = sprite_manager.new_sprite().unwrap();
        sprite
            .set_image(images[0].clone(), LCDBitmapFlip::kBitmapUnflipped)
            .unwrap();
        sprite.set_tag(sprite_tag as u8).unwrap();
        let (x, y) = pos;
        sprite.move_to(x, y).unwrap();
        sprite_manager.add_sprite(&sprite).unwrap();
        let empty_bitmap = {
            let size = Size2D::new(0, 0);
            graphics
                .new_bitmap(size, LCDColor::Solid(LCDSolidColor::kColorWhite))
                .unwrap()
        };
        Self {
            images,
            sprite,
            idx: Some(0),
            empty_bitmap,
        }
    }

    fn set_image(&mut self) {
        if let Some(idx) = self.idx {
            self.sprite
                .set_image(self.images[idx].clone(), LCDBitmapFlip::kBitmapUnflipped)
                .unwrap();
        } else {
            self.sprite
                .set_image(self.empty_bitmap.clone(), LCDBitmapFlip::kBitmapUnflipped)
                .unwrap();
        }
    }

    pub fn reset(&mut self) {
        self.idx = Some(0);
        self.set_image();
    }

    pub fn incr(&mut self, wrap: bool) {
        let prev = self.idx;
        self.idx = match (self.idx, wrap) {
            (Some(x), true) => Some((x + 1) % self.images.len()),
            (Some(x), false) => {
                let new_x = x + 1;
                if new_x < self.images.len() {
                    Some(new_x)
                } else {
                    None
                }
            }
            (None, _) => None,
        };
        system::System::log_to_console(&format!("IncrSprite::incr {:?} -> {:?}", prev, self.idx));
        self.set_image();
    }

    pub fn get_idx(&self) -> Option<usize> {
        self.idx
    }

    pub fn is_active(&self) -> bool {
        self.idx.is_some()
    }
}

pub struct TextSpriteWithValue<V> {
    pub sprite: TextSprite,
    value: V,
    value_to_string: Box<dyn (Fn(&V) -> String)>,
}

impl<V> Debug for TextSpriteWithValue<V>
where
    V: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TextSpriteWithValue")
            .field("sprite", &self.sprite)
            .field("value", &self.value)
            .field("value_to_string", &"<opaque")
            .finish()
    }
}

impl<V: PartialEq + Clone> TextSpriteWithValue<V> {
    pub fn new(sprite: TextSprite, value: V, value_to_string: Box<dyn (Fn(&V) -> String)>) -> Self {
        let mut t = Self {
            sprite,
            value,
            value_to_string,
        };
        t.update_sprite();
        t
    }

    fn update_sprite(&mut self) {
        self.sprite
            .update_text(&(self.value_to_string)(&self.value))
            .unwrap();
    }
    pub fn update_value(&mut self, value: &V) {
        if *value != self.value {
            self.value = value.clone();
            self.update_sprite();
        }
    }
}

pub struct CountStore {
    /// Total count seen
    count: usize,
    /// Change to count since last drained
    dirty_count: usize,
}

impl CountStore {
    pub fn new() -> Self {
        Self {
            count: 0,
            dirty_count: 0,
        }
    }

    /// Returns total count of store, does not alter dirty count
    pub fn peek_count(&self) -> usize {
        self.count
    }

    /// Returns diff to count since last time this was called
    pub fn drain(&mut self) -> usize {
        let dcount = self.dirty_count;
        self.dirty_count = 0;
        dcount
    }

    pub fn add(&mut self, count: usize) {
        self.count += count;
        self.dirty_count += count;
    }
}

#[derive(Debug, Copy, Clone)]
pub enum VisibilityState {
    Visible,
    Hidden,
}

impl Not for VisibilityState {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::Visible => Self::Hidden,
            Self::Hidden => Self::Visible,
        }
    }
}

#[derive(Debug)]
pub struct Timer {
    start_time: f32,
    duration: f32,
    finished: bool,
    just_finished: bool,
}

impl Timer {
    pub fn new(duration: f32) -> Self {
        Self {
            start_time: system::System::get().get_elapsed_time().unwrap(),
            duration,
            finished: false,
            just_finished: false,
        }
    }

    pub fn update(&mut self) {
        if self.finished {
            self.just_finished = false;
            return;
        }
        let now = system::System::get()
            .get_elapsed_time()
            .unwrap_or(self.start_time);
        if now - self.start_time > self.duration {
            self.finished = true;
            self.just_finished = true;
        }
    }

    pub fn just_finished(&mut self) -> bool {
        self.just_finished
    }
    pub fn finished(&mut self) -> bool {
        self.finished
    }

    pub fn reset(&mut self) {
        self.start_time = system::System::get().get_elapsed_time().unwrap();
        self.finished = false;
        self.just_finished = false;
    }
}

#[derive(Debug)]
pub struct AutoTicker {
    level: usize,
    last_tick: f32,
    /** Rate is "degrees per sec", and is multiplied by level */
    base_rate: f32,
}

impl AutoTicker {
    pub fn new(base_rate: f32) -> Self {
        Self {
            level: 0,
            last_tick: System::get().get_elapsed_time().unwrap_or(0.0),
            base_rate,
        }
    }

    fn get_rate(&self) -> f32 {
        // Rate is "degrees per sec"
        self.level as f32 * self.base_rate
    }
    pub fn poll(&mut self, level: usize) -> f32 {
        if self.level != level {
            self.level = level;
        }
        if level == 0 {
            return 0.0;
        }

        let now = System::get().get_elapsed_time().unwrap_or(self.last_tick);
        let rate = self.get_rate();
        let time_diff = now - self.last_tick;
        self.last_tick = now;
        time_diff * rate
    }
}
