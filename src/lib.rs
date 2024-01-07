#![no_std]
extern crate alloc;

use crankstart::log_to_console;
use crankstart::sprite::Sprite;
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
mod machine;
mod menu;
mod save;

use crate::audio_events::{AudioEventChannel, SoundStore};
use crate::bottom_bar::BottomBar;
use crate::core_elements::{CoreParameters, CoreState, Timer};
use crate::fill_bar::FillBar;
use crate::flour_pile::FlourPile;
use crate::game_value::GameUInt;
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

struct State {
    parameters: CoreParameters,
    state: CoreState,
    pasta_machine: PastaMachineState,
    bottom_bar: BottomBar,
    flour_pile: FlourPile,
    menu: Menu,
    save_timer: Timer,
    sound_store: SoundStore,
    audio_event_channel: AudioEventChannel,
}

impl State {
    pub fn new(_playdate: &Playdate) -> Result<Box<Self>, Error> {
        crankstart::display::Display::get().set_refresh_rate(20.0)?;
        let (state, parameters, menu_counts) = match save::load_state() {
            Ok((state, parameters, menu_counts)) => {
                System::log_to_console("Loaded save");
                (state, parameters, Some(menu_counts))
            }
            Err(e) => {
                log_to_console!("Failed to load save, using defaults. Error: {:?}", e);
                (CoreState::default(), CoreParameters::default(), None)
            }
        };
        let mut menu = Menu::new();
        if let Some(counts) = menu_counts {
            menu.init_counts(&counts);
        }
        let sound_store = SoundStore::new()?;
        Ok(Box::new(Self {
            parameters,
            state,
            pasta_machine: PastaMachineState::new(),
            bottom_bar: BottomBar::new(),
            flour_pile: FlourPile::new((80.0, 80.0)),
            menu,
            save_timer: Timer::new(5.0),
            sound_store,
            audio_event_channel: AudioEventChannel::new(),
        }))
    }
}

impl Game for State {
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
            SpriteType::MachineBody => self.pasta_machine.update(&mut self.state),
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
            save::save_state(&self);
            self.save_timer.reset();
        }

        System::get().draw_fps(0, 0)?;

        Ok(())
    }
}

crankstart_game!(State);
