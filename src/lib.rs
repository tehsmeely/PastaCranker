#![no_std]
extern crate alloc;

use alloc::rc::Rc;
use alloc::vec::Vec;
use alloc::{format, vec};
use core::cell::RefCell;
use core::fmt::Debug;
use crankstart::log_to_console;
use crankstart::sprite::Sprite;
use crankstart::system::MenuItem;
use crankstart_sys::PDRect;
use {
    alloc::boxed::Box,
    anyhow::Error,
    crankstart::{
        crankstart_game,
        graphics::{Graphics, LCDColor, LCDSolidColor},
        system::System,
        Game, Playdate,
    },
};
mod audio_events;
mod bottom_bar;
mod core_elements;
mod dough_store;
mod fill_bar;
mod flour_pile;
mod game_value;
mod helpers;
mod info_overlay;
mod level_select;
mod machine;
mod menu;
mod save;

use crate::audio_events::{AudioEventChannel, SoundStore};
use crate::bottom_bar::BottomBar;
use crate::core_elements::{CoreParameters, CoreState, Timer};
use crate::flour_pile::FlourPile;
use crate::game_value::GameUInt;
use crate::info_overlay::InfoOverlay;
use crate::level_select::LevelSelect;
use crate::menu::Menu;
use machine::PastaMachineState;

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
enum SpriteType {
    MachineCrank,
    MachineBody,
    MachineDough,
    DoughStoreDough,
    BottomBar,
    FillBar,
    FlourPile,
    AButtonIndicator,
    Menu,
}

impl From<u8> for SpriteType {
    fn from(val: u8) -> Self {
        match val {
            0 => Self::MachineCrank,
            1 => Self::MachineBody,
            2 => Self::MachineDough,
            3 => Self::DoughStoreDough,
            4 => Self::BottomBar,
            5 => Self::FillBar,
            6 => Self::FlourPile,
            7 => Self::AButtonIndicator,
            8 => Self::Menu,
            _ => panic!("Unknown sprite type {}", val),
        }
    }
}

#[derive(Debug)]
struct GameState {
    parameters: CoreParameters,
    state: CoreState,
    pasta_machine: PastaMachineState,
    bottom_bar: BottomBar,
    flour_pile: FlourPile,
    menu: Menu,
    save_timer: Timer,
    sound_store: SoundStore,
    audio_event_channel: AudioEventChannel,
    save_index: usize,
    info_overlay: Rc<RefCell<InfoOverlay>>,
    system_menu_items: SystemMenuItems,
}

struct SystemMenuItems(Vec<MenuItem>);
impl Debug for SystemMenuItems {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let filling = format!("<..{}..>", self.0.len());
        f.debug_tuple("SystemMenuItems").field(&filling).finish()
    }
}

impl GameState {
    pub fn new(save_index: usize) -> Result<Self, Error> {
        crankstart::display::Display::get().set_refresh_rate(20.0)?;
        let (state, parameters, menu_counts, is_new_game) = match save::load_state(save_index) {
            Ok((state, parameters, menu_counts)) => {
                System::log_to_console("Loaded save");
                (state, parameters, Some(menu_counts), false)
            }
            Err(e) => {
                log_to_console!("Failed to load save, using defaults. Error: {:?}", e);
                (CoreState::default(), CoreParameters::default(), None, true)
            }
        };
        let info_overlay = Rc::new(RefCell::new(InfoOverlay::new(is_new_game)));
        let info_menu_item = {
            let info_overlay_clone = info_overlay.clone();
            System::get().add_menu_item(
                "Show Help",
                Box::new(move || {
                    info_overlay_clone.borrow_mut().show();
                }),
            )?
        };
        let system_menu_items = SystemMenuItems(vec![info_menu_item]);
        let mut menu = Menu::new();
        if let Some(counts) = menu_counts {
            menu.init_counts(&counts);
        }
        let sound_store = SoundStore::new()?;
        Ok(Self {
            parameters,
            state,
            pasta_machine: PastaMachineState::new(),
            bottom_bar: BottomBar::new(),
            flour_pile: FlourPile::new((80.0, 80.0)),
            menu,
            save_timer: Timer::new(5.0),
            sound_store,
            audio_event_channel: AudioEventChannel::new(),
            save_index,
            info_overlay,
            system_menu_items,
        })
    }
}

#[derive(Debug)]
enum GameMode {
    LevelSelect(LevelSelect),
    Game(GameState),
}

impl GameState {
    fn update_sprite(
        &mut self,
        sprite: &mut Sprite,
        _playdate: &mut Playdate,
    ) -> Result<(), Error> {
        let sprite_type: SpriteType = sprite.get_tag()?.into();
        match sprite_type {
            SpriteType::MachineCrank => self.pasta_machine.update_crank(
                &mut self.state,
                &self.parameters,
                &mut self.audio_event_channel,
            ),
            SpriteType::MachineBody => self.pasta_machine.update(&mut self.state, &self.parameters),
            SpriteType::FillBar => self.flour_pile.fill_bar_update(),
            SpriteType::FlourPile => self.flour_pile.update(
                &mut self.state,
                &self.parameters,
                &mut self.audio_event_channel,
            ),
            SpriteType::BottomBar => self.bottom_bar.update(&self.state, &mut self.menu),
            SpriteType::Menu => self.menu.update(
                &mut self.parameters,
                &mut self.state,
                &mut self.audio_event_channel,
            ),
            SpriteType::DoughStoreDough
            | SpriteType::MachineDough
            | SpriteType::AButtonIndicator => {}
        }
        Ok(())
    }

    fn draw_sprite(
        &self,
        sprite: &Sprite,
        _bounds: &PDRect,
        _draw_rect: &PDRect,
        _playdate: &Playdate,
    ) -> Result<(), Error> {
        // This function only needs to implement drawing for sprites that set "use_custom_draw"
        let tag = sprite.get_tag()?.into();
        match tag {
            SpriteType::FillBar => self.flour_pile.draw_fill_bar()?,
            _ => {}
        }
        Ok(())
    }

    fn update(&mut self, _playdate: &mut Playdate) -> Result<(), Error> {
        let graphics = Graphics::get();
        graphics.clear_context().unwrap();

        audio_events::process_events(&mut self.audio_event_channel, &mut self.sound_store);
        self.save_timer.update();
        if self.save_timer.just_finished() {
            save::save_state(self.save_index, &self);
            self.save_timer.reset();
        }

        self.info_overlay.borrow_mut().update();
        Ok(())
    }
}

impl GameMode {
    fn new(_playdate: &Playdate) -> Result<Box<Self>, Error> {
        let level_select = LevelSelect::new();
        Ok(Box::new(GameMode::LevelSelect(level_select)))
    }
}

impl Game for GameMode {
    fn update_sprite(&mut self, sprite: &mut Sprite, playdate: &mut Playdate) -> Result<(), Error> {
        match self {
            GameMode::LevelSelect(_) => {}
            GameMode::Game(state) => state.update_sprite(sprite, playdate)?,
        }
        Ok(())
    }

    fn draw_sprite(
        &self,
        sprite: &Sprite,
        bounds: &PDRect,
        draw_rect: &PDRect,
        playdate: &Playdate,
    ) -> Result<(), Error> {
        // This function only needs to implement drawing for sprites that set "use_custom_draw"
        match self {
            GameMode::LevelSelect(_) => {}
            GameMode::Game(state) => state.draw_sprite(sprite, bounds, draw_rect, playdate)?,
        }
        Ok(())
    }

    fn update(&mut self, playdate: &mut Playdate) -> Result<(), Error> {
        match self {
            GameMode::LevelSelect(level_select) => {
                if let Some(selected_level_idx) = level_select.update() {
                    log_to_console!("El levelo selecte! {}", selected_level_idx);
                    let game_state = GameState::new(selected_level_idx)?;
                    *self = GameMode::Game(game_state);
                }
            }
            GameMode::Game(state) => state.update(playdate)?,
        }
        Ok(())
    }
}

crankstart_game!(GameMode);
