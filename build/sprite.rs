use std::path::PathBuf;
use image::{DynamicImage, ImageError, imageops, RgbaImage, SubImage};
use imageproc::contours;
use imageproc::contours::{BorderType, Contour};
use crate::geometry::SpriteSnip;

#[derive(Clone)]
pub struct SpriteImage {
    path: PathBuf,
    name: String,
    rgba_img: RgbaImage
}

impl SpriteImage {
    pub fn new(path: PathBuf) -> Result<Self, ImageError> {
        let rgba_img =  image::open(&path)?.clone().into_rgba8();
        let name = path.with_extension("").file_name().unwrap().to_str().unwrap().to_string();

        Ok(Self { name, rgba_img, path })
    }

    pub fn crop(&self, snip: &ContourSpriteSnip) -> SubImage<&RgbaImage> {
        imageops::crop_imm(&self.rgba_img, snip.x(), snip.y(), snip.width(), snip.height())
    }

    pub fn contours(&self) -> Vec<Contour<u32>> {
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


    pub fn path(&self) -> &PathBuf {
        &self.path
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn rgba_img(&self) -> &RgbaImage {
        &self.rgba_img
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct ContourSpriteSnip {
    contour: Vec<imageproc::point::Point<u32>>,
    snip: SpriteSnip
}

impl ContourSpriteSnip {
    pub fn new(contour: Contour<u32>) -> Self {
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

    pub fn contour(&self) -> &Vec<imageproc::point::Point<u32>> {
        &self.contour
    }
    pub fn snip(&self) -> SpriteSnip {
        self.snip
    }
}