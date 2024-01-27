#![windows_subsystem = "windows"]

mod build_info;
mod config;
mod frame_rate;
mod icon;
mod keyboard_zoo;
mod sandbox;
mod assets;
mod game_input;
mod game;
mod particles;
mod animate;
mod texture;
mod characters;

use crate::keyboard_zoo::KeyboardZoo;
use fork::{fork, Fork};
use crate::sandbox::sandbox;

fn main() -> Result<(), String> {
    let mut keyboard_zoo = KeyboardZoo::new()?;
    if keyboard_zoo.run_sandbox() {
        // Run a sandbox in a forked process
        match fork() {
            Ok(Fork::Child) => sandbox(),
            Ok(Fork::Parent(_)) => keyboard_zoo.game(),
            Err(_) => Err("Sandbox fork failed".to_string()),
        }
    } else {
        keyboard_zoo.game()
    }
}
