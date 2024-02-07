use rdev::{grab, Event, EventType, Key, GrabError};

pub fn sandbox() -> Result<(), String> {
    println!("running toddler sandbox");
    let callback = |event: Event| -> Option<Event> {
        match event.event_type {
            EventType::KeyPress(key) | EventType::KeyRelease(key) => {
                match key {
                    Key::Alt
                    | Key::AltGr
                    | Key::Backspace
                    | Key::CapsLock
                    | Key::ControlLeft
                    | Key::ControlRight
                    | Key::Delete
                    | Key::End
                    | Key::F1
                    | Key::F10
                    | Key::F11
                    | Key::F12
                    | Key::F2
                    | Key::F3
                    | Key::F4
                    | Key::F5
                    | Key::F6
                    | Key::F7
                    | Key::F8
                    | Key::F9
                    | Key::Home
                    | Key::MetaLeft
                    | Key::MetaRight
                    | Key::PageDown
                    | Key::PageUp
                    | Key::Return
                    | Key::ShiftLeft
                    | Key::ShiftRight
                    | Key::Tab
                    | Key::PrintScreen
                    | Key::ScrollLock
                    | Key::Pause
                    | Key::NumLock
                    | Key::Insert
                    | Key::KpReturn
                    | Key::KpDelete
                    | Key::Function
                    | Key::Unknown(_) => {
                        if let EventType::KeyPress(key) = event.event_type {
                            println!("sandbox trap: {:?}", key);
                        }
                        None
                    }
                    _ => Some(event),
                }
            },
            _ => Some(event),
        }
    };

    if let Err(error) = grab(callback) {
        println!("failed to start toddler sandbox {:?}", error);
        Err("failed to start toddler sandbox".to_string())
    } else {
        Ok(())
    }
}
