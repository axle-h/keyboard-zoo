use std::fs::DirEntry;
use std::path::PathBuf;

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
