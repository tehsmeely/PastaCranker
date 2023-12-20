use crate::game_value::GameUInt;
use crate::SpriteType;
use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::ops::{Add, AddAssign, Not};
use crankstart::geometry::ScreenSize;
use crankstart::graphics::{Bitmap, Graphics, LCDColor};
use crankstart::sprite::{Sprite, SpriteManager, TextSprite};
use crankstart::system;
use crankstart_sys::{LCDBitmapFlip, LCDSolidColor};
use euclid::Size2D;

/// Core parameters that may be changed/upgraded and impact how other things behave
pub struct CoreParameters {
    pub(crate) knead_tick_size: f32,
    pub(crate) pasta_price: usize,
}

impl Default for CoreParameters {
    fn default() -> Self {
        Self {
            knead_tick_size: 0.02,
            pasta_price: 5,
        }
    }
}

/// Core state of the game, including things that change/increase over time
pub struct CoreState {
    // TODO: Quantities here will need to be able to grow larger than usize
    pub(crate) money: GameUInt,
    pub(crate) diamonds: GameUInt,
    pub(crate) dough_balls: usize,
}

impl Default for CoreState {
    fn default() -> Self {
        Self {
            money: GameUInt::from(15000000usize),
            diamonds: GameUInt::from(42usize),
            dough_balls: 0,
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
}

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

impl<V: PartialEq + Clone> TextSpriteWithValue<V> {
    pub fn new(sprite: TextSprite, value: V, value_to_string: Box<dyn (Fn(&V) -> String)>) -> Self {
        Self {
            sprite,
            value,
            value_to_string,
        }
    }

    pub fn update_value(&mut self, value: &V) {
        if *value != self.value {
            self.value = value.clone();
            self.sprite
                .update_text(&(self.value_to_string)(&self.value))
                .unwrap();
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
