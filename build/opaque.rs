use std::fs;
use std::fs::{DirEntry, File};
use std::path::{Path, PathBuf};
use itertools::Itertools;
use std::io::Write;

pub struct OpaqueAsset {
    name: String,
    path: PathBuf
}

impl OpaqueAsset {
    fn new(entry: DirEntry) -> Self {
        let name = entry.path().with_extension("").file_name().unwrap().to_str().unwrap().to_string();
        Self { path: entry.path(), name }
    }

    fn include_name(&self) -> String {
        self.name.to_ascii_uppercase()
    }

    fn include_bytes(&self) -> String {
        format!("include_bytes!(\"{}\")", self.path.file_name().unwrap().to_str().unwrap())
    }

    fn assign_include_bytes(&self) -> String {
        format!("const {}: &[u8] = {};", self.include_name(), self.include_bytes())
    }

    fn match_clause(&self) -> String {
        format!("        \"{}\" => {},", self.name, self.include_name())
    }
}



pub fn load_assets<P: AsRef<Path>>(dir: P, extension: &str) -> Result<Vec<OpaqueAsset>, String> {
    let result = fs::read_dir(dir).map_err(|e| e.to_string())
        ?.map(|dir_entry| dir_entry.unwrap())
        .filter(|dir_entry| dir_entry.path().extension().and_then(|s| s.to_str()) == Some(extension))
        .map(|dir_entry| OpaqueAsset::new(dir_entry))
        .collect();
    Ok(result)
}

pub fn named_asset_mod_file<P: AsRef<Path>>(mod_name: P, assets: Vec<OpaqueAsset>) -> Result<(), String> {
    let sorted = assets.into_iter()
        .sorted_by(|a, b| a.name.cmp(&b.name))
        .collect::<Vec<OpaqueAsset>>();
    let match_clauses = sorted.iter()
        .map(|a| a.match_clause())
        .join("\n");
    let includes = sorted.iter()
        .map(|a| a.assign_include_bytes()).join("\n");

    let rs  = format!(
        "{}\n\npub fn asset(name: &str) -> &'static [u8] {{\n    match name {{\n{}\n        _ => panic!(\"no such asset {{}}\", name)\n    }}\n}}",
        includes, match_clauses
    );

    write_file(mod_name, &rs)
}

pub fn simple_asset_mod_file<P: AsRef<Path>>(mod_name: P, assets: Vec<OpaqueAsset>) -> Result<(), String> {
    let sorted = assets.into_iter()
        .sorted_by(|a, b| a.name.cmp(&b.name))
        .collect::<Vec<OpaqueAsset>>();
    let includes = sorted.iter()
        .map(|a| format!("    {},", a.include_bytes())).join("\n");
    let rs  = format!(
        "pub const ASSETS: [&[u8]; {}] = [\n{}\n];",
        sorted.len(),
        includes
    );
    write_file(mod_name, &rs)
}

fn write_file<P: AsRef<Path>>(path: P, content: &str) -> Result<(), String> {
    File::create(path).map_err(|e| e.to_string())
        ?.write_all(content.as_bytes()).map_err(|e| e.to_string())
}
