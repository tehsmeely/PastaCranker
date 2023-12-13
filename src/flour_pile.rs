use crate::fill_bar::FillBar;
use crate::helpers::load_sprite_at;
use crate::{CoreParameters, SpriteType};
use anyhow::Error;
use crankstart::graphics::{Bitmap, Graphics};
use crankstart::sprite::Sprite;
use crankstart::system::System;
use crankstart_sys::{LCDBitmapFlip, PDButtons};

pub struct FlourPile {
    sprite: Sprite,
    button_indicator: AButtonIndicator,
    fill_bar: FillBar,
}

impl FlourPile {
    pub fn new(pos: (f32, f32)) -> Self {
        let sprite = load_sprite_at(
            "res/flour_pile",
            pos.0,
            pos.1,
            Some(SpriteType::FlourPile as u8),
        );
        let button_indicator = AButtonIndicator::new((pos.0 + 30.0, pos.1 - 45.0));
        let fill_bar = FillBar::new((pos.0 + 55.0, pos.1 - 45.0));
        Self {
            sprite,
            button_indicator,
            fill_bar,
        }
    }

    pub fn fill_bar_update(&mut self) {
        self.fill_bar.update();
    }

    fn tick(&mut self, tick_size: f32) {
        System::log_to_console("FlourPile tick");
        self.fill_bar.incr_fill_pct(tick_size);
        self.fill_bar.update();
    }

    pub fn update(&mut self, parameters: &CoreParameters) {
        // TODO: Disable input if menu is open ...
        let (_, pressed, released) = System::get().get_button_state().unwrap();
        if (pressed & PDButtons::kButtonA).0 != 0 {
            self.button_indicator.set_pressed();
            self.tick(parameters.knead_tick_size);
        } else if (released & PDButtons::kButtonA).0 != 0 {
            self.button_indicator.set_unpressed();
            self.tick(parameters.knead_tick_size);
        }
    }
    pub fn draw_fill_bar(&self) -> Result<(), Error> {
        self.fill_bar.draw()
    }

    pub fn is_full(&self) -> bool {
        self.fill_bar.get_fill_pct() > 0.99
    }

    pub fn reset(&mut self) {
        self.fill_bar.set_fill_pct(0.0);
    }
}

struct AButtonIndicator {
    sprite: Sprite,
    normal_image: Bitmap,
    pressed_image: Bitmap,
}

impl AButtonIndicator {
    fn new(pos: (f32, f32)) -> Self {
        let sprite = load_sprite_at(
            "res/a_button",
            pos.0,
            pos.1,
            Some(SpriteType::AButtonIndicator as u8),
        );
        let graphics = Graphics::get();
        let normal_image = graphics.load_bitmap("res/a_button").unwrap();
        let pressed_image = graphics.load_bitmap("res/a_button_pressed").unwrap();
        Self {
            sprite,
            normal_image,
            pressed_image,
        }
    }

    fn set_pressed(&mut self) {
        self.sprite
            .set_image(self.pressed_image.clone(), LCDBitmapFlip::kBitmapUnflipped)
            .unwrap();
    }
    fn set_unpressed(&mut self) {
        self.sprite
            .set_image(self.normal_image.clone(), LCDBitmapFlip::kBitmapUnflipped)
            .unwrap();
    }
}
