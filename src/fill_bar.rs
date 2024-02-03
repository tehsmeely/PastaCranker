use crate::helpers::load_sprite_at;
use crate::SpriteType;
use alloc::format;
use anyhow::Error;
use crankstart::geometry::{ScreenPoint, ScreenRect};
use crankstart::graphics::{Graphics, LCDColor};
use crankstart::log_to_console;
use crankstart::sprite::Sprite;
use crankstart::system::System;
use crankstart_sys::{LCDBitmapFlip, LCDPattern, LCDSolidColor};
use euclid::{Point2D, Rect, Size2D};

#[derive(Debug)]
pub struct FillBar {
    fill_pct: f32,
    background_sprite: Sprite,
    full_fill_rect: ScreenRect,
    dirty: bool,
}

impl FillBar {
    pub fn new(pos: (f32, f32)) -> Self {
        let mut background_sprite = load_sprite_at(
            "res/bar_background",
            pos.0,
            pos.1,
            Some(SpriteType::FillBar as u8),
        );
        let full_fill_rect = {
            let origin = Point2D::new(pos.0 as i32 - 3, pos.1 as i32 - 16);
            Rect::new(origin, Size2D::new(5, 32))
        };
        background_sprite.set_use_custom_draw().unwrap();
        Self {
            fill_pct: 0.0,
            background_sprite,
            full_fill_rect,
            dirty: true,
        }
    }

    pub fn update(&mut self) {
        if self.dirty {
            self.background_sprite.mark_dirty().unwrap();
            self.dirty = false;
        }
    }

    pub fn get_fill_rect(&self) -> ScreenRect {
        // Scale height by fill pct, and move origin down by that amount too so bottom left is fixed
        let height = (self.full_fill_rect.size.height as f32 * self.fill_pct) as i32;
        let y_diff = self.full_fill_rect.size.height - height;
        let mut rect = self.full_fill_rect.clone();
        rect.origin.y += y_diff;
        rect.size.height = height;
        rect
    }

    pub fn set_fill_pct(&mut self, pct: f32) {
        self.fill_pct = f32::clamp(pct, 0.0, 1.0);
        self.dirty = true;
    }

    pub fn incr_fill_pct(&mut self, pct: f32) {
        self.set_fill_pct(self.fill_pct + pct);
    }
    pub fn get_fill_pct(&self) -> f32 {
        self.fill_pct
    }

    pub fn draw(&self) -> Result<(), Error> {
        if let Some(image) = self.background_sprite.get_image()? {
            let bounds = self.background_sprite.get_bounds()?;
            // location is topleft not center like for normal sprite positioning
            let location = ScreenPoint::new(bounds.x as i32, bounds.y as i32);
            image.draw(location, LCDBitmapFlip::kBitmapUnflipped)?;
            let graphics = Graphics::get();
            graphics.fill_rect(
                self.get_fill_rect(),
                LCDColor::Solid(LCDSolidColor::kColorBlack),
            )?;
        }
        Ok(())
    }
}
