use image::{DynamicImage, GenericImage, GrayImage, ImageError, ImageFormat, imageops, Pixel, RgbaImage, SubImage};

use itertools::Itertools;
use std::fs;
use std::fs::{DirEntry, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use const_format::concatcp;
use imageproc::contours;
use imageproc::contours::{BorderType, Contour};
use rayon::prelude::*;
use texture_packer::{TexturePacker, TexturePackerConfig};
use texture_packer::exporter::ImageExporter;
use texture_packer::texture::Texture;
include!("./src/assets/geometry.rs");

const ROOT_DIR: &str = "./src/assets/";
const FONT_IMAGES_DIR: &str = concatcp!(ROOT_DIR, "font_images");

const SPRITES_JSON: &str =  concatcp!(ROOT_DIR, "sprites.json");
const SPRITES_PNG: &str =  concatcp!(ROOT_DIR, "sprites.png");
const SOUND_DESTROY_DIR: &str =  concatcp!(ROOT_DIR, "sound/destroy/");
const SOUND_DESTROY_MOD: &str =  concatcp!(SOUND_DESTROY_DIR, "mod.rs");
const SOUND_EXPLOSION_DIR: &str =  concatcp!(ROOT_DIR, "sound/explosion/");
const SOUND_EXPLOSION_MOD: &str =  concatcp!(SOUND_EXPLOSION_DIR, "mod.rs");
const MUSIC_DIR: &str =  concatcp!(ROOT_DIR, "sound/music/");
const MUSIC_MOD: &str =  concatcp!(MUSIC_DIR, "mod.rs");

fn main() {
    build_info_build::build_script();

    embed_resource::compile("icon.rc", embed_resource::NONE);

    build_assets().unwrap();
}

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

#[derive(Clone)]
struct SpriteImage {
    name: String,
    rgba_img: RgbaImage
}

impl SpriteImage {
    fn new(path: &Path) -> Result<Self, ImageError> {
        let rgba_img =  image::open(path)?.clone().into_rgba8();
        let name = path.with_extension("").file_name().unwrap().to_str().unwrap().to_string();

        Ok(Self { name, rgba_img })
    }

    fn crop(&self, snip: &ContourSpriteSnip) -> SubImage<&RgbaImage> {
        imageops::crop_imm(&self.rgba_img, snip.x(), snip.y(), snip.width(), snip.height())
    }

    fn contours(&self) -> Vec<Contour<u32>> {
        let mut luma_img = DynamicImage::ImageRgba8(self.rgba_img.clone()).into_luma_alpha8();

        // 1. create silhouette of image
        for pixel in luma_img.pixels_mut() {
            if pixel[1] > 0 {
                // if not transparent then set to white
                pixel[0] = 255;
            } else {
                // otherwise set to black
                pixel[0] = 0;
            }
        }
        let silhouette_img = DynamicImage::ImageLumaA8(luma_img).into_luma8();

        contours::find_contours::<u32>(&silhouette_img)
            .into_iter()
            .filter(|c| c.border_type == BorderType::Outer)
            .collect()
    }
}

fn build_assets() -> Result<(), String> {
    if Path::new(SPRITES_JSON).exists()
        && Path::new(SPRITES_PNG).exists()
        && Path::new(SOUND_DESTROY_MOD).exists()
        && Path::new(SOUND_EXPLOSION_MOD).exists()
        && Path::new(MUSIC_MOD).exists(){
        return Ok(());
    }

    // async process all the images
    struct PartialAsset {
        name: String,
        sprite_image: RgbaImage,
        character: char,
        triangles: Vec<SpriteTriangle>,
        unit_scale: f64
    }
    let partial_assets = fs::read_dir(FONT_IMAGES_DIR).map_err(|e| e.to_string())?.into_iter()
        .map(|dir_entry| dir_entry.unwrap().path())
        .filter(|path| path.extension().and_then(|s| s.to_str()) == Some("png"))
        .collect::<Vec<PathBuf>>()
        .into_par_iter()
        .map(|path| SpriteImage::new(path.as_path()).expect("cannot read image"))
        .flat_map(|image|
              extract_chars(&image).expect("cannot get characters").into_iter().map(move |char_snip| {
                let name = format!("{}:{}", char_snip.character, image.name);
                let (triangles, unit_scale) = triangulate(&char_snip.snip, &image).expect("cannot triangulate");
                let sprite_image = image.crop(&char_snip.snip).to_image();
                PartialAsset { name, sprite_image, character: char_snip.character, triangles, unit_scale }
            }).collect::<Vec<PartialAsset>>()
        ).collect::<Vec<PartialAsset>>();

    // pack the sprites in sync
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

    for asset in partial_assets.into_iter() {
        packer.pack_own(asset.name.clone(), asset.sprite_image.clone()).expect("too many character sprites");
        let frame = packer.get_frame(&asset.name).unwrap().frame;
        assets.push((SpriteSnip::new(frame.x, frame.y, frame.w, frame.h), asset));
    }

    // not using texture_packer export because it's hella slow, it's actually pixel-wise to consider rotations, flips and texture implementations
    let mut sprite_sheet = RgbaImage::new(packer.width(), packer.height());
    for (snip, asset) in assets.iter() {
        sprite_sheet.copy_from(&asset.sprite_image, snip.x(), snip.y()).map_err(|e| e.to_string())?;
    }
    let mut file = File::create(SPRITES_PNG).map_err(|s| s.to_string())?;
    sprite_sheet.write_to(&mut file, ImageFormat::Png).map_err(|s| s.to_string())?;

    let meta_file = File::create(SPRITES_JSON).map_err(|s| s.to_string())?;
    let sprite_sheet = SpriteAssetSheet::new(
        assets.into_iter().map(|(snip, asset)|
            SpriteAsset::new(
                asset.name,
                asset.character,
                snip,
                asset.triangles,
                asset.unit_scale
            )
        ).collect()
    );
    serde_json::to_writer(meta_file, &sprite_sheet).map_err(|e| e.to_string())?;

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

struct CharSpriteSnip {
    character: char,
    snip: ContourSpriteSnip
}

#[derive(Clone, PartialEq, Eq)]
struct ContourSpriteSnip {
    contour: Vec<imageproc::point::Point<u32>>,
    snip: SpriteSnip
}

impl ContourSpriteSnip {
    fn new(contour: Contour<u32>) -> Self {
        let xs = contour.points.iter().map(|p| p.x).collect::<Vec<u32>>();
        let ys = contour.points.iter().map(|p| p.y).collect::<Vec<u32>>();

        let x1 = *xs.iter().min().unwrap();
        let y1 = *ys.iter().min().unwrap();
        let x2 = *xs.iter().max().unwrap();
        let y2 = *ys.iter().max().unwrap();

        let snip = SpriteSnip::from_corners((x1, y1), (x2, y2));
        Self { snip, contour: contour.points }
    }

    pub fn x(&self) -> u32 {
        self.snip.x()
    }
    pub fn y(&self) -> u32 {
        self.snip.y()
    }
    pub fn width(&self) -> u32 {
        self.snip.width()
    }
    pub fn height(&self) -> u32 {
        self.snip.height()
    }
}

fn extract_chars(image: &SpriteImage) -> Result<Vec<CharSpriteSnip>, String> {
    // most of the "Outer" contours will be the paths around the character sprites
    let snips = image.contours().into_iter()
        .map(|c| ContourSpriteSnip::new(c))
        .collect::<Vec<ContourSpriteSnip>>();

    // remove snips that are completely within another snip, these will be floating sub geometry
    let mut snips = snips.iter()
        .cloned()
        .filter(|snip| snips.iter().find(|&other| other != snip && other.snip.contains(&snip.snip)).is_none())
        .sorted_by(|s1, s2| (s1.x() + s1.y()).cmp(&(s2.x() + s2.y())))
        .collect::<Vec<ContourSpriteSnip>>();

    // get a 1d vertical slice through the image to find the rows NOTE the image must have regular rows
    let mut y1d = vec![false; image.rgba_img.height() as usize];
    for snip in snips.iter() {
        for i in snip.y()..(snip.y() + snip.height()) {
            y1d[i as usize] = true;
        }
    }

    // find edges in the 1d slice to find row boundaries
    let mut row_bounds = vec![];
    let mut current_start: Option<u32> = None;
    for (i, current) in y1d.into_iter().enumerate() {
        if current && current_start.is_none() {
            current_start = Some(i as u32);
        } else if !current && current_start.is_some() {
            row_bounds.push((current_start.unwrap(), i as u32));
            current_start = None;
        }
    }

    // columns are then simply ordered over x per row
    let snips_by_row = snips.into_iter().into_group_map_by(|snip| {
        let (row, _) = row_bounds.iter().enumerate()
            .find(|(i, (y1, y2))| snip.y() >= *y1 && snip.y() <= *y2)
            .expect(&format!("bad row col on image {} at {},{}", image.name, snip.snip.x(), snip.snip.y()));
        row
    });

    // snips can now be placed into row-col order, which implies alphabet order
    let mut results = vec![];
    let mut character = 'a';
    for (_, row_snips) in snips_by_row.into_iter().sorted_by(|(g1, _), (g2, _)| g1.cmp(&g2)) {
        for (_, snip) in row_snips.into_iter().sorted_by(|s1, s2| s1.x().cmp(&s2.x())).enumerate() {
            results.push(CharSpriteSnip { character, snip });
            character = std::char::from_u32(character as u32 + 1).unwrap();
        }
    }

    assert_eq!(results.len(), 26, "{} should have 26 characters but it has {}", image.name, results.len());
    Ok(results)
}

fn triangulate(sprite: &ContourSpriteSnip, source_image: &SpriteImage) -> Result<(Vec<SpriteTriangle>, f64), String> {

    // 1. simplify the contour, this produces far fewer triangles, TODO check this
    let simplify_points = sprite.contour
        .iter()
        .map(|p| simplify_polyline::Point { vec: [p.x as f64, p.y as f64] })
        .collect::<Vec<simplify_polyline::Point<2, f64>>>();
    let points = simplify_polyline::simplify(&simplify_points, 1.0, true)
        .into_iter()
        .map(|p| (p.vec[0] as u32, p.vec[1] as u32))
        .unique() // remove integer duplicates
        .map(|(x, y)| SpritePoint::new(x as f64, y as f64))
        .collect::<Vec<SpritePoint>>();

    // 2. scale to 0.0 -> 1.0
    let aabb = aabb(&points).unwrap();
    let unit_scale = 1.0 / aabb.width().max(aabb.height());

    // 3. triangulate: convert path into a set of triangles
    let mut polygon = poly2tri::Polygon::new();
    for p in points.into_iter() {
        polygon.add_point(p.x(), p.y());
    }
    let cdt = poly2tri::CDT::new(polygon);
    let triangulate = cdt.triangulate();

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
            let [r, g, b] = source_image.rgba_img.get_pixel(point.x() as u32, point.y() as u32).to_rgb().0;
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
