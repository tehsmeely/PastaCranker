use crankstart::graphics::Graphics;
use crankstart::sprite::{Sprite, SpriteManager};
use crankstart_sys::LCDBitmapFlip;

pub fn wrap(val: f32, min: f32, max: f32) -> f32 {
    // Note: if val is bigger than min-max overshot, it'll not be returned in that range
    // i.e. if you pass in   -200, 0, 100, you'll get back -100 which is not what you expect
    if val < min {
        max + (val - min)
    } else if val > max {
        min + (val - max)
    } else {
        val
    }
}
pub enum WrapResult {
    NoWrap,
    Underflow,
    Overflow,
}
pub fn wrap_with_info(val: f32, min: f32, max: f32) -> (f32, WrapResult) {
    // Note: if val is bigger than min-max overshot, it'll not be returned in that range
    // i.e. if you pass in   -200, 0, 100, you'll get back -100 which is not what you expect
    if val < min {
        (max + (val - min), WrapResult::Underflow)
    } else if val > max {
        (min + (val - max), WrapResult::Overflow)
    } else {
        (val, WrapResult::NoWrap)
    }
}

pub fn load_sprite_at(filename: &str, x: f32, y: f32, tag: Option<u8>) -> Sprite {
    let sprite_manager = SpriteManager::get_mut();
    let mut sprite = sprite_manager.new_sprite().unwrap();
    let image = Graphics::get().load_bitmap(filename).unwrap();
    sprite
        .set_image(image, LCDBitmapFlip::kBitmapUnflipped)
        .unwrap();
    sprite.move_to(x, y).unwrap();
    if let Some(tag) = tag {
        sprite.set_tag(tag).unwrap();
    }
    sprite_manager.add_sprite(&sprite).unwrap();
    sprite
}
