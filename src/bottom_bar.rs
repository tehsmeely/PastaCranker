use crate::SpriteType;
use core::ops::Not;
use crankstart::graphics::{Bitmap, Graphics, LCDColor};
use crankstart::sprite::{Sprite, SpriteManager, TextSprite};
use crankstart_sys::{LCDBitmapFlip, LCDSolidColor};

pub struct BottomBar {
    background: Sprite,
    money: TextSprite,
    diamonds: TextSprite,
    menu_indicator: MenuIndicator,
}

impl BottomBar {
    pub fn new() -> Self {
        let y = 216.0;
        let background = {
            crate::helpers::load_sprite_at(
                "res/bottom_bar",
                200.0,
                y,
                Some(SpriteType::BottomBar as u8),
            )
        };
        let mut money = TextSprite::new("10", LCDColor::Solid(LCDSolidColor::kColorWhite)).unwrap();
        money.get_sprite_mut().move_to(320.0, y).unwrap();
        let mut diamonds =
            TextSprite::new("", LCDColor::Solid(LCDSolidColor::kColorWhite)).unwrap();
        diamonds.get_sprite_mut().move_to(100.0, y).unwrap();
        let menu_indicator = MenuIndicator::new(30.0, y);
        Self {
            background,
            money,
            diamonds,
            menu_indicator,
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum MenuIndicatorState {
    Visible,
    Hidden,
}

impl Not for MenuIndicatorState {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::Visible => Self::Hidden,
            Self::Hidden => Self::Visible,
        }
    }
}

struct MenuIndicator {
    sprite: Sprite,
    state: MenuIndicatorState,
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
            sprite_manager.add_sprite(&sprite).unwrap();
            sprite
        };
        Self {
            sprite,
            state: MenuIndicatorState::Hidden,
            hidden_image,
            visible_image,
        }
    }

    fn toggle(&mut self) {
        self.state = !self.state;
        self.set_image();
    }
    fn set_image(&mut self) {
        let image = match self.state {
            MenuIndicatorState::Visible => &self.visible_image,
            MenuIndicatorState::Hidden => &self.hidden_image,
        };
        self.sprite
            .set_image(image.clone(), LCDBitmapFlip::kBitmapUnflipped)
            .unwrap();
    }
}
