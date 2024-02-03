use crate::audio_events::{AudioEvent, AudioEventChannel};
use crate::core_elements::{CoreParameters, CoreState, VisibilityState};
use crate::game_value::{GameUInt, GameValue};
use crate::{GameState, SpriteType};
use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Debug;
use core::ops::{RemAssign, SubAssign};
use crankstart::graphics::{Bitmap, Graphics};
use crankstart::log_to_console;
use crankstart::sprite::{Sprite, TextSprite};
use crankstart::system::System;
use crankstart_sys::{LCDBitmapFlip, PDButtons};
use num_bigint::BigUint;

#[derive(Debug)]
pub struct Menu {
    state: VisibilityState,
    background: Sprite,
    menu_items: Vec<MenuItem>,
    selected_item_index: usize,
    pressed_item_index: usize,
}

impl Menu {
    const ITEM_Y_START: f32 = 35.0;
    const ITEM_SPACING: f32 = 72.0;
    const ITEM_MAX_DISPLAY_Y: f32 = 170.0;
    pub fn new() -> Self {
        let mut background = crate::helpers::load_sprite_at(
            "res/menu_background",
            -95.5,
            95.50,
            Some(SpriteType::Menu as u8),
        );
        background.set_z_index(9).unwrap();
        let mut menu_items = Vec::new();
        let num_items = 30;
        let max_y_offset = Self::ITEM_Y_START + (num_items as f32 * Self::ITEM_SPACING);
        let max_scroll_amount = if max_y_offset > Self::ITEM_MAX_DISPLAY_Y {
            max_y_offset - Self::ITEM_MAX_DISPLAY_Y
        } else {
            0.0
        };
        for (i, data) in menu_item_data_prefabs::all().into_iter().enumerate() {
            let y = Self::ITEM_Y_START + (i as f32 * Self::ITEM_SPACING);
            let offset = BoundedPosition::new(y, y - max_scroll_amount, y);
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

    pub fn init_counts(&mut self, counts: &[usize]) {
        if self.menu_items.len() != counts.len() {
            panic!("init_counts: counts.len() != menu_items.len(). Save file invalid?");
        }
        for (i, count) in counts.iter().enumerate() {
            self.menu_items[i].data.count = *count;
            self.menu_items[i].update_text();
        }
    }

    pub fn to_counts(&self) -> Vec<usize> {
        self.menu_items.iter().map(|item| item.data.count).collect()
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
    fn update_internal(
        &mut self,
        state: &mut CoreState,
        parameters: &mut CoreParameters,
        audio_events: &mut AudioEventChannel,
    ) {
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
                audio_events.push(AudioEvent::UpgradeBought);
            } else {
                audio_events.push(AudioEvent::UpgradeDenied);
            }
        } else if (released & PDButtons::kButtonA).0 != 0 {
            self.menu_items[self.pressed_item_index]
                .set_pressed(false, self.pressed_item_index == self.selected_item_index);
        }
    }
    pub fn update(
        &mut self,
        parameters: &mut CoreParameters,
        state: &mut CoreState,
        audio_events: &mut AudioEventChannel,
    ) {
        // Only process key presses if enabled
        match self.state {
            VisibilityState::Hidden => {}
            VisibilityState::Visible => {
                self.update_internal(state, parameters, audio_events);
            }
        }
    }
}
mod menu_item_data_prefabs {
    use crate::menu::MenuItemData;
    use crate::GameUInt;
    use alloc::boxed::Box;
    use alloc::vec;
    use alloc::vec::Vec;
    use num_bigint::BigUint;

    /// TODO: wrap a cost_fn helper that takes a max count to avoid the None check?
    pub(super) fn all() -> Vec<MenuItemData> {
        vec![
            pasta_sell_price(),
            dough_tick(),
            auto_cranker(),
            kneading_grannies(),
        ]
    }

    fn pasta_cost(count: u32) -> GameUInt {
        (BigUint::from(20usize) + BigUint::from(20usize).pow(count)).into()
    }
    fn pasta_sell_price() -> MenuItemData {
        MenuItemData {
            name: "Pasta Sell Price".into(),
            description: "Sell pasta for more".into(),
            count: 0,
            cost_fn: Box::new(|count| {
                if count > 10 {
                    return None;
                }
                let cost = pasta_cost(count as u32) * GameUInt::from(6usize);
                Some(cost.into())
            }),
            on_buy_fn: Box::new(|count, _state, parameters| {
                parameters.pasta_price = pasta_cost(count as u32);
            }),
        }
    }
    fn dough_tick() -> MenuItemData {
        MenuItemData {
            name: "Knead for Speed".into(),
            description: "Knead faster".into(),
            count: 0,
            cost_fn: Box::new(|count| {
                if count > 10 {
                    return None;
                }
                let cost: BigUint = BigUint::from(10usize).pow(count as u32);
                Some(cost.into())
            }),
            on_buy_fn: Box::new(|count, _state, parameters| {
                parameters.knead_tick_size = 0.01 + (count as f32 * 0.01);
            }),
        }
    }

    fn auto_cranker() -> MenuItemData {
        MenuItemData {
            name: "Auto-cranker".into(),
            description: "Automatically crank".into(),
            count: 0,
            cost_fn: Box::new(|count| {
                if count > 10 {
                    return None;
                }
                let cost: BigUint = BigUint::from(10usize).pow(count as u32);
                Some(cost.into())
            }),
            on_buy_fn: Box::new(|count, _state, parameters| parameters.auto_crank_level = count),
        }
    }

    fn kneading_grannies() -> MenuItemData {
        MenuItemData {
            name: "Kneading Grans".into(),
            description: "Hire Grans to Knead".into(),
            count: 0,
            cost_fn: Box::new(|count| {
                if count > 10 {
                    return None;
                }
                let cost: BigUint = BigUint::from(10usize).pow(count as u32);
                Some(cost.into())
            }),
            on_buy_fn: Box::new(|count, _state, parameters| parameters.auto_knead_level = count),
        }
    }
}

struct MenuItemData {
    name: String,
    description: String,
    count: usize,
    cost_fn: Box<dyn Fn(usize) -> Option<GameUInt>>,
    // TODO: Think about how this buy_fn is deterministic (in terms of saving an loading state) as
    // well as not overwriting other items (i.e. if both would change/set dough tick size)
    on_buy_fn: Box<dyn Fn(usize, &mut CoreState, &mut CoreParameters)>,
}

impl Debug for MenuItemData {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MenuItemData")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("count", &self.count)
            .field("cost_fn", &"<opaque")
            .field("on_buy_fn", &"<opaque")
            .finish()
    }
}

impl MenuItemData {
    fn cost(&self) -> Option<GameUInt> {
        (self.cost_fn)(self.count)
    }
    fn cost_str(&self) -> String {
        match self.cost() {
            Some(cost) => cost.to_string_hum(),
            None => return "Complete".into(),
        }
    }

    fn on_buy(&self, state: &mut CoreState, parameters: &mut CoreParameters) {
        (self.on_buy_fn)(self.count, state, parameters);
    }

    fn new_test(i: usize) -> Self {
        Self {
            name: format!("{}", i),
            description: format!("Menu item {}", i),
            count: 1,
            cost_fn: Box::new(move |count| {
                let cost: BigUint = BigUint::from(100 + i) * BigUint::from(count);
                Some(cost.into())
            }),
            on_buy_fn: Box::new(move |count, state, parameters| {
                parameters.knead_tick_size += 0.1;
            }),
        }
    }
}

#[derive(Debug)]
pub struct MenuItem {
    data: MenuItemData,
    sprite: Sprite,
    state: VisibilityState,
    y_offset: BoundedPosition,
    name_text: TextSprite,
    desc_text: TextSprite,
    cost_text: TextSprite,
    selected_image: Bitmap,
    unselected_image: Bitmap,
    pressed_image: Bitmap,
}

impl MenuItem {
    const NAME_TEXT_OFFSET: f32 = -18.0;
    const COST_TEXT_OFFSET: f32 = 18.0;
    pub fn new(data: MenuItemData, y_offset: BoundedPosition) -> Self {
        let y = y_offset.get();
        let mut sprite =
            crate::helpers::load_sprite_at("res/menu_item_background0", -95.0, y, None);
        sprite.set_z_index(10).unwrap();
        let mut name_text = TextSprite::new(
            "",
            crankstart::graphics::LCDColor::Solid(crankstart_sys::LCDSolidColor::kColorWhite),
        )
        .unwrap();
        name_text
            .get_sprite_mut()
            .move_to(-95.0, y + Self::NAME_TEXT_OFFSET)
            .unwrap();
        name_text.get_sprite_mut().set_z_index(11).unwrap();
        let mut desc_text = TextSprite::new(
            "",
            crankstart::graphics::LCDColor::Solid(crankstart_sys::LCDSolidColor::kColorWhite),
        )
        .unwrap();
        desc_text.get_sprite_mut().move_to(-95.0, y).unwrap();
        desc_text.get_sprite_mut().set_z_index(11).unwrap();
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
            name_text,
            desc_text,
            cost_text,
            selected_image,
            unselected_image,
            pressed_image,
        };
        t.update_text();
        t
    }

    fn update_text(&mut self) {
        let name_str = format!("{}: {}", self.data.name, self.data.count);
        self.name_text.update_text(name_str).unwrap();
        let cost_str = format!("Cost: {}", self.data.cost_str());
        self.cost_text.update_text(cost_str).unwrap();
        self.desc_text.update_text(&self.data.description).unwrap();
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

    /// Press menu item. Returns true if successfully bought, false if not.
    pub fn press_and_trigger(
        &mut self,
        state: &mut CoreState,
        parameters: &mut CoreParameters,
    ) -> bool {
        if let Some(cost) = self.data.cost() {
            System::log_to_console(&format!(
                "Trying to buy: cost: {}, with cash {}",
                cost.to_string_hum(),
                state.money.to_string_hum()
            ));
            if cost < state.money {
                self.data.count += 1;
                state.money.sub_assign(cost);
                self.data.on_buy(state, parameters);
                self.update_text();
                true
            } else {
                false
            }
        } else {
            log_to_console!("No item cost is None, which means it's at max");
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
        self.name_text
            .get_sprite_mut()
            .move_to(94.5, y + Self::NAME_TEXT_OFFSET)
            .unwrap();
        self.desc_text.get_sprite_mut().move_to(94.5, y).unwrap();
        self.cost_text
            .get_sprite_mut()
            .move_to(94.5, y + Self::COST_TEXT_OFFSET)
            .unwrap();
    }
    pub fn set_state(&mut self, state: VisibilityState) {
        self.state = state;
        let y = self.y_offset.get();
        let x = match state {
            VisibilityState::Hidden => -95.5,
            VisibilityState::Visible => 94.5,
        };
        self.sprite.move_to(x, y).unwrap();
        self.name_text
            .get_sprite_mut()
            .move_to(x, y + Self::NAME_TEXT_OFFSET)
            .unwrap();
        self.desc_text.get_sprite_mut().move_to(x, y).unwrap();
        self.cost_text
            .get_sprite_mut()
            .move_to(x, y + Self::COST_TEXT_OFFSET)
            .unwrap();
    }
}

#[derive(Debug)]
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
