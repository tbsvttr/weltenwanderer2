//! Visual theme: color palette, layout constants, and virtual canvas scaling.

pub mod font;
pub mod sprites;

use macroquad::prelude::*;

/// Virtual canvas width in pixels. The window scales this up.
pub const CANVAS_W: f32 = 480.0;
/// Virtual canvas height in pixels. The window scales this up.
pub const CANVAS_H: f32 = 270.0;

/// Tile size for grid alignment and font glyphs.
pub const TILE: f32 = 8.0;
/// Sprite size for entity-kind icons.
pub const SPRITE_SIZE: f32 = 16.0;

/// 16-color PICO-8-inspired palette.
pub mod palette {
    use macroquad::prelude::Color;

    /// Black background.
    pub const BLACK: Color = Color::new(0.0, 0.0, 0.0, 1.0);
    /// Dark blue for deep backgrounds.
    pub const DARK_BLUE: Color = Color::new(0.114, 0.169, 0.326, 1.0);
    /// Dark purple for accents.
    pub const DARK_PURPLE: Color = Color::new(0.494, 0.145, 0.326, 1.0);
    /// Dark green for nature elements.
    pub const DARK_GREEN: Color = Color::new(0.0, 0.529, 0.320, 1.0);
    /// Brown for earthy tones.
    pub const BROWN: Color = Color::new(0.671, 0.322, 0.212, 1.0);
    /// Dark gray for inactive elements.
    pub const DARK_GRAY: Color = Color::new(0.373, 0.341, 0.310, 1.0);
    /// Light gray for borders and secondary text.
    pub const LIGHT_GRAY: Color = Color::new(0.761, 0.765, 0.780, 1.0);
    /// White for primary text.
    pub const WHITE: Color = Color::new(1.0, 0.945, 0.910, 1.0);
    /// Red for errors and danger.
    pub const RED: Color = Color::new(1.0, 0.0, 0.302, 1.0);
    /// Orange for warnings.
    pub const ORANGE: Color = Color::new(1.0, 0.639, 0.0, 1.0);
    /// Yellow for highlights.
    pub const YELLOW: Color = Color::new(1.0, 0.925, 0.153, 1.0);
    /// Green for success.
    pub const GREEN: Color = Color::new(0.0, 0.894, 0.212, 1.0);
    /// Blue for links and info.
    pub const BLUE: Color = Color::new(0.161, 0.678, 1.0, 1.0);
    /// Indigo for factions and magic.
    pub const INDIGO: Color = Color::new(0.514, 0.463, 0.612, 1.0);
    /// Pink for special items.
    pub const PINK: Color = Color::new(1.0, 0.467, 0.659, 1.0);
    /// Peach for warm highlights.
    pub const PEACH: Color = Color::new(1.0, 0.800, 0.667, 1.0);
}

/// Color associated with an entity kind name.
pub fn kind_color(kind: &str) -> Color {
    match kind.to_lowercase().as_str() {
        "character" => palette::YELLOW,
        "location" => palette::GREEN,
        "faction" => palette::INDIGO,
        "event" => palette::ORANGE,
        "item" => palette::PINK,
        "lore" => palette::BLUE,
        "ruleset" => palette::PEACH,
        _ => palette::LIGHT_GRAY,
    }
}

/// Set up a `Camera2D` that maps the virtual canvas to the current window.
pub fn setup_virtual_canvas() {
    let scale_x = screen_width() / CANVAS_W;
    let scale_y = screen_height() / CANVAS_H;
    let scale = scale_x.min(scale_y);

    let viewport_w = CANVAS_W * scale;
    let viewport_h = CANVAS_H * scale;
    let offset_x = (screen_width() - viewport_w) / 2.0;
    let offset_y = (screen_height() - viewport_h) / 2.0;

    set_camera(&Camera2D {
        zoom: vec2(2.0 / CANVAS_W, 2.0 / CANVAS_H),
        target: vec2(CANVAS_W / 2.0, CANVAS_H / 2.0),
        viewport: Some((
            offset_x as i32,
            offset_y as i32,
            viewport_w as i32,
            viewport_h as i32,
        )),
        ..Default::default()
    });
}

/// Convert screen-space mouse position to virtual canvas coordinates.
pub fn mouse_canvas_position() -> (f32, f32) {
    let (mx, my) = mouse_position();
    let scale_x = screen_width() / CANVAS_W;
    let scale_y = screen_height() / CANVAS_H;
    let scale = scale_x.min(scale_y);

    let viewport_w = CANVAS_W * scale;
    let viewport_h = CANVAS_H * scale;
    let offset_x = (screen_width() - viewport_w) / 2.0;
    let offset_y = (screen_height() - viewport_h) / 2.0;

    let cx = (mx - offset_x) / scale;
    let cy = (my - offset_y) / scale;
    (cx, cy)
}
