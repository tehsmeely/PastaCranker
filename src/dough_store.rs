use crate::SpriteType;
use alloc::format;
use crankstart::graphics::{Graphics, LCDColor};
use crankstart::sprite::{Sprite, SpriteManager, TextSprite};
use crankstart::system::System;
use crankstart_sys::{LCDBitmapFlip, LCDSolidColor};

pub struct DoughStore {
    dough_sprite: Sprite,
    count_text: TextSprite,
    dough_count: u8,
}

impl DoughStore {
    pub fn new(pos: (f32, f32)) -> Self {
        System::log_to_console("DoughStore new");
        let graphics = Graphics::get();
        let image = graphics.load_bitmap("res/doughball").unwrap();
        let sprite_manager = SpriteManager::get_mut();
        let mut sprite = sprite_manager.new_sprite().unwrap();
        sprite
            .set_image(image, LCDBitmapFlip::kBitmapUnflipped)
            .unwrap();
        sprite.set_tag(SpriteType::DoughStoreDough as u8).unwrap();
        let (x, y) = pos;
        sprite.move_to(x, y).unwrap();
        sprite_manager.add_sprite(&sprite).unwrap();
        let mut count_text =
            TextSprite::new("", LCDColor::Solid(LCDSolidColor::kColorWhite)).unwrap();
        count_text.get_sprite_mut().move_to(x + 40.0, y).unwrap();
        let mut s = Self {
            dough_sprite: sprite,
            count_text,
            dough_count: 0,
        };
        s.update_count_text();
        s
    }

    pub fn set_dough_count(&mut self, count: u8) {
        self.dough_count = count;
        self.update_count_text()
    }

    fn update_count_text(&mut self) {
        self.count_text
            .update_text(&format!("x{}", self.dough_count))
            .unwrap();
    }

    /// Take one dough ball from the store, returns true if there was one to take.
    pub fn take_one(&mut self) -> bool {
        if self.dough_count > 0 {
            self.dough_count -= 1;
            self.update_count_text();
            true
        } else {
            false
        }
    }

    pub fn add_one(&mut self) {
        self.dough_count += 1;
        self.update_count_text();
    }
}
