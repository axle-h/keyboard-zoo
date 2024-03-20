#![cfg_attr(not(test), windows_subsystem = "windows")]

use crate::sandbox::sandbox;

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
mod random;

fn main() -> Result<(), String> {
    main::main()
}

#[cfg(unix)]
mod main {
    use fork::{fork, Fork};
    use crate::sandbox::sandbox;
    use crate::keyboard_zoo::KeyboardZoo;

    pub fn main() -> Result<(), String> {
        let mut keyboard_zoo = KeyboardZoo::new()?;
        if keyboard_zoo.run_sandbox() {
            // Run sandbox in a forked process
            match fork() {
                Ok(Fork::Child) => sandbox(),
                Ok(Fork::Parent(_)) => keyboard_zoo.game(),
                Err(_) => Err("Sandbox fork failed".to_string()),
            }
        } else {
            keyboard_zoo.game()
        }
    }
}

#[cfg(windows)]
mod main {
    use std::thread;
    use crate::sandbox::sandbox;
    use crate::keyboard_zoo::KeyboardZoo;
    use winapi::um::wincon::{FreeConsole, AttachConsole, ATTACH_PARENT_PROCESS};

    pub fn main() -> Result<(), String> {
        unsafe {
            attach_parent_console();
        }
        let mut keyboard_zoo = KeyboardZoo::new()?;

        if keyboard_zoo.run_sandbox() {
            // Run sandbox in a thread
            thread::spawn(|| sandbox());
        }

        keyboard_zoo.game()
    }

    unsafe fn attach_parent_console() {
        FreeConsole();
        AttachConsole(ATTACH_PARENT_PROCESS);
    }
}
