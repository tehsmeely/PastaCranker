use crate::core_elements::{TextSpriteWithValue, VisibilityState};
use crate::game_value::{GameUInt, GameValue};
use crate::helpers::load_sprite_at;
use crate::{CoreState, Menu, SpriteType};
use alloc::boxed::Box;
use alloc::format;
use core::ops::Not;
use crankstart::graphics::{Bitmap, Graphics, LCDColor};
use crankstart::sprite::{Sprite, SpriteManager, TextSprite};
use crankstart::system;
use crankstart::system::System;
use crankstart_sys::{LCDBitmapFlip, LCDSolidColor, PDButtons};

#[derive(Debug)]
pub struct BottomBar {
    background: Sprite,
    money: TextSpriteWithValue<GameUInt>,
    diamonds: TextSpriteWithValue<GameUInt>,
    menu_indicator: MenuIndicator,
    money_icon: Sprite,
    diamond_icon: Sprite,
}

impl BottomBar {
    pub fn new() -> Self {
        let z = 20;
        let y = 216.0;
        let mut background = {
            crate::helpers::load_sprite_at(
                "res/bottom_bar",
                200.0,
                y,
                Some(SpriteType::BottomBar as u8),
            )
        };
        background.set_z_index(z);
        let mut money_icon = load_sprite_at("res/coin", 290.0, y, None);
        money_icon.set_z_index(z + 1).unwrap();
        let mut money = TextSprite::new("", LCDColor::Solid(LCDSolidColor::kColorWhite)).unwrap();
        money.get_sprite_mut().move_to(335.0, y).unwrap();
        money.get_sprite_mut().set_z_index(z + 1).unwrap();
        let money = TextSpriteWithValue::new(
            money,
            GameUInt::default(),
            Box::new(GameUInt::to_string_hum),
        );
        let mut diamond_icon = load_sprite_at("res/diamond", 105.0, y, None);
        diamond_icon.set_z_index(z + 1).unwrap();
        let mut diamonds =
            TextSprite::new("", LCDColor::Solid(LCDSolidColor::kColorWhite)).unwrap();
        diamonds.get_sprite_mut().move_to(135.0, y).unwrap();
        diamonds.get_sprite_mut().set_z_index(z + 1).unwrap();
        let diamonds = TextSpriteWithValue::new(
            diamonds,
            GameUInt::default(),
            Box::new(GameUInt::to_string_hum),
        );
        let menu_indicator = MenuIndicator::new(30.0, y);
        Self {
            background,
            money,
            diamonds,
            menu_indicator,
            money_icon,
            diamond_icon,
        }
    }

    pub fn update(&mut self, state: &CoreState, menu: &mut Menu) {
        self.money.update_value(&state.money);
        self.diamonds.update_value(&state.diamonds);

        let (_, pressed, _released) = System::get().get_button_state().unwrap();
        if (pressed & self.menu_indicator.get_toggle_button()).0 != 0 {
            self.menu_indicator.toggle();
            menu.set_state(self.menu_indicator.state);
        }
    }
}

#[derive(Debug)]
struct MenuIndicator {
    sprite: Sprite,
    state: VisibilityState,
    hidden_image: Bitmap,
    visible_image: Bitmap,
}

impl MenuIndicator {
    fn new(x: f32, y: f32) -> Self {
        let hidden_image = Graphics::get()
            .load_bitmap("res/menu_indicator_left")
            .unwrap();
        let visible_image = Graphics::get()
            .load_bitmap("res/menu_indicator_right")
            .unwrap();
        let sprite = {
            let sprite_manager = SpriteManager::get_mut();
            let mut sprite = sprite_manager.new_sprite().unwrap();
            sprite
                .set_image(hidden_image.clone(), LCDBitmapFlip::kBitmapUnflipped)
                .unwrap();
            sprite.move_to(x, y).unwrap();
            sprite.set_z_index(21).unwrap();
            sprite_manager.add_sprite(&sprite).unwrap();
            sprite
        };
        Self {
            sprite,
            state: VisibilityState::Hidden,
            hidden_image,
            visible_image,
        }
    }

    /// Returns the button used to toggle, as it changes based on state
    fn get_toggle_button(&self) -> PDButtons {
        match self.state {
            VisibilityState::Visible => PDButtons::kButtonRight,
            VisibilityState::Hidden => PDButtons::kButtonLeft,
        }
    }

    fn toggle(&mut self) {
        System::log_to_console("Toggle menu indicator");
        self.state = !self.state;
        self.set_image();
    }
    fn set_image(&mut self) {
        let image = match self.state {
            VisibilityState::Visible => &self.visible_image,
            VisibilityState::Hidden => &self.hidden_image,
        };
        self.sprite
            .set_image(image.clone(), LCDBitmapFlip::kBitmapUnflipped)
            .unwrap();
    }
}
