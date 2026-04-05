use std::error::Error;
use crate::game::SandboxGame;

pub mod audio;
pub mod game;
pub mod gui;
pub mod tools;
pub mod world;
pub mod script;

fn main() -> Result<(), Box<dyn Error>> {
    innovus::run_game_application::<SandboxGame>()
}
