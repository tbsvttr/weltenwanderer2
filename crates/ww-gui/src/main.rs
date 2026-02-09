//! Pixel art GUI for Weltenwanderer — macroquad entry point.
//!
//! Manages the screen stack and main render loop. The virtual canvas
//! (480x270) is scaled to fit the window, giving pixel-perfect rendering.

use macroquad::prelude::*;

use ww_gui::app::AppState;
use ww_gui::screen::dice::DiceScreen;
use ww_gui::screen::explorer::ExplorerScreen;
use ww_gui::screen::graph::GraphScreen;
use ww_gui::screen::play::PlayScreen;
use ww_gui::screen::sheet::SheetScreen;
use ww_gui::screen::solo::SoloScreen;
use ww_gui::screen::timeline::TimelineScreen;
use ww_gui::screen::title::TitleScreen;
use ww_gui::screen::{Screen, ScreenId, Transition};
use ww_gui::theme::font::build_font_texture;
use ww_gui::theme::palette;
use ww_gui::theme::sprites::build_sprites;
use ww_gui::theme::{CANVAS_H, CANVAS_W, setup_virtual_canvas};

/// Create a screen instance for a given screen id.
fn make_screen(id: ScreenId) -> Box<dyn Screen> {
    match id {
        ScreenId::Title => Box::new(TitleScreen::new()),
        ScreenId::Explorer => Box::new(ExplorerScreen::new()),
        ScreenId::Graph => Box::new(GraphScreen::new()),
        ScreenId::Timeline => Box::new(TimelineScreen::new()),
        ScreenId::Play => Box::new(PlayScreen::new()),
        ScreenId::Solo => Box::new(SoloScreen::new()),
        ScreenId::Sheet => Box::new(SheetScreen::new()),
        ScreenId::Dice => Box::new(DiceScreen::new()),
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Weltenwanderer".to_owned(),
        window_width: (CANVAS_W * 2.0) as i32,
        window_height: (CANVAS_H * 2.0) as i32,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // Parse CLI args: --world <dir>
    let args: Vec<String> = std::env::args().collect();
    let world_dir = args
        .windows(2)
        .find(|w| w[0] == "--world")
        .map(|w| w[1].clone());

    // Build font and sprite assets
    let font = build_font_texture();
    let sprites = build_sprites();
    let mut app = AppState::new(font, sprites);

    // Screen stack
    let mut screens: Vec<Box<dyn Screen>> = Vec::new();

    // Start with title screen or explorer if --world given
    if let Some(dir) = &world_dir {
        let path = std::path::Path::new(dir);
        match app.load_world(path) {
            Ok(()) => {
                screens.push(make_screen(ScreenId::Explorer));
            }
            Err(e) => {
                eprintln!("Failed to load world: {e}");
                let mut title = TitleScreen::with_dir(dir);
                title.error = Some(e);
                screens.push(Box::new(title));
            }
        }
    } else {
        screens.push(make_screen(ScreenId::Title));
    }

    loop {
        // Clear with black (letterbox bars)
        clear_background(palette::BLACK);

        // Set up virtual canvas camera
        setup_virtual_canvas();

        // Draw canvas background
        draw_rectangle(0.0, 0.0, CANVAS_W, CANVAS_H, palette::DARK_BLUE);

        // Update + draw the top screen
        if let Some(screen) = screens.last_mut() {
            let transition = screen.update(&mut app);

            // Handle transition
            match transition {
                Transition::Push(id) => {
                    screens.push(make_screen(id));
                }
                Transition::Pop => {
                    screens.pop();
                }
                Transition::Replace(id) => {
                    screens.pop();
                    screens.push(make_screen(id));
                }
                Transition::None => {}
            }
        }

        // Draw the current top screen (may have changed after transition)
        if let Some(screen) = screens.last() {
            screen.draw(&app);
        }

        // Quit conditions
        if app.should_quit || screens.is_empty() {
            break;
        }

        next_frame().await;
    }
}
