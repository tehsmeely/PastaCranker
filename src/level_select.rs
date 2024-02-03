use crate::core_elements::{CoreParameters, CoreState, VisibilityState};
use crate::game_value::GameValue;
use crate::save;
use alloc::format;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::ops::SubAssign;
use crankstart::graphics::{Bitmap, Graphics};
use crankstart::log_to_console;
use crankstart::sprite::{Sprite, TextSprite};
use crankstart::system::System;
use crankstart_sys::{LCDBitmapFlip, PDButtons};

#[derive(Debug)]
pub struct LevelSelect {
    save_files: Vec<Option<(CoreState, CoreParameters)>>,
    menu_items: Vec<MenuItem>,
    selected_index: i32,
    pressed_index: Option<i32>,
}

impl LevelSelect {
    pub fn new() -> Self {
        let save_files = save::load_all_partial();

        let mut menu_items = Vec::new();
        let mut y = 40.0;
        for (idx, save) in save_files.iter().enumerate() {
            let item = MenuItem::new(y, idx, save);
            y += 80.0;
            menu_items.push(item);
        }
        menu_items[0].set_selected(true);
        Self {
            save_files,
            menu_items,
            selected_index: 0,
            pressed_index: None,
        }
    }

    fn change_selected_item(&mut self, diff: i32) {
        let len = self.menu_items.len() as i32;
        let new_index = (self.selected_index + diff).clamp(0, len - 1);
        self.menu_items[self.selected_index as usize].set_selected(false);
        self.selected_index = new_index;
        self.menu_items[self.selected_index as usize].set_selected(true);
    }

    /// Returns None until a level is selected, at which point it returns Some(level_number)
    pub fn update(&mut self) -> Option<usize> {
        let (_, pressed, released) = System::get().get_button_state().unwrap();
        if (pressed & PDButtons::kButtonUp).0 != 0 {
            self.change_selected_item(-1);
        } else if (pressed & PDButtons::kButtonDown).0 != 0 {
            self.change_selected_item(1);
        }

        if (pressed & PDButtons::kButtonA).0 != 0 {
            self.menu_items[self.selected_index as usize].set_pressed(true, true);
            self.pressed_index = Some(self.selected_index);
        }
        if (released & PDButtons::kButtonA).0 != 0 {
            if let Some(idx) = self.pressed_index {
                let still_selecting_pressed_item = idx == self.selected_index;
                self.menu_items[idx as usize].set_pressed(false, still_selecting_pressed_item);
                if still_selecting_pressed_item {
                    return Some(idx as usize);
                }
            }
        }
        None
    }
}

#[derive(Debug)]
pub struct MenuItem {
    sprite: Sprite,
    name_text: TextSprite,
    desc_text: TextSprite,
    selected_image: Bitmap,
    unselected_image: Bitmap,
    pressed_image: Bitmap,
}

impl MenuItem {
    const NAME_TEXT_OFFSET: f32 = -18.0;
    const COST_TEXT_OFFSET: f32 = 18.0;
    pub fn new(y: f32, idx: usize, data: &Option<(CoreState, CoreParameters)>) -> Self {
        let x = 200.0;
        let mut sprite = crate::helpers::load_sprite_at("res/menu_item_background0", x, y, None);
        sprite.set_z_index(10).unwrap();
        let name_text_str = match data {
            Some((_, _)) => format!("Level {}", idx + 1),
            None => "New Game".to_string(),
        };
        let mut name_text = TextSprite::new(
            name_text_str,
            crankstart::graphics::LCDColor::Solid(crankstart_sys::LCDSolidColor::kColorWhite),
        )
        .unwrap();
        name_text
            .get_sprite_mut()
            .move_to(x, y + Self::NAME_TEXT_OFFSET)
            .unwrap();
        name_text.get_sprite_mut().set_z_index(11).unwrap();
        // TODO: Populate desc_text_str
        let desc_text_str = "Desc";
        let mut desc_text = TextSprite::new(
            desc_text_str,
            crankstart::graphics::LCDColor::Solid(crankstart_sys::LCDSolidColor::kColorWhite),
        )
        .unwrap();
        desc_text.get_sprite_mut().move_to(x, y).unwrap();
        desc_text.get_sprite_mut().set_z_index(11).unwrap();

        let graphics = Graphics::get();
        let unselected_image = graphics.load_bitmap("res/menu_item_background0").unwrap();
        let selected_image = graphics.load_bitmap("res/menu_item_background1").unwrap();
        let pressed_image = graphics.load_bitmap("res/menu_item_background2").unwrap();
        let mut t = Self {
            sprite,
            name_text,
            desc_text,
            selected_image,
            unselected_image,
            pressed_image,
        };
        t
    }

    pub fn set_selected(&mut self, selected: bool) {
        if selected {
            self.sprite
                .set_image(self.selected_image.clone(), LCDBitmapFlip::kBitmapUnflipped)
                .unwrap();
        } else {
            self.sprite
                .set_image(
                    self.unselected_image.clone(),
                    LCDBitmapFlip::kBitmapUnflipped,
                )
                .unwrap();
        }
    }

    pub fn set_pressed(&mut self, pressed: bool, selected: bool) {
        if pressed {
            self.sprite
                .set_image(self.pressed_image.clone(), LCDBitmapFlip::kBitmapUnflipped)
                .unwrap();
        } else {
            self.set_selected(selected);
        }
    }
}
