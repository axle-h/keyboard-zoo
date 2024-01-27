use crate::build_info::APP_NAME;
use confy::ConfyError;
use sdl2::keyboard::Keycode;
use sdl2::mixer::MAX_VOLUME;
use serde::{Deserialize, Serialize};

pub const APP_CONFIG_ROOT: &str = APP_NAME;
const CONFIG_NAME: &str = "config";

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Config {
    pub input: InputConfig,
    pub video: VideoConfig,
    pub audio: AudioConfig,
    pub physics: PhysicsConfig
}

impl Config {
    pub fn load() -> Result<Self, String> {
        let config_path = confy::get_configuration_file_path(APP_CONFIG_ROOT, CONFIG_NAME)
            .map_err(|e| e.to_string())?;

        #[cfg(debug_assertions)]
        println!("loading config: {}", config_path.to_str().unwrap());

        match confy::load(APP_CONFIG_ROOT, CONFIG_NAME) {
            Ok(config) => Ok(config),
            Err(ConfyError::BadYamlData(error)) => {
                Err(format!(
                    "Bad config file at {}, {}",
                    config_path.to_str().unwrap(),
                    error
                ))
            }
            Err(error) => Err(format!("{}", error)),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct InputConfig {
    pub run_toddler_sandbox: bool,
    pub baby_smash_mode: bool,

    #[serde(with = "KeycodeDef")]
    pub up: Keycode,
    #[serde(with = "KeycodeDef")]
    pub down: Keycode,
    #[serde(with = "KeycodeDef")]
    pub left: Keycode,
    #[serde(with = "KeycodeDef")]
    pub right: Keycode,
    #[serde(with = "KeycodeDef")]
    pub nuke: Keycode,
    #[serde(with = "KeycodeDef")]
    pub explosion: Keycode,
    #[serde(with = "KeycodeDef")]
    pub spawn_character: Keycode,
    #[serde(with = "KeycodeDef")]
    pub spawn_asset: Keycode,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct AudioConfig {
    pub music_volume: f64,
    pub effects_volume: f64,
}

impl AudioConfig {
    pub fn music_volume(&self) -> i32 {
        (self.music_volume * MAX_VOLUME as f64).round() as i32
    }

    pub fn effects_volume(&self) -> i32 {
        (self.effects_volume * MAX_VOLUME as f64).round() as i32
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VideoMode {
    Window { width: u32, height: u32 },
    FullScreen { width: u32, height: u32 },
    FullScreenDesktop,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct VideoConfig {
    pub mode: VideoMode,
    pub vsync: bool,
    pub disable_screensaver: bool,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct PhysicsConfig {
    pub debug_draw: bool,
    pub pixels_per_meter: f32,
    pub polygon_scale: f32,
    pub velocity_iterations: i32,
    pub position_iterations: i32,
    pub gravity: f32,
    pub push_force_magnitude: f32,
    pub body_density: f32,
    pub body_friction: f32,
    pub body_restitution: f32,
    pub explosion_force_magnitude: f32,
    pub explosion_distance: f32
}

impl Default for Config {
    fn default() -> Self {
        Self {
            input: InputConfig {
                run_toddler_sandbox: false,
                baby_smash_mode: true,
                up: Keycode::Up,
                down: Keycode::Down,
                left: Keycode::Left,
                right: Keycode::Right,
                nuke: Keycode::Backspace,
                explosion: Keycode::Space,
                spawn_character: Keycode::RShift,
                spawn_asset: Keycode::Return,
            },
            video: VideoConfig {
                mode: VideoMode::Window {
                    width: 1280,
                    height: 720,
                },
                vsync: true,
                disable_screensaver: true,
            },
            audio: AudioConfig {
                music_volume: 0.5,
                effects_volume: 1.0,
            },
            physics: PhysicsConfig {
                debug_draw: false,
                pixels_per_meter: 10.0,
                polygon_scale: 20.0,
                velocity_iterations: 8,
                position_iterations: 3,
                gravity: 1.0, // positive gravity as we are rendering upside down when converting between sdl & box2d
                push_force_magnitude: 2.0,
                body_density: 0.001,
                body_friction: 0.3,
                body_restitution: 0.5,
                explosion_force_magnitude: 200.0,
                explosion_distance: 100.0,
            }
        }
    }
}

/// redefined here for serde sigh
#[derive(Serialize, Deserialize)]
#[serde(remote = "Keycode")]
enum KeycodeDef {
    Backspace = sdl2::sys::SDL_KeyCode::SDLK_BACKSPACE as isize,
    Tab = sdl2::sys::SDL_KeyCode::SDLK_TAB as isize,
    Return = sdl2::sys::SDL_KeyCode::SDLK_RETURN as isize,
    Escape = sdl2::sys::SDL_KeyCode::SDLK_ESCAPE as isize,
    Space = sdl2::sys::SDL_KeyCode::SDLK_SPACE as isize,
    Exclaim = sdl2::sys::SDL_KeyCode::SDLK_EXCLAIM as isize,
    Quotedbl = sdl2::sys::SDL_KeyCode::SDLK_QUOTEDBL as isize,
    Hash = sdl2::sys::SDL_KeyCode::SDLK_HASH as isize,
    Dollar = sdl2::sys::SDL_KeyCode::SDLK_DOLLAR as isize,
    Percent = sdl2::sys::SDL_KeyCode::SDLK_PERCENT as isize,
    Ampersand = sdl2::sys::SDL_KeyCode::SDLK_AMPERSAND as isize,
    Quote = sdl2::sys::SDL_KeyCode::SDLK_QUOTE as isize,
    LeftParen = sdl2::sys::SDL_KeyCode::SDLK_LEFTPAREN as isize,
    RightParen = sdl2::sys::SDL_KeyCode::SDLK_RIGHTPAREN as isize,
    Asterisk = sdl2::sys::SDL_KeyCode::SDLK_ASTERISK as isize,
    Plus = sdl2::sys::SDL_KeyCode::SDLK_PLUS as isize,
    Comma = sdl2::sys::SDL_KeyCode::SDLK_COMMA as isize,
    Minus = sdl2::sys::SDL_KeyCode::SDLK_MINUS as isize,
    Period = sdl2::sys::SDL_KeyCode::SDLK_PERIOD as isize,
    Slash = sdl2::sys::SDL_KeyCode::SDLK_SLASH as isize,
    Num0 = sdl2::sys::SDL_KeyCode::SDLK_0 as isize,
    Num1 = sdl2::sys::SDL_KeyCode::SDLK_1 as isize,
    Num2 = sdl2::sys::SDL_KeyCode::SDLK_2 as isize,
    Num3 = sdl2::sys::SDL_KeyCode::SDLK_3 as isize,
    Num4 = sdl2::sys::SDL_KeyCode::SDLK_4 as isize,
    Num5 = sdl2::sys::SDL_KeyCode::SDLK_5 as isize,
    Num6 = sdl2::sys::SDL_KeyCode::SDLK_6 as isize,
    Num7 = sdl2::sys::SDL_KeyCode::SDLK_7 as isize,
    Num8 = sdl2::sys::SDL_KeyCode::SDLK_8 as isize,
    Num9 = sdl2::sys::SDL_KeyCode::SDLK_9 as isize,
    Colon = sdl2::sys::SDL_KeyCode::SDLK_COLON as isize,
    Semicolon = sdl2::sys::SDL_KeyCode::SDLK_SEMICOLON as isize,
    Less = sdl2::sys::SDL_KeyCode::SDLK_LESS as isize,
    Equals = sdl2::sys::SDL_KeyCode::SDLK_EQUALS as isize,
    Greater = sdl2::sys::SDL_KeyCode::SDLK_GREATER as isize,
    Question = sdl2::sys::SDL_KeyCode::SDLK_QUESTION as isize,
    At = sdl2::sys::SDL_KeyCode::SDLK_AT as isize,
    LeftBracket = sdl2::sys::SDL_KeyCode::SDLK_LEFTBRACKET as isize,
    Backslash = sdl2::sys::SDL_KeyCode::SDLK_BACKSLASH as isize,
    RightBracket = sdl2::sys::SDL_KeyCode::SDLK_RIGHTBRACKET as isize,
    Caret = sdl2::sys::SDL_KeyCode::SDLK_CARET as isize,
    Underscore = sdl2::sys::SDL_KeyCode::SDLK_UNDERSCORE as isize,
    Backquote = sdl2::sys::SDL_KeyCode::SDLK_BACKQUOTE as isize,
    A = sdl2::sys::SDL_KeyCode::SDLK_a as isize,
    B = sdl2::sys::SDL_KeyCode::SDLK_b as isize,
    C = sdl2::sys::SDL_KeyCode::SDLK_c as isize,
    D = sdl2::sys::SDL_KeyCode::SDLK_d as isize,
    E = sdl2::sys::SDL_KeyCode::SDLK_e as isize,
    F = sdl2::sys::SDL_KeyCode::SDLK_f as isize,
    G = sdl2::sys::SDL_KeyCode::SDLK_g as isize,
    H = sdl2::sys::SDL_KeyCode::SDLK_h as isize,
    I = sdl2::sys::SDL_KeyCode::SDLK_i as isize,
    J = sdl2::sys::SDL_KeyCode::SDLK_j as isize,
    K = sdl2::sys::SDL_KeyCode::SDLK_k as isize,
    L = sdl2::sys::SDL_KeyCode::SDLK_l as isize,
    M = sdl2::sys::SDL_KeyCode::SDLK_m as isize,
    N = sdl2::sys::SDL_KeyCode::SDLK_n as isize,
    O = sdl2::sys::SDL_KeyCode::SDLK_o as isize,
    P = sdl2::sys::SDL_KeyCode::SDLK_p as isize,
    Q = sdl2::sys::SDL_KeyCode::SDLK_q as isize,
    R = sdl2::sys::SDL_KeyCode::SDLK_r as isize,
    S = sdl2::sys::SDL_KeyCode::SDLK_s as isize,
    T = sdl2::sys::SDL_KeyCode::SDLK_t as isize,
    U = sdl2::sys::SDL_KeyCode::SDLK_u as isize,
    V = sdl2::sys::SDL_KeyCode::SDLK_v as isize,
    W = sdl2::sys::SDL_KeyCode::SDLK_w as isize,
    X = sdl2::sys::SDL_KeyCode::SDLK_x as isize,
    Y = sdl2::sys::SDL_KeyCode::SDLK_y as isize,
    Z = sdl2::sys::SDL_KeyCode::SDLK_z as isize,
    Delete = sdl2::sys::SDL_KeyCode::SDLK_DELETE as isize,
    CapsLock = sdl2::sys::SDL_KeyCode::SDLK_CAPSLOCK as isize,
    F1 = sdl2::sys::SDL_KeyCode::SDLK_F1 as isize,
    F2 = sdl2::sys::SDL_KeyCode::SDLK_F2 as isize,
    F3 = sdl2::sys::SDL_KeyCode::SDLK_F3 as isize,
    F4 = sdl2::sys::SDL_KeyCode::SDLK_F4 as isize,
    F5 = sdl2::sys::SDL_KeyCode::SDLK_F5 as isize,
    F6 = sdl2::sys::SDL_KeyCode::SDLK_F6 as isize,
    F7 = sdl2::sys::SDL_KeyCode::SDLK_F7 as isize,
    F8 = sdl2::sys::SDL_KeyCode::SDLK_F8 as isize,
    F9 = sdl2::sys::SDL_KeyCode::SDLK_F9 as isize,
    F10 = sdl2::sys::SDL_KeyCode::SDLK_F10 as isize,
    F11 = sdl2::sys::SDL_KeyCode::SDLK_F11 as isize,
    F12 = sdl2::sys::SDL_KeyCode::SDLK_F12 as isize,
    PrintScreen = sdl2::sys::SDL_KeyCode::SDLK_PRINTSCREEN as isize,
    ScrollLock = sdl2::sys::SDL_KeyCode::SDLK_SCROLLLOCK as isize,
    Pause = sdl2::sys::SDL_KeyCode::SDLK_PAUSE as isize,
    Insert = sdl2::sys::SDL_KeyCode::SDLK_INSERT as isize,
    Home = sdl2::sys::SDL_KeyCode::SDLK_HOME as isize,
    PageUp = sdl2::sys::SDL_KeyCode::SDLK_PAGEUP as isize,
    End = sdl2::sys::SDL_KeyCode::SDLK_END as isize,
    PageDown = sdl2::sys::SDL_KeyCode::SDLK_PAGEDOWN as isize,
    Right = sdl2::sys::SDL_KeyCode::SDLK_RIGHT as isize,
    Left = sdl2::sys::SDL_KeyCode::SDLK_LEFT as isize,
    Down = sdl2::sys::SDL_KeyCode::SDLK_DOWN as isize,
    Up = sdl2::sys::SDL_KeyCode::SDLK_UP as isize,
    NumLockClear = sdl2::sys::SDL_KeyCode::SDLK_NUMLOCKCLEAR as isize,
    KpDivide = sdl2::sys::SDL_KeyCode::SDLK_KP_DIVIDE as isize,
    KpMultiply = sdl2::sys::SDL_KeyCode::SDLK_KP_MULTIPLY as isize,
    KpMinus = sdl2::sys::SDL_KeyCode::SDLK_KP_MINUS as isize,
    KpPlus = sdl2::sys::SDL_KeyCode::SDLK_KP_PLUS as isize,
    KpEnter = sdl2::sys::SDL_KeyCode::SDLK_KP_ENTER as isize,
    Kp1 = sdl2::sys::SDL_KeyCode::SDLK_KP_1 as isize,
    Kp2 = sdl2::sys::SDL_KeyCode::SDLK_KP_2 as isize,
    Kp3 = sdl2::sys::SDL_KeyCode::SDLK_KP_3 as isize,
    Kp4 = sdl2::sys::SDL_KeyCode::SDLK_KP_4 as isize,
    Kp5 = sdl2::sys::SDL_KeyCode::SDLK_KP_5 as isize,
    Kp6 = sdl2::sys::SDL_KeyCode::SDLK_KP_6 as isize,
    Kp7 = sdl2::sys::SDL_KeyCode::SDLK_KP_7 as isize,
    Kp8 = sdl2::sys::SDL_KeyCode::SDLK_KP_8 as isize,
    Kp9 = sdl2::sys::SDL_KeyCode::SDLK_KP_9 as isize,
    Kp0 = sdl2::sys::SDL_KeyCode::SDLK_KP_0 as isize,
    KpPeriod = sdl2::sys::SDL_KeyCode::SDLK_KP_PERIOD as isize,
    Application = sdl2::sys::SDL_KeyCode::SDLK_APPLICATION as isize,
    Power = sdl2::sys::SDL_KeyCode::SDLK_POWER as isize,
    KpEquals = sdl2::sys::SDL_KeyCode::SDLK_KP_EQUALS as isize,
    F13 = sdl2::sys::SDL_KeyCode::SDLK_F13 as isize,
    F14 = sdl2::sys::SDL_KeyCode::SDLK_F14 as isize,
    F15 = sdl2::sys::SDL_KeyCode::SDLK_F15 as isize,
    F16 = sdl2::sys::SDL_KeyCode::SDLK_F16 as isize,
    F17 = sdl2::sys::SDL_KeyCode::SDLK_F17 as isize,
    F18 = sdl2::sys::SDL_KeyCode::SDLK_F18 as isize,
    F19 = sdl2::sys::SDL_KeyCode::SDLK_F19 as isize,
    F20 = sdl2::sys::SDL_KeyCode::SDLK_F20 as isize,
    F21 = sdl2::sys::SDL_KeyCode::SDLK_F21 as isize,
    F22 = sdl2::sys::SDL_KeyCode::SDLK_F22 as isize,
    F23 = sdl2::sys::SDL_KeyCode::SDLK_F23 as isize,
    F24 = sdl2::sys::SDL_KeyCode::SDLK_F24 as isize,
    Execute = sdl2::sys::SDL_KeyCode::SDLK_EXECUTE as isize,
    Help = sdl2::sys::SDL_KeyCode::SDLK_HELP as isize,
    Menu = sdl2::sys::SDL_KeyCode::SDLK_MENU as isize,
    Select = sdl2::sys::SDL_KeyCode::SDLK_SELECT as isize,
    Stop = sdl2::sys::SDL_KeyCode::SDLK_STOP as isize,
    Again = sdl2::sys::SDL_KeyCode::SDLK_AGAIN as isize,
    Undo = sdl2::sys::SDL_KeyCode::SDLK_UNDO as isize,
    Cut = sdl2::sys::SDL_KeyCode::SDLK_CUT as isize,
    Copy = sdl2::sys::SDL_KeyCode::SDLK_COPY as isize,
    Paste = sdl2::sys::SDL_KeyCode::SDLK_PASTE as isize,
    Find = sdl2::sys::SDL_KeyCode::SDLK_FIND as isize,
    Mute = sdl2::sys::SDL_KeyCode::SDLK_MUTE as isize,
    VolumeUp = sdl2::sys::SDL_KeyCode::SDLK_VOLUMEUP as isize,
    VolumeDown = sdl2::sys::SDL_KeyCode::SDLK_VOLUMEDOWN as isize,
    KpComma = sdl2::sys::SDL_KeyCode::SDLK_KP_COMMA as isize,
    KpEqualsAS400 = sdl2::sys::SDL_KeyCode::SDLK_KP_EQUALSAS400 as isize,
    AltErase = sdl2::sys::SDL_KeyCode::SDLK_ALTERASE as isize,
    Sysreq = sdl2::sys::SDL_KeyCode::SDLK_SYSREQ as isize,
    Cancel = sdl2::sys::SDL_KeyCode::SDLK_CANCEL as isize,
    Clear = sdl2::sys::SDL_KeyCode::SDLK_CLEAR as isize,
    Prior = sdl2::sys::SDL_KeyCode::SDLK_PRIOR as isize,
    Return2 = sdl2::sys::SDL_KeyCode::SDLK_RETURN2 as isize,
    Separator = sdl2::sys::SDL_KeyCode::SDLK_SEPARATOR as isize,
    Out = sdl2::sys::SDL_KeyCode::SDLK_OUT as isize,
    Oper = sdl2::sys::SDL_KeyCode::SDLK_OPER as isize,
    ClearAgain = sdl2::sys::SDL_KeyCode::SDLK_CLEARAGAIN as isize,
    CrSel = sdl2::sys::SDL_KeyCode::SDLK_CRSEL as isize,
    ExSel = sdl2::sys::SDL_KeyCode::SDLK_EXSEL as isize,
    Kp00 = sdl2::sys::SDL_KeyCode::SDLK_KP_00 as isize,
    Kp000 = sdl2::sys::SDL_KeyCode::SDLK_KP_000 as isize,
    ThousandsSeparator = sdl2::sys::SDL_KeyCode::SDLK_THOUSANDSSEPARATOR as isize,
    DecimalSeparator = sdl2::sys::SDL_KeyCode::SDLK_DECIMALSEPARATOR as isize,
    CurrencyUnit = sdl2::sys::SDL_KeyCode::SDLK_CURRENCYUNIT as isize,
    CurrencySubUnit = sdl2::sys::SDL_KeyCode::SDLK_CURRENCYSUBUNIT as isize,
    KpLeftParen = sdl2::sys::SDL_KeyCode::SDLK_KP_LEFTPAREN as isize,
    KpRightParen = sdl2::sys::SDL_KeyCode::SDLK_KP_RIGHTPAREN as isize,
    KpLeftBrace = sdl2::sys::SDL_KeyCode::SDLK_KP_LEFTBRACE as isize,
    KpRightBrace = sdl2::sys::SDL_KeyCode::SDLK_KP_RIGHTBRACE as isize,
    KpTab = sdl2::sys::SDL_KeyCode::SDLK_KP_TAB as isize,
    KpBackspace = sdl2::sys::SDL_KeyCode::SDLK_KP_BACKSPACE as isize,
    KpA = sdl2::sys::SDL_KeyCode::SDLK_KP_A as isize,
    KpB = sdl2::sys::SDL_KeyCode::SDLK_KP_B as isize,
    KpC = sdl2::sys::SDL_KeyCode::SDLK_KP_C as isize,
    KpD = sdl2::sys::SDL_KeyCode::SDLK_KP_D as isize,
    KpE = sdl2::sys::SDL_KeyCode::SDLK_KP_E as isize,
    KpF = sdl2::sys::SDL_KeyCode::SDLK_KP_F as isize,
    KpXor = sdl2::sys::SDL_KeyCode::SDLK_KP_XOR as isize,
    KpPower = sdl2::sys::SDL_KeyCode::SDLK_KP_POWER as isize,
    KpPercent = sdl2::sys::SDL_KeyCode::SDLK_KP_PERCENT as isize,
    KpLess = sdl2::sys::SDL_KeyCode::SDLK_KP_LESS as isize,
    KpGreater = sdl2::sys::SDL_KeyCode::SDLK_KP_GREATER as isize,
    KpAmpersand = sdl2::sys::SDL_KeyCode::SDLK_KP_AMPERSAND as isize,
    KpDblAmpersand = sdl2::sys::SDL_KeyCode::SDLK_KP_DBLAMPERSAND as isize,
    KpVerticalBar = sdl2::sys::SDL_KeyCode::SDLK_KP_VERTICALBAR as isize,
    KpDblVerticalBar = sdl2::sys::SDL_KeyCode::SDLK_KP_DBLVERTICALBAR as isize,
    KpColon = sdl2::sys::SDL_KeyCode::SDLK_KP_COLON as isize,
    KpHash = sdl2::sys::SDL_KeyCode::SDLK_KP_HASH as isize,
    KpSpace = sdl2::sys::SDL_KeyCode::SDLK_KP_SPACE as isize,
    KpAt = sdl2::sys::SDL_KeyCode::SDLK_KP_AT as isize,
    KpExclam = sdl2::sys::SDL_KeyCode::SDLK_KP_EXCLAM as isize,
    KpMemStore = sdl2::sys::SDL_KeyCode::SDLK_KP_MEMSTORE as isize,
    KpMemRecall = sdl2::sys::SDL_KeyCode::SDLK_KP_MEMRECALL as isize,
    KpMemClear = sdl2::sys::SDL_KeyCode::SDLK_KP_MEMCLEAR as isize,
    KpMemAdd = sdl2::sys::SDL_KeyCode::SDLK_KP_MEMADD as isize,
    KpMemSubtract = sdl2::sys::SDL_KeyCode::SDLK_KP_MEMSUBTRACT as isize,
    KpMemMultiply = sdl2::sys::SDL_KeyCode::SDLK_KP_MEMMULTIPLY as isize,
    KpMemDivide = sdl2::sys::SDL_KeyCode::SDLK_KP_MEMDIVIDE as isize,
    KpPlusMinus = sdl2::sys::SDL_KeyCode::SDLK_KP_PLUSMINUS as isize,
    KpClear = sdl2::sys::SDL_KeyCode::SDLK_KP_CLEAR as isize,
    KpClearEntry = sdl2::sys::SDL_KeyCode::SDLK_KP_CLEARENTRY as isize,
    KpBinary = sdl2::sys::SDL_KeyCode::SDLK_KP_BINARY as isize,
    KpOctal = sdl2::sys::SDL_KeyCode::SDLK_KP_OCTAL as isize,
    KpDecimal = sdl2::sys::SDL_KeyCode::SDLK_KP_DECIMAL as isize,
    KpHexadecimal = sdl2::sys::SDL_KeyCode::SDLK_KP_HEXADECIMAL as isize,
    LCtrl = sdl2::sys::SDL_KeyCode::SDLK_LCTRL as isize,
    LShift = sdl2::sys::SDL_KeyCode::SDLK_LSHIFT as isize,
    LAlt = sdl2::sys::SDL_KeyCode::SDLK_LALT as isize,
    LGui = sdl2::sys::SDL_KeyCode::SDLK_LGUI as isize,
    RCtrl = sdl2::sys::SDL_KeyCode::SDLK_RCTRL as isize,
    RShift = sdl2::sys::SDL_KeyCode::SDLK_RSHIFT as isize,
    RAlt = sdl2::sys::SDL_KeyCode::SDLK_RALT as isize,
    RGui = sdl2::sys::SDL_KeyCode::SDLK_RGUI as isize,
    Mode = sdl2::sys::SDL_KeyCode::SDLK_MODE as isize,
    AudioNext = sdl2::sys::SDL_KeyCode::SDLK_AUDIONEXT as isize,
    AudioPrev = sdl2::sys::SDL_KeyCode::SDLK_AUDIOPREV as isize,
    AudioStop = sdl2::sys::SDL_KeyCode::SDLK_AUDIOSTOP as isize,
    AudioPlay = sdl2::sys::SDL_KeyCode::SDLK_AUDIOPLAY as isize,
    AudioMute = sdl2::sys::SDL_KeyCode::SDLK_AUDIOMUTE as isize,
    MediaSelect = sdl2::sys::SDL_KeyCode::SDLK_MEDIASELECT as isize,
    Www = sdl2::sys::SDL_KeyCode::SDLK_WWW as isize,
    Mail = sdl2::sys::SDL_KeyCode::SDLK_MAIL as isize,
    Calculator = sdl2::sys::SDL_KeyCode::SDLK_CALCULATOR as isize,
    Computer = sdl2::sys::SDL_KeyCode::SDLK_COMPUTER as isize,
    AcSearch = sdl2::sys::SDL_KeyCode::SDLK_AC_SEARCH as isize,
    AcHome = sdl2::sys::SDL_KeyCode::SDLK_AC_HOME as isize,
    AcBack = sdl2::sys::SDL_KeyCode::SDLK_AC_BACK as isize,
    AcForward = sdl2::sys::SDL_KeyCode::SDLK_AC_FORWARD as isize,
    AcStop = sdl2::sys::SDL_KeyCode::SDLK_AC_STOP as isize,
    AcRefresh = sdl2::sys::SDL_KeyCode::SDLK_AC_REFRESH as isize,
    AcBookmarks = sdl2::sys::SDL_KeyCode::SDLK_AC_BOOKMARKS as isize,
    BrightnessDown = sdl2::sys::SDL_KeyCode::SDLK_BRIGHTNESSDOWN as isize,
    BrightnessUp = sdl2::sys::SDL_KeyCode::SDLK_BRIGHTNESSUP as isize,
    DisplaySwitch = sdl2::sys::SDL_KeyCode::SDLK_DISPLAYSWITCH as isize,
    KbdIllumToggle = sdl2::sys::SDL_KeyCode::SDLK_KBDILLUMTOGGLE as isize,
    KbdIllumDown = sdl2::sys::SDL_KeyCode::SDLK_KBDILLUMDOWN as isize,
    KbdIllumUp = sdl2::sys::SDL_KeyCode::SDLK_KBDILLUMUP as isize,
    Eject = sdl2::sys::SDL_KeyCode::SDLK_EJECT as isize,
    Sleep = sdl2::sys::SDL_KeyCode::SDLK_SLEEP as isize,
}
