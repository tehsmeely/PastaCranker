use crate::audio_events::{AudioEvent, AudioEventChannel};
use crate::core_elements::AutoTicker;
use crate::fill_bar::FillBar;
use crate::helpers::load_sprite_at;
use crate::{CoreParameters, CoreState, GameUInt, SpriteType};
use alloc::vec::Vec;
use anyhow::Error;
use crankstart::graphics::{Bitmap, Graphics};
use crankstart::log_to_console;
use crankstart::sprite::{Sprite, SpriteManager};
use crankstart::system::System;
use crankstart_sys::{LCDBitmapFlip, PDButtons};

#[derive(Debug)]
pub struct FlourPile {
    sprite: Sprite,
    button_indicator: AButtonIndicator,
    fill_bar: FillBar,
    auto_knead: AutoTicker,
    kneading_grans: KneadingGrans,
}

#[derive(Debug)]
struct KneadingGrans {
    sprites: Vec<Sprite>,
    image: Bitmap,
    level: usize,
    midpoint: (f32, f32),
}

impl KneadingGrans {
    fn new(x: f32, y: f32) -> Self {
        let sprites = Vec::new();
        let image = Graphics::get().load_bitmap("res/gran").unwrap();
        Self {
            sprites,
            image,
            level: 0,
            midpoint: (x, y),
        }
    }

    fn set_level(&mut self, level: usize) {
        if level == self.level {
            return;
        }
        self.sprites.clear();
        let sprite_manager = SpriteManager::get_mut();
        for i in 0..level {
            let centred_i = i as i32 - (level as i32 / 2);
            let mut sprite = sprite_manager.new_sprite().unwrap();
            sprite
                .set_image(self.image.clone(), LCDBitmapFlip::kBitmapUnflipped)
                .unwrap();
            // TODO: Centre on midpoint properly
            sprite
                .move_to(self.midpoint.0 + (centred_i as f32 * 20.0), self.midpoint.1)
                .unwrap();
            sprite_manager.add_sprite(&sprite).unwrap();
            self.sprites.push(sprite);
        }
        self.level = level;
    }
}

impl FlourPile {
    pub fn new(pos: (f32, f32)) -> Self {
        let sprite = load_sprite_at(
            "res/flour_pile",
            pos.0,
            pos.1,
            Some(SpriteType::FlourPile as u8),
        );
        {
            log_to_console!("TEST!!!");
            let level = 19usize;
            for i in 0..level {
                let centred_i = i as i32 - (level as i32 / 2);
                log_to_console!("i: {}, centred_i: {}", i, centred_i);
            }
        }
        let button_indicator = AButtonIndicator::new((pos.0 + 30.0, pos.1 - 45.0));
        let fill_bar = FillBar::new((pos.0 + 55.0, pos.1 - 45.0));
        let auto_knead = AutoTicker::new(0.02);
        let kneading_grans = KneadingGrans::new(pos.0, pos.1 + 45.0);
        Self {
            sprite,
            button_indicator,
            fill_bar,
            auto_knead,
            kneading_grans,
        }
    }

    pub fn fill_bar_update(&mut self) {
        self.fill_bar.update();
    }

    fn tick(&mut self, tick_size: f32) {
        self.fill_bar.incr_fill_pct(tick_size);
        self.fill_bar.update();
    }

    pub fn update(
        &mut self,
        state: &mut CoreState,
        parameters: &CoreParameters,
        events: &mut AudioEventChannel,
    ) {
        self.kneading_grans.set_level(parameters.auto_knead_level);
        let auto_knead = self.auto_knead.poll(parameters.auto_knead_level);
        // TODO: Disable input if menu is open ...
        let (_, pressed, released) = System::get().get_button_state().unwrap();
        if (pressed & PDButtons::kButtonA).0 != 0 {
            self.button_indicator.set_pressed();
            self.tick(parameters.knead_tick_size);
        } else if (released & PDButtons::kButtonA).0 != 0 {
            self.button_indicator.set_unpressed();
            self.tick(parameters.knead_tick_size);
        } else {
            self.tick(auto_knead);
        }

        if self.is_full() {
            events.push(AudioEvent::DoughCreated);
            state.dough_balls += GameUInt::one();
            self.reset();
        }
    }
    pub fn draw_fill_bar(&self) -> Result<(), Error> {
        self.fill_bar.draw()
    }

    fn is_full(&self) -> bool {
        self.fill_bar.get_fill_pct() > 0.99
    }

    fn reset(&mut self) {
        self.fill_bar.set_fill_pct(0.0);
    }
}

#[derive(Debug)]
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
