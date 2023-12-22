use crate::core_elements::{CoreState, TextSpriteWithValue};
use crate::game_value::{GameUInt, GameValue};
use crate::SpriteType;
use alloc::boxed::Box;
use alloc::format;
use crankstart::graphics::{Graphics, LCDColor};
use crankstart::sprite::{Sprite, SpriteManager, TextSprite};
use crankstart::system::System;
use crankstart_sys::{LCDBitmapFlip, LCDSolidColor};

pub struct DoughStore {
    dough_sprite: Sprite,
    count_text: TextSpriteWithValue<GameUInt>,
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
        let mut count_sprite =
            TextSprite::new("", LCDColor::Solid(LCDSolidColor::kColorWhite)).unwrap();
        count_sprite.get_sprite_mut().move_to(x + 40.0, y).unwrap();
        let count_text = TextSpriteWithValue::new(
            count_sprite,
            GameUInt::default(),
            Box::new(|count| format!("x{}", GameUInt::to_string_hum(count))),
        );
        let mut s = Self {
            dough_sprite: sprite,
            count_text,
        };
        s
    }

    fn update(&mut self, state: &CoreState) {
        self.count_text.update_value(&state.dough_balls);
    }
}
