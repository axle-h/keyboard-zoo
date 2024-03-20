use sdl2::surface::Surface;
use sdl2::sys::{image, SDL_RWFromMem};
use sdl2::{get_error, libc};

const ICON_FILE: &[u8] = include_bytes!("../icon.png");

fn surface_from_buffer(buf: &'static [u8]) -> Result<Surface<'static>, String> {
    //! Loads an SDL Surface from a byte buffer
    unsafe {
        let buf = SDL_RWFromMem(buf.as_ptr() as *mut libc::c_void, buf.len() as i32);
        let raw = image::IMG_Load_RW(buf, 1);
        if (raw as *mut ()).is_null() {
            Err(get_error())
        } else {
            Ok(Surface::from_ll(raw))
        }
    }
}

pub fn app_icon() -> Result<Surface<'static>, String> {
    surface_from_buffer(ICON_FILE)
}
