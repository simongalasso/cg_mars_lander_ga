use piston::input::*;

use crate::game::game::*;

pub fn handle_args(e: &Event, game: &mut Game) {
    if let Some(args) = e.press_args() {
        match args {
            Button::Keyboard(Key::Space) => {
                game.paused = !game.paused;
            },
            Button::Keyboard(Key::Right) => if game.paused {
                game.next_turn = true;
            },
            // Button::Keyboard(Key::Left) => {
            // }
            _ => {}
        }
    }
}