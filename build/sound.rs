use std::path::{Path, PathBuf};
use crate::opaque::{load_assets, simple_asset_mod_file};

const SOUND_DESTROY_DIR: &str =  "sound/destroy/";
const SOUND_EXPLOSION_DIR: &str =  "sound/explosion/";
const MUSIC_DIR: &str =  "sound/music/";

pub fn build_sound_modules<P : AsRef<Path>>(root_dir: P) -> Result<(), String> {
    fn load_sound_assets(dir: PathBuf) -> Result<(), String> {
        let assets = load_assets(&dir, "ogg")?;
        simple_asset_mod_file(dir.join("mod.rs"), assets)
    }

    load_sound_assets(root_dir.as_ref().join(SOUND_DESTROY_DIR))?;
    load_sound_assets(root_dir.as_ref().join(SOUND_EXPLOSION_DIR))?;
    load_sound_assets(root_dir.as_ref().join(MUSIC_DIR))
}