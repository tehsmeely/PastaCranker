use crate::helpers::{load_sprite_at, wrap_with_info};
use crankstart::sprite::Sprite;
use crankstart::system::System;

#[derive(Debug)]
pub struct InfoOverlay {
    sprite: Sprite,
    visible: bool,
}

impl InfoOverlay {
    pub fn new(visible_initially: bool) -> Self {
        let mut sprite = load_sprite_at("res/info_overlay", 200.0, 120.0, None);
        sprite.set_z_index(100).unwrap();
        match visible_initially {
            true => sprite.set_visible(true).unwrap(),
            false => sprite.set_visible(false).unwrap(),
        }
        Self {
            sprite,
            visible: visible_initially,
        }
    }

    pub fn show(&mut self) {
        self.sprite.set_visible(true).unwrap();
        self.visible = true;
    }

    pub fn update(&mut self) {
        if self.visible {
            // Hide self on any input:
            let (_, pressed, released) = System::get().get_button_state().unwrap();
            let any_input = (pressed | released).0 > 0;
            if any_input {
                self.sprite.set_visible(false).unwrap();
                self.visible = false;
            }
        }
    }
}
