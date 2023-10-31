#![no_std]

extern crate alloc;

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
mod bottom_bar;
mod core_elements;
mod dough_store;
mod fill_bar;
mod flour_pile;
mod helpers;
mod machine;

use crate::bottom_bar::BottomBar;
use crate::fill_bar::FillBar;
use crate::flour_pile::FlourPile;
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
            _ => panic!("Unknown sprite type {}", val),
        }
    }
}

struct State {
    pasta_machine: PastaMachineState,
    bottom_bar: BottomBar,
    flour_pile: FlourPile,
}

impl State {
    pub fn new(_playdate: &Playdate) -> Result<Box<Self>, Error> {
        crankstart::display::Display::get().set_refresh_rate(20.0)?;
        Ok(Box::new(Self {
            pasta_machine: PastaMachineState::new(),
            bottom_bar: BottomBar::new(),
            flour_pile: FlourPile::new((80.0, 80.0)),
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
            SpriteType::MachineCrank => self.pasta_machine.update_crank(),
            SpriteType::MachineBody => self.pasta_machine.update(&mut self.flour_pile),
            SpriteType::FillBar => self.flour_pile.fill_bar_update(),
            SpriteType::FlourPile => self.flour_pile.update(),
            SpriteType::MachineDough
            | SpriteType::DoughStoreDough
            | SpriteType::BottomBar
            | SpriteType::AButtonIndicator => {}
        }
        Ok(())
    }

    fn update(&mut self, _playdate: &mut Playdate) -> Result<(), Error> {
        let graphics = Graphics::get();
        graphics.clear(LCDColor::Solid(LCDSolidColor::kColorWhite))?;

        System::get().draw_fps(0, 0)?;

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
}

crankstart_game!(State);
