//! Procedural 16x16 pixel art sprites for entity-kind icons.
//!
//! Each sprite is defined as a 16x16 grid of palette indices.
//! Index 0 = transparent, 1-15 map to the palette colors.

use macroquad::prelude::*;

use super::palette;

/// Palette lookup: index 0 is transparent, 1-15 are colors.
fn palette_color(idx: u8) -> Color {
    match idx {
        0 => Color::new(0.0, 0.0, 0.0, 0.0),
        1 => palette::BLACK,
        2 => palette::DARK_BLUE,
        3 => palette::DARK_PURPLE,
        4 => palette::DARK_GREEN,
        5 => palette::BROWN,
        6 => palette::DARK_GRAY,
        7 => palette::LIGHT_GRAY,
        8 => palette::WHITE,
        9 => palette::RED,
        10 => palette::ORANGE,
        11 => palette::YELLOW,
        12 => palette::GREEN,
        13 => palette::BLUE,
        14 => palette::INDIGO,
        15 => palette::PINK,
        _ => palette::WHITE,
    }
}

/// A 16x16 sprite icon.
pub struct SpriteIcon {
    /// The texture for this sprite.
    pub texture: Texture2D,
}

/// Build a sprite texture from a 16x16 palette-indexed grid.
fn build_sprite(data: &[u8; 256]) -> SpriteIcon {
    let mut pixels = [0u8; 16 * 16 * 4];
    for (i, &idx) in data.iter().enumerate() {
        let color = palette_color(idx);
        let p = i * 4;
        pixels[p] = (color.r * 255.0) as u8;
        pixels[p + 1] = (color.g * 255.0) as u8;
        pixels[p + 2] = (color.b * 255.0) as u8;
        pixels[p + 3] = (color.a * 255.0) as u8;
    }
    let texture = Texture2D::from_rgba8(16, 16, &pixels);
    texture.set_filter(FilterMode::Nearest);
    SpriteIcon { texture }
}

/// Character icon: a simple humanoid/sword silhouette.
#[rustfmt::skip]
const CHARACTER_DATA: [u8; 256] = [
    0,0,0,0,0,0,11,11,11,11,0,0,0,0,0,0,
    0,0,0,0,0,11,11,11,11,11,11,0,0,0,0,0,
    0,0,0,0,0,11,11,11,11,11,11,0,0,0,0,0,
    0,0,0,0,0,0,11,11,11,11,0,0,0,0,0,0,
    0,0,0,0,0,0,0,11,11,0,0,0,0,0,0,0,
    0,0,0,11,11,11,11,11,11,11,11,11,11,0,0,0,
    0,0,0,0,11,11,11,11,11,11,11,11,0,0,0,0,
    0,0,0,0,0,11,11,11,11,11,11,0,0,0,0,0,
    0,0,0,0,0,0,11,11,11,11,0,0,0,0,0,0,
    0,0,0,0,0,0,11,11,11,11,0,0,0,0,0,0,
    0,0,0,0,0,0,11,11,11,11,0,0,0,0,0,0,
    0,0,0,0,0,0,11,11,11,11,0,0,0,0,0,0,
    0,0,0,0,0,0,11,0,0,11,0,0,0,0,0,0,
    0,0,0,0,0,11,11,0,0,11,11,0,0,0,0,0,
    0,0,0,0,0,11,11,0,0,11,11,0,0,0,0,0,
    0,0,0,0,0,11,11,0,0,11,11,0,0,0,0,0,
];

/// Location icon: a castle/tower silhouette.
#[rustfmt::skip]
const LOCATION_DATA: [u8; 256] = [
    0,0,12,0,0,0,0,12,12,0,0,0,0,12,0,0,
    0,0,12,0,0,0,0,12,12,0,0,0,0,12,0,0,
    0,12,12,12,0,0,12,12,12,12,0,0,12,12,12,0,
    0,12,12,12,0,0,12,12,12,12,0,0,12,12,12,0,
    0,12,12,12,12,12,12,12,12,12,12,12,12,12,12,0,
    0,12,12,12,12,12,12,12,12,12,12,12,12,12,12,0,
    0,12,12,12,12,12,12,12,12,12,12,12,12,12,12,0,
    0,12, 1,12,12,12,12, 1, 1,12,12,12,12, 1,12,0,
    0,12, 1,12,12,12,12, 1, 1,12,12,12,12, 1,12,0,
    0,12, 1,12,12,12,12, 1, 1,12,12,12,12, 1,12,0,
    0,12,12,12,12,12,12,12,12,12,12,12,12,12,12,0,
    0,12,12,12,12,12,12,12,12,12,12,12,12,12,12,0,
    0,12,12,12,12,12, 5, 5, 5, 5,12,12,12,12,12,0,
    0,12,12,12,12,12, 5, 1, 1, 5,12,12,12,12,12,0,
    0,12,12,12,12,12, 5, 1, 1, 5,12,12,12,12,12,0,
    0,12,12,12,12,12, 5, 1, 1, 5,12,12,12,12,12,0,
];

/// Faction icon: a banner/flag.
#[rustfmt::skip]
const FACTION_DATA: [u8; 256] = [
    0,0,0,0,0,14,14,0,0,0,0,0,0,0,0,0,
    0,0,0,0,0,14,14,14,14,14,14,14,14,0,0,0,
    0,0,0,0,0,14,14,14,14,14,14,14,14,14,0,0,
    0,0,0,0,0,14, 8, 8, 8, 8, 8, 8,14,14,0,0,
    0,0,0,0,0,14, 8, 8, 8, 8, 8, 8,14,0,0,0,
    0,0,0,0,0,14, 8, 8, 8, 8, 8, 8,14,0,0,0,
    0,0,0,0,0,14,14,14,14,14,14,14,14,0,0,0,
    0,0,0,0,0,14,14,14,14,14,14,14,0,0,0,0,
    0,0,0,0,0,14,14,0,0,0,0,0,0,0,0,0,
    0,0,0,0,0,14,14,0,0,0,0,0,0,0,0,0,
    0,0,0,0,0,14,14,0,0,0,0,0,0,0,0,0,
    0,0,0,0,0,14,14,0,0,0,0,0,0,0,0,0,
    0,0,0,0,0,14,14,0,0,0,0,0,0,0,0,0,
    0,0,0,0,0,14,14,0,0,0,0,0,0,0,0,0,
    0,0,0,0,0,14,14,0,0,0,0,0,0,0,0,0,
    0,0,0,0,0,14,14,0,0,0,0,0,0,0,0,0,
];

/// Event icon: a scroll/parchment.
#[rustfmt::skip]
const EVENT_DATA: [u8; 256] = [
    0,0,0,0,10,10,10,10,10,10,10,10,0,0,0,0,
    0,0,0,10, 5, 5, 5, 5, 5, 5, 5, 5,10,0,0,0,
    0,0,10, 5, 8, 8, 8, 8, 8, 8, 8, 5,10,0,0,0,
    0,0,10, 5, 8, 1, 1, 1, 1, 1, 8, 5,10,0,0,0,
    0,0,10, 5, 8, 8, 8, 8, 8, 8, 8, 5,10,0,0,0,
    0,0,10, 5, 8, 1, 1, 1, 1, 8, 8, 5,10,0,0,0,
    0,0,10, 5, 8, 8, 8, 8, 8, 8, 8, 5,10,0,0,0,
    0,0,10, 5, 8, 1, 1, 1, 1, 1, 8, 5,10,0,0,0,
    0,0,10, 5, 8, 8, 8, 8, 8, 8, 8, 5,10,0,0,0,
    0,0,10, 5, 8, 1, 1, 1, 8, 8, 8, 5,10,0,0,0,
    0,0,10, 5, 8, 8, 8, 8, 8, 8, 8, 5,10,0,0,0,
    0,0,10, 5, 8, 1, 1, 1, 1, 1, 8, 5,10,0,0,0,
    0,0,10, 5, 8, 8, 8, 8, 8, 8, 8, 5,10,0,0,0,
    0,0,0,10, 5, 5, 5, 5, 5, 5, 5,10,0,0,0,0,
    0,0,0,0,10,10,10,10,10,10,10,10,0,0,0,0,
    0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
];

/// Item icon: a gem/crystal.
#[rustfmt::skip]
const ITEM_DATA: [u8; 256] = [
    0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
    0,0,0,0,0,0,15,15,15,15,0,0,0,0,0,0,
    0,0,0,0,0,15,15,15,15,15,15,0,0,0,0,0,
    0,0,0,0,15,15, 8,15,15,15,15,15,0,0,0,0,
    0,0,0,15,15, 8,15,15,15,15,15,15,15,0,0,0,
    0,0,15,15, 8,15,15,15,15,15,15,15,15,15,0,0,
    0,0,15,15,15,15,15,15,15,15,15,15,15,15,0,0,
    0,0,0,15,15,15,15,15,15,15,15,15,15,0,0,0,
    0,0,0,0,15,15,15,15,15,15,15,15,0,0,0,0,
    0,0,0,0,0,15,15,15,15,15,15,0,0,0,0,0,
    0,0,0,0,0,0,15, 3,15,15,0,0,0,0,0,0,
    0,0,0,0,0,0,0, 3, 3,0,0,0,0,0,0,0,
    0,0,0,0,0,0,0, 3,0,0,0,0,0,0,0,0,
    0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
    0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
    0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
];

/// Lore icon: an open book.
#[rustfmt::skip]
const LORE_DATA: [u8; 256] = [
    0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
    0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
    0,0,13,13,13,13,13,0,0,13,13,13,13,13,0,0,
    0,13, 8, 8, 8, 8,13,0,0,13, 8, 8, 8, 8,13,0,
    0,13, 8, 1, 1, 8,13,0,0,13, 8, 1, 1, 8,13,0,
    0,13, 8, 8, 8, 8,13,0,0,13, 8, 8, 8, 8,13,0,
    0,13, 8, 1, 1, 8,13,0,0,13, 8, 1, 1, 8,13,0,
    0,13, 8, 8, 8, 8,13,0,0,13, 8, 8, 8, 8,13,0,
    0,13, 8, 1, 1, 8,13,0,0,13, 8, 1, 8, 8,13,0,
    0,13, 8, 8, 8, 8,13,0,0,13, 8, 8, 8, 8,13,0,
    0,13, 8, 1, 8, 8,13,0,0,13, 8, 1, 1, 8,13,0,
    0,0,13, 8, 8,13,13,13,13,13,13, 8, 8,13,0,0,
    0,0,0,13,13,13, 5, 5, 5, 5,13,13,13,0,0,0,
    0,0,0,0,13,13,13,13,13,13,13,13,0,0,0,0,
    0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
    0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
];

/// All built sprite icons, indexed by kind name.
pub struct SpriteSet {
    /// Character icon.
    pub character: SpriteIcon,
    /// Location icon.
    pub location: SpriteIcon,
    /// Faction icon.
    pub faction: SpriteIcon,
    /// Event icon.
    pub event: SpriteIcon,
    /// Item icon.
    pub item: SpriteIcon,
    /// Lore/book icon.
    pub lore: SpriteIcon,
}

/// Build all sprite textures. Call once at startup.
pub fn build_sprites() -> SpriteSet {
    SpriteSet {
        character: build_sprite(&CHARACTER_DATA),
        location: build_sprite(&LOCATION_DATA),
        faction: build_sprite(&FACTION_DATA),
        event: build_sprite(&EVENT_DATA),
        item: build_sprite(&ITEM_DATA),
        lore: build_sprite(&LORE_DATA),
    }
}

impl SpriteSet {
    /// Get the sprite texture for a given entity kind name.
    pub fn for_kind(&self, kind: &str) -> &Texture2D {
        match kind.to_lowercase().as_str() {
            "character" => &self.character.texture,
            "location" => &self.location.texture,
            "faction" => &self.faction.texture,
            "event" => &self.event.texture,
            "item" => &self.item.texture,
            "lore" | "ruleset" => &self.lore.texture,
            _ => &self.lore.texture,
        }
    }
}
