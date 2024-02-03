use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{BufWriter, Cursor, Write};
use std::ops::RangeInclusive;
use std::path::{Path, PathBuf};
use image::{GenericImage, ImageFormat, RgbaImage};
use imagequant::RGBA;
use itertools::Itertools;
use png::ScaledFloat;
use rayon::prelude::*;
use texture_packer::{TexturePacker, TexturePackerConfig};
use texture_packer::texture::Texture;
use crate::geometry::{SpriteAsset, SpriteAssetSheet, SpriteSnip, SpriteTriangle};
use crate::sprite::{ContourSpriteSnip, SpriteImage};
use crate::triangulate::triangulate;

const LETTERS_SRC: &str = "letters";
const NUMBERS_SRC: &str = "numbers";
const META_FILE: &str =  "sprites.json";
const PNG_FILE: &str =  "sprites.png";
const OPTIMIZED_PNG_FILE: &str =  "sprites.min.png";


pub fn build_sprite_sheet<P : AsRef<Path>>(root_path: P) -> Result<(), String> {
    let meta_path = root_path.as_ref().join(META_FILE);
    let packed_sprites_path = root_path.as_ref().join(PNG_FILE);

    if meta_path.exists() && packed_sprites_path.exists() {
        return Ok(());
    }

    let mut assets = async_load_all_sprites(&root_path, SpriteType::Letters);
    let mut numbers = async_load_all_sprites(&root_path, SpriteType::Numbers);
    assets.append(&mut numbers);

    // pack the sprites in sync
    let (packed_sprites, meta) = pack_sprites(assets)?;

    // save the sprite sheet
    save_quantized_png(packed_sprites, packed_sprites_path)?;

    // save meta
    let meta_file = File::create(meta_path).map_err(|s| s.to_string())?;
    serde_json::to_writer(meta_file, &meta).map_err(|e| e.to_string())?;

    Ok(())
}

fn save_quantized_png(src: RgbaImage, path: PathBuf) -> Result<(), String> {
    let mut liq = imagequant::new();
    liq.set_speed(5).map_err(|e| e.to_string())?;
    liq.set_quality(70, 99).map_err(|e| e.to_string())?;

    let mut img = liq.new_image(
        src.pixels().map(|p| RGBA::from(p.0)).collect::<Vec<RGBA>>(),
        src.width() as usize,
        src.height() as usize,
        0.0
    ).map_err(|e| e.to_string())?;
    let mut res = liq.quantize(&mut img).map_err(|e| e.to_string())?;
    let (palette, pixels) = res.remapped(&mut img).unwrap();

    let file = File::create(path).map_err(|e| e.to_string())?;
    let mut encoder = png::Encoder::new(BufWriter::new(file), src.width(), src.height());
    encoder.set_trns(palette.iter().map(|p| p.a).collect::<Vec<u8>>());
    encoder.set_palette(palette.into_iter().flat_map(|p| [p.r, p.g, p.b]).collect::<Vec<u8>>());
    encoder.set_color(png::ColorType::Indexed);
    //encoder.set_source_gamma(ScaledFloat::new(res.output_gamma() as f32));
    encoder.set_depth(png::BitDepth::Eight);

    encoder.write_header().map_err(|e| e.to_string())?
        .write_image_data(&pixels).map_err(|e| e.to_string())
}

#[derive(Copy, Clone)]
enum SpriteType { Letters, Numbers }

impl SpriteType {
    fn src_path(&self) -> &str {
        match self {
            SpriteType::Letters => LETTERS_SRC,
            SpriteType::Numbers => NUMBERS_SRC
        }
    }

    fn expected_size(&self) -> usize {
        match self {
            SpriteType::Letters => 26,
            SpriteType::Numbers => 10
        }
    }

    fn chars(&self) -> Vec<char> {
        match self {
            SpriteType::Letters => ('a' ..= 'z').collect(),
            SpriteType::Numbers => ('1' ..= '9').chain(['0']).collect()
        }
    }
}

fn async_load_all_sprites<P : AsRef<Path>>(root_path: P, sprite_type: SpriteType) -> Vec<PartialAsset> {
    fs::read_dir(root_path.as_ref().join(sprite_type.src_path()))
        .expect("cannot read font source path")
        .into_iter()
        .map(|dir_entry| dir_entry.unwrap().path())
        .filter(|path| path.extension().and_then(|s| s.to_str()) == Some("png"))
        .collect::<Vec<PathBuf>>()
        .into_par_iter()
        .map(|path| SpriteImage::new(path).expect("cannot read image"))
        .flat_map(|image|
            extract_chars(&image, sprite_type).expect("cannot get characters").into_iter().map(move |(character, snip)| {
                let name = format!("{}:{}", character, image.name());
                let (triangles, unit_scale) = triangulate(&snip, &image).expect("cannot triangulate");
                let sprite_image = image.crop(&snip).to_image();
                PartialAsset { name, sprite_image, character, triangles, unit_scale }
            }).collect::<Vec<PartialAsset>>()
        ).collect()
}

fn pack_sprites(partial_assets: Vec<PartialAsset>) -> Result<(RgbaImage, SpriteAssetSheet), String> {
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
    let mut packed_sprites = RgbaImage::new(packer.width(), packer.height());
    for (snip, asset) in assets.iter() {
        packed_sprites.copy_from(&asset.sprite_image, snip.x(), snip.y()).map_err(|e| e.to_string())?;
    }

    let meta = SpriteAssetSheet::new(
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

    Ok((packed_sprites, meta))
}

fn extract_chars(image: &SpriteImage, sprite_type: SpriteType) -> Result<HashMap<char, ContourSpriteSnip>, String> {
    // most of the "Outer" contours will be the paths around the character sprites
    let snips = image.contours().into_iter()
        .map(|c| ContourSpriteSnip::new(c))
        .collect::<Vec<ContourSpriteSnip>>();

    // remove snips that are completely within another snip, these will be floating sub geometry
    let mut snips = snips.iter()
        .cloned()
        .filter(|snip| snips.iter().find(|&other| other != snip && other.snip().contains(&snip.snip())).is_none())
        .sorted_by(|s1, s2| (s1.x() + s1.y()).cmp(&(s2.x() + s2.y())))
        .collect::<Vec<ContourSpriteSnip>>();

    // get a 1d vertical slice through the image to find the rows NOTE the image must have regular rows
    let mut y1d = vec![false; image.rgba_img().height() as usize];
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
            .expect(&format!("bad row col on image {} at {},{}", image.name(), snip.snip().x(), snip.snip().y()));
        row
    });

    // snips can now be placed into row-col order, which implies alphabet order
    let mut results = vec![];
    let mut char_iter = sprite_type.chars().into_iter();
    for (_, row_snips) in snips_by_row.into_iter().sorted_by(|(g1, _), (g2, _)| g1.cmp(&g2)) {
        for (_, snip) in row_snips.into_iter().sorted_by(|s1, s2| s1.x().cmp(&s2.x())).enumerate() {
            let character = char_iter.next().unwrap_or('_');
            results.push((character, snip));
        }
    }

    if results.len() > sprite_type.expected_size() {
        // dump images for debug and panic
        let dump_path = image.path().parent().unwrap().join(format!("{}-debug", image.name()));
        fs::create_dir_all(&dump_path).map_err(|e| e.to_string())?;
        let results_len = results.len();
        for (idx, (_, snip)) in results.into_iter().enumerate() {
            let snip_path = dump_path.join(format!("{}.png", idx));
            let mut snip_file = File::create(snip_path).map_err(|s| s.to_string())?;
            image.crop(&snip).to_image().write_to(&mut snip_file, ImageFormat::Png).map_err(|s| s.to_string())?;
        }
        panic!("{} should have {} characters but it has {}, snips dumped to {}",
               image.name(), sprite_type.expected_size(), results_len, dump_path.to_str().unwrap());
    }

    Ok(results.into_iter().collect())
}

struct PartialAsset {
    name: String,
    sprite_image: RgbaImage,
    character: char,
    triangles: Vec<SpriteTriangle>,
    unit_scale: f64
}


