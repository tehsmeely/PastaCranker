use crate::core_elements::{CoreParameters, CoreState, VisibilityState};
use crate::game_value::{GameUInt, GameValue};
use crate::{SpriteType, State};
use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::ops::{RemAssign, SubAssign};
use crankstart::graphics::{Bitmap, Graphics};
use crankstart::sprite::{Sprite, TextSprite};
use crankstart::system::System;
use crankstart_sys::{LCDBitmapFlip, PDButtons};
use num_bigint::BigUint;

pub struct Menu {
    state: VisibilityState,
    background: Sprite,
    menu_items: Vec<MenuItem>,
    selected_item_index: usize,
    pressed_item_index: usize,
}

impl Menu {
    const ITEM_SPACING: f32 = 50.0;
    const ITEM_MAX_DISPLAY_Y: f32 = 190.0;
    pub fn new() -> Self {
        let background = crate::helpers::load_sprite_at(
            "res/menu_background",
            -95.5,
            95.50,
            Some(SpriteType::Menu as u8),
        );
        let mut menu_items = Vec::new();
        let num_items = 30;
        let max_y_offset = 25.0 + (num_items as f32 * Self::ITEM_SPACING);
        let max_scroll_amount = if max_y_offset > Self::ITEM_MAX_DISPLAY_Y {
            max_y_offset - Self::ITEM_MAX_DISPLAY_Y
        } else {
            0.0
        };
        for i in 0..num_items {
            let x = 25.0 + (i as f32 * Self::ITEM_SPACING);
            let offset = BoundedPosition::new(x, x - max_scroll_amount, x);
            let data = MenuItemData::new_test(i);
            let item = MenuItem::new(data, offset);
            menu_items.push(item);
        }
        let mut s = Self {
            state: VisibilityState::Hidden,
            background,
            menu_items,
            selected_item_index: 0,
            pressed_item_index: 0,
        };
        s.change_selected_item(0);
        s
    }

    fn scroll(&mut self, change: f32) {
        System::log_to_console("Scrolling!");
        for item in self.menu_items.iter_mut() {
            item.scroll(change);
        }
    }

    fn change_selected_item(&mut self, change: i32) {
        System::log_to_console(format!("change_selected_item({})", change).as_str());
        let new_index = (self.selected_item_index as i32 + change)
            .clamp(0, self.menu_items.len() as i32 - 1) as usize;
        self.menu_items[self.selected_item_index].set_selected(false);
        self.selected_item_index = new_index;
        self.menu_items[self.selected_item_index].set_selected(true);

        let current_items_y = self.menu_items[self.selected_item_index].y_offset.get();
        if current_items_y > Self::ITEM_MAX_DISPLAY_Y {
            self.scroll(-Self::ITEM_SPACING);
        } else if current_items_y < 0.0 {
            self.scroll(Self::ITEM_SPACING);
        }
    }

    pub fn set_state(&mut self, state: VisibilityState) {
        self.state = state;
        match state {
            VisibilityState::Hidden => {
                self.background.move_to(-95.5, 95.50).unwrap();
                for item in &mut self.menu_items {
                    item.set_state(VisibilityState::Hidden);
                }
            }
            VisibilityState::Visible => {
                self.background.move_to(95.5, 95.50).unwrap();
                for item in &mut self.menu_items {
                    item.set_state(VisibilityState::Visible);
                }
            }
        }
    }
    fn update_internal(&mut self, state: &mut CoreState, parameters: &mut CoreParameters) {
        let (_, pressed, released) = System::get().get_button_state().unwrap();
        if (pressed & PDButtons::kButtonUp).0 != 0 {
            self.change_selected_item(-1);
        } else if (pressed & PDButtons::kButtonDown).0 != 0 {
            self.change_selected_item(1);
        }

        if (pressed & PDButtons::kButtonA).0 != 0 {
            System::log_to_console("Pressed A");
            if self.menu_items[self.selected_item_index].press_and_trigger(state, parameters) {
                System::log_to_console(&format!(
                    "Pressed A, cost: {}",
                    self.menu_items[self.selected_item_index].data.cost_str()
                ));
                self.menu_items[self.selected_item_index].set_pressed(true, true);
                self.pressed_item_index = self.selected_item_index;
            }
        } else if (released & PDButtons::kButtonA).0 != 0 {
            self.menu_items[self.pressed_item_index]
                .set_pressed(false, self.pressed_item_index == self.selected_item_index);
        }
    }
    pub fn update(&mut self, parameters: &mut CoreParameters, state: &mut CoreState) {
        // Only process key presses if enabled
        match self.state {
            VisibilityState::Hidden => {}
            VisibilityState::Visible => {
                self.update_internal(state, parameters);
            }
        }
    }
}
mod menu_item_data_prefabs {
    use crate::menu::MenuItemData;
    use crate::GameUInt;
    use alloc::boxed::Box;
    use num_bigint::BigUint;

    fn dough_tick() -> MenuItemData {
        MenuItemData {
            description: "Knead for Speed".into(),
            count: 0,
            cost_fn: Box::new(|count| {
                let cost: BigUint = BigUint::from(10usize).pow(count as u32);
                cost.into()
            }),
            on_buy_fn: Box::new(|count, _state, parameters| {
                parameters.knead_tick_size = 0.01 + (count as f32 * 0.01);
            }),
        }
    }
}

struct MenuItemData {
    description: String,
    count: usize,
    cost_fn: Box<dyn Fn(usize) -> GameUInt>,
    // TODO: Think about how this buy_fn is deterministic (in terms of saving an loading state) as
    // well as not overwriting other items (i.e. if both would change/set dough tick size)
    on_buy_fn: Box<dyn Fn(usize, &mut CoreState, &mut CoreParameters)>,
}

impl MenuItemData {
    fn cost(&self) -> GameUInt {
        (self.cost_fn)(self.count)
    }
    fn cost_str(&self) -> String {
        let cost: GameUInt = self.cost();
        cost.to_string_hum()
    }

    fn on_buy(&self, state: &mut CoreState, parameters: &mut CoreParameters) {
        (self.on_buy_fn)(self.count, state, parameters);
    }

    fn new_test(i: usize) -> Self {
        Self {
            description: format!("Menu item {}", i),
            count: 1,
            cost_fn: Box::new(move |count| {
                let cost: BigUint = BigUint::from(100 + i) * BigUint::from(count);
                cost.into()
            }),
            on_buy_fn: Box::new(move |count, state, parameters| {
                parameters.knead_tick_size += 0.1;
            }),
        }
    }
}

pub struct MenuItem {
    data: MenuItemData,
    sprite: Sprite,
    state: VisibilityState,
    y_offset: BoundedPosition,
    text: TextSprite,
    cost_text: TextSprite,
    selected_image: Bitmap,
    unselected_image: Bitmap,
    pressed_image: Bitmap,
}

impl MenuItem {
    const TEXT_OFFSET: f32 = -8.0;
    const COST_TEXT_OFFSET: f32 = 8.0;
    pub fn new(data: MenuItemData, y_offset: BoundedPosition) -> Self {
        let y = y_offset.get();
        let mut sprite =
            crate::helpers::load_sprite_at("res/menu_item_background0", -95.0, y, None);
        sprite.set_z_index(10).unwrap();
        let mut text = TextSprite::new(
            "",
            crankstart::graphics::LCDColor::Solid(crankstart_sys::LCDSolidColor::kColorWhite),
        )
        .unwrap();
        text.get_sprite_mut()
            .move_to(-95.0, y + Self::TEXT_OFFSET)
            .unwrap();
        text.get_sprite_mut().set_z_index(11).unwrap();
        let mut cost_text = TextSprite::new(
            "",
            crankstart::graphics::LCDColor::Solid(crankstart_sys::LCDSolidColor::kColorWhite),
        )
        .unwrap();
        cost_text
            .get_sprite_mut()
            .move_to(-95.0, y + Self::COST_TEXT_OFFSET)
            .unwrap();
        cost_text.get_sprite_mut().set_z_index(11).unwrap();
        let graphics = Graphics::get();
        let unselected_image = graphics.load_bitmap("res/menu_item_background0").unwrap();
        let selected_image = graphics.load_bitmap("res/menu_item_background1").unwrap();
        let pressed_image = graphics.load_bitmap("res/menu_item_background2").unwrap();
        let mut t = Self {
            data,
            sprite,
            state: VisibilityState::Hidden,
            y_offset,
            text,
            cost_text,
            selected_image,
            unselected_image,
            pressed_image,
        };
        t.update_text();
        t
    }

    fn update_text(&mut self) {
        let descr_str = format!("{}: {}", self.data.description, self.data.count);
        self.text.update_text(descr_str).unwrap();
        let cost_str = format!("Cost: {}", self.data.cost_str());
        self.cost_text.update_text(cost_str).unwrap();
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

    pub fn press_and_trigger(
        &mut self,
        state: &mut CoreState,
        parameters: &mut CoreParameters,
    ) -> bool {
        System::log_to_console(&format!(
            "Trying to buy: cost: {}, with cash {}",
            self.data.cost().to_string_hum(),
            state.money.to_string_hum()
        ));
        let cost = self.data.cost();
        if cost < state.money {
            self.data.count += 1;
            state.money.sub_assign(cost);
            self.data.on_buy(state, parameters);
            self.update_text();
            true
        } else {
            false
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

    fn scroll(&mut self, change: f32) {
        self.y_offset.change(change);
        let (x, _) = self.sprite.get_position().unwrap();
        let y = self.y_offset.get();
        self.sprite.move_to(x, y).unwrap();
        self.text
            .get_sprite_mut()
            .move_to(94.5, y + Self::TEXT_OFFSET)
            .unwrap();
        self.cost_text
            .get_sprite_mut()
            .move_to(94.5, y + Self::COST_TEXT_OFFSET)
            .unwrap();
    }
    pub fn set_state(&mut self, state: VisibilityState) {
        self.state = state;
        let y = self.y_offset.get();
        match state {
            VisibilityState::Hidden => {
                self.sprite.move_to(-95.5, y).unwrap();
                self.text
                    .get_sprite_mut()
                    .move_to(-95.5, y + Self::TEXT_OFFSET)
                    .unwrap();
                self.cost_text
                    .get_sprite_mut()
                    .move_to(-95.5, y + Self::COST_TEXT_OFFSET)
                    .unwrap();
            }
            VisibilityState::Visible => {
                self.sprite.move_to(94.5, y).unwrap();
                self.text
                    .get_sprite_mut()
                    .move_to(94.5, y + Self::TEXT_OFFSET)
                    .unwrap();
                self.cost_text
                    .get_sprite_mut()
                    .move_to(94.5, y + Self::COST_TEXT_OFFSET)
                    .unwrap();
            }
        }
    }
}

struct BoundedPosition {
    base: f32,
    current: f32,
    min_: f32,
    max_: f32,
}

impl BoundedPosition {
    fn new(base: f32, min_: f32, max_: f32) -> Self {
        Self {
            base,
            current: base,
            min_,
            max_,
        }
    }

    fn change(&mut self, change: f32) {
        self.current = (self.current + change).clamp(self.min_, self.max_);
    }

    fn reset(&mut self) {
        self.current = self.base;
    }

    fn get(&self) -> f32 {
        self.current
    }
}
