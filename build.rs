use std::fmt::format;
use contour_tracing::image::single_l8_to_paths;
use image::{DynamicImage, Luma, Pixel};

use itertools::Itertools;
use std::fs;
use std::fs::{DirEntry, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use texture_packer::{TexturePacker, TexturePackerConfig};
use texture_packer::exporter::ImageExporter;
use texture_packer::texture::Texture;
include!("./src/assets/geometry.rs");

fn main() {
    // Calling `build_info_build::build_script` collects all data and makes it available to `build_info::build_info!`
    // and `build_info::format!` in the main program.
    build_info_build::build_script();

    embed_resource::compile("icon.rc", embed_resource::NONE);

    build_assets().unwrap();
}

const IMAGE_DIR: &str = "./src/assets/images";
const SPRITES_JSON: &str = "./src/assets/sprites.json";
const SPRITES_PNG: &str = "./src/assets/sprites.png";
const SOUND_CREATE_DIR: &str = "./src/assets/sound/create";
const SOUND_CREATE_MOD: &str = "./src/assets/sound/create/mod.rs";
const SOUND_DESTROY_DIR: &str = "./src/assets/sound/destroy";
const SOUND_DESTROY_MOD: &str = "./src/assets/sound/destroy/mod.rs";
const SOUND_EXPLOSION_DIR: &str = "./src/assets/sound/explosion";
const SOUND_EXPLOSION_MOD: &str = "./src/assets/sound/explosion/mod.rs";
const MUSIC_DIR: &str = "./src/assets/sound/music";
const MUSIC_MOD: &str = "./src/assets/sound/music/mod.rs";

struct Asset {
    name: String,
    path: PathBuf
}

impl Asset {
    fn new(entry: DirEntry) -> Self {
        Self { path: entry.path(), name: asset_name(&entry) }
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

fn build_assets() -> Result<(), String> {
    if Path::new(SPRITES_JSON).exists()
        && Path::new(SPRITES_PNG).exists()
        && Path::new(SOUND_CREATE_MOD).exists()
        && Path::new(SOUND_DESTROY_MOD).exists()
        && Path::new(SOUND_EXPLOSION_MOD).exists()
        && Path::new(MUSIC_MOD).exists(){
        return Ok(());
    }

    let create_sound_files = load_sound_assets(SOUND_CREATE_DIR)?;

    let mut packer = TexturePacker::new_skyline(
        TexturePackerConfig {
            // these are max sd2 texture sizes
            max_width: 16384,
            max_height: 16384,
            allow_rotation: false,
            texture_outlines: false,
            border_padding: 0,
            texture_padding: 0,
            texture_extrusion: 0,
            force_max_dimensions: false,
            trim: false,
        }
    );
    let mut assets = vec![];
    for dir_entry in fs::read_dir(IMAGE_DIR).map_err(|e| e.to_string())? {
        let dir_entry = dir_entry.map_err(|e| e.to_string())?;
        if dir_entry.path().extension().and_then(|s| s.to_str()) != Some("png") {
            continue;
        }
        let name = asset_name(&dir_entry);

        if create_sound_files.iter().find(|e| e.name == name).is_none() {
            return Err(format!("no create sound present for sprite: {}", name))
        }

        let texture = image::open(&dir_entry.path()).map_err(|e| e.to_string())?;
        packer.pack_own(name.clone(), texture).unwrap();

        let frame = packer.get_frame(&name).unwrap().frame;

        let (triangles, unit_scale) = triangulate(&dir_entry.path())?;
        let snip = SpriteRect::new(frame.x as f64, frame.y as f64, frame.w as f64, frame.h as f64);
        let asset = SpriteAsset::new(name.clone(), snip, triangles, unit_scale);
        assets.push(asset);
    }

    let exporter = ImageExporter::export(&packer).unwrap();
    let mut file = File::create(SPRITES_PNG).unwrap();
    exporter
        .write_to(&mut file, image::ImageFormat::Png)
        .unwrap();

    let meta_file = File::create(SPRITES_JSON).unwrap();
    let sprite_sheet = SpriteAssetSheet::new(assets);
    serde_json::to_writer(meta_file, &sprite_sheet).map_err(|e| e.to_string())?;

    named_asset_mod_file(SOUND_CREATE_MOD, create_sound_files)?;

    simple_asset_mod_file(
        SOUND_DESTROY_MOD,
        load_sound_assets(SOUND_DESTROY_DIR)?
    )?;

    simple_asset_mod_file(
        SOUND_EXPLOSION_MOD,
        load_sound_assets(SOUND_EXPLOSION_DIR)?
    )?;

    simple_asset_mod_file(
        MUSIC_MOD,
        load_sound_assets(MUSIC_DIR)?
    )
}

fn triangulate(input_path: &Path) -> Result<(Vec<SpriteTriangle>, f64), String> {
    let mut img = image::open(input_path)
        .map_err(|e| e.to_string())?
        .into_luma_alpha8();

    // 1. create silhouette of image
    for pixel in img.pixels_mut() {
        if pixel[1] > 0 {
            // if not transparent then set to black
            pixel[0] = 0;
        } else {
            // otherwise set to white
            pixel[0] = 255;
        }
    }
    let mut luma = DynamicImage::ImageLumaA8(img).into_luma8();

    // 2. trace a path around the silhouette
    let path = single_l8_to_paths(&mut luma, Luma([0]), true);
    let (_, outline_path) = svg_path_parser::parse(&path)
        .max_by(|(_, p1), (_, p2)| p1.len().cmp(&p2.len()))
        .unwrap();

    // 3. simplify the path, this produces far fewer triangles
    let simplify_points = outline_path
        .into_iter()
        .map(|(x, y)| simplify_polyline::Point { vec: [x, y] })
        .collect::<Vec<simplify_polyline::Point<2, f64>>>();
    let points = simplify_polyline::simplify(&simplify_points, 1.0, true)
        .into_iter()
        .map(|p| (p.vec[0] as u32, p.vec[1] as u32))
        .unique() // remove integer duplicates
        .map(|(x, y)| SpritePoint::new(x as f64, y as f64))
        .collect::<Vec<SpritePoint>>();

    // 4. scale to 0.0 -> 1.0
    let aabb = aabb(&points).unwrap();
    let unit_scale = 1.0 / aabb.width().max(aabb.height());

    // 4. triangulate: convert path into a set of triangles
    let mut polygon = poly2tri::Polygon::new();
    for p in points.into_iter() {
        polygon.add_point(p.x(), p.y());
    }
    let cdt = poly2tri::CDT::new(polygon);
    let triangulate = cdt.triangulate();

    let color_img = image::open(input_path)
        .map_err(|e| e.to_string())?
        .into_rgba8();
    let mut triangles: Vec<SpriteTriangle> = vec![];
    for idx in 0 .. triangulate.size() {
        let points = triangulate
            .get_triangle(idx)
            .points
            .map(|p| SpritePoint::new(p[0], p[1]));

        let mut rs = 0.0;
        let mut gs = 0.0;
        let mut bs = 0.0;
        let mut count = 0.0;
        let mut raw_triangle = SpriteTriangle::new(points);
        for point in raw_triangle.interior_points().into_iter() {
            let [r, g, b] = color_img.get_pixel(point.x() as u32, point.y() as u32).to_rgb().0;
            rs += r as f64;
            gs += g as f64;
            bs += b as f64;
            count += 1.0;
        }
        let r = (rs / count).round() as u8;
        let g = (gs / count).round() as u8;
        let b = (bs / count).round() as u8;

        let scaled_points = points
            .map(|p| SpritePoint::new((p.x() - aabb.x()) * unit_scale, (p.y() - aabb.y()) * unit_scale));
        let mut triangle = SpriteTriangle::new(scaled_points);
        triangle.set_color(r, g, b);
        triangles.push(triangle);
    }
    Ok((triangles, unit_scale))
}

fn aabb(points: &[SpritePoint]) -> Option<SpriteRect> {
    let mut xs = vec![];
    let mut ys = vec![];
    for p in points.iter() {
        xs.push(p.x());
        ys.push(p.y());
    }
    Some(SpriteRect::from_p1_p2(
        *xs.iter().min_by(|&&a, &b| a.partial_cmp(b).unwrap())?,
        *ys.iter().min_by(|&&a, &b| a.partial_cmp(b).unwrap())?,
        *xs.iter().max_by(|&&a, &b| a.partial_cmp(b).unwrap())?,
        *ys.iter().max_by(|&&a, &b| a.partial_cmp(b).unwrap())?,
    ))
}

fn asset_name(dir_entry: &DirEntry) -> String {
    dir_entry.path().with_extension("").file_name().unwrap().to_str().unwrap().to_string()
}

fn load_sound_assets(dir: &str) -> Result<Vec<Asset>, String> {
    let mut result = vec![];
    for dir_entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let dir_entry = dir_entry.map_err(|e| e.to_string())?;
        if dir_entry.path().extension().and_then(|s| s.to_str()) != Some("ogg") {
            continue;
        }
        result.push(Asset::new(dir_entry));
    }
    Ok(result)
}

fn named_asset_mod_file(mod_name: &str, assets: Vec<Asset>) -> Result<(), String> {
    let sorted = assets.into_iter()
        .sorted_by(|a, b| a.name.cmp(&b.name))
        .collect::<Vec<Asset>>();
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

fn simple_asset_mod_file(mod_name: &str, assets: Vec<Asset>) -> Result<(), String> {
    let sorted = assets.into_iter()
        .sorted_by(|a, b| a.name.cmp(&b.name))
        .collect::<Vec<Asset>>();
    let includes = sorted.iter()
        .map(|a| format!("    {},", a.include_bytes())).join("\n");
    let rs  = format!(
        "pub const ASSETS: [&[u8]; {}] = [\n{}\n];",
        sorted.len(),
        includes
    );
    write_file(mod_name, &rs)
}

fn write_file(path: &str, content: &str) -> Result<(), String> {
    let path = Path::new(path);
    File::create(path).map_err(|e| e.to_string())
        ?.write_all(content.as_bytes()).map_err(|e| e.to_string())
}
