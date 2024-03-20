use image::Pixel;
use crate::geometry::{SpritePoint, SpriteTriangle};
use crate::sprite::{ContourPoints, SpriteImage};

const PATH_TOLERANCE: f64 = 5.0;

pub fn triangulate(contour_points: &ContourPoints, source_image: &SpriteImage) -> Result<Vec<SpriteTriangle>, String> {

    // 1. simplify the contour, this produces far fewer triangles
    let points = simplify_polyline::simplify(
            &contour_points
                .into_iter()
                .map(|p| simplify_polyline::Point { vec: [p.x as f64, p.y as f64] })
                .collect::<Vec<simplify_polyline::Point<2, f64>>>(),
            PATH_TOLERANCE,
            true
        )
        .into_iter()
        .map(|p| SpritePoint::new(p.vec[0], p.vec[1]))
        .collect::<Vec<SpritePoint>>();

    // 2. scale to 0.0 -> 1.0
    let aabb = aabb(&points);
    let unit_scale = 1.0 / aabb.width.max(aabb.height);

    // 3. triangulate: convert path into a set of triangles
    let mut polygon = poly2tri::Polygon::new();
    for p in points.into_iter() {
        polygon.add_point(p.x, p.y);
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
        for (x, y) in IntTriangle::from_points(&points).interior_points().into_iter() {
            if x >= source_image.rgba_img().width() || y >= source_image.rgba_img().height() {
                continue;
            }
            let [r, g, b] = source_image.rgba_img().get_pixel(x, y).to_rgb().0;
            rs += r as f64;
            gs += g as f64;
            bs += b as f64;
            count += 1.0;
        }
        let r = (rs / count).round() as u8;
        let g = (gs / count).round() as u8;
        let b = (bs / count).round() as u8;

        let scaled_points = points
            .map(|p| SpritePoint::new((p.x - aabb.x) * unit_scale, (p.y - aabb.y) * unit_scale));
        let triangle = SpriteTriangle::new(scaled_points, [r, g, b]);
        triangles.push(triangle);
    }
    Ok(triangles)
}

fn aabb(points: &[SpritePoint]) -> RectF64 {
    let mut xs = vec![];
    let mut ys = vec![];
    for p in points.iter() {
        xs.push(p.x);
        ys.push(p.y);
    }
    RectF64::from_p1_p2(
        *xs.iter().min_by(|&&a, &b| a.partial_cmp(b).unwrap()).unwrap(),
        *ys.iter().min_by(|&&a, &b| a.partial_cmp(b).unwrap()).unwrap(),
        *xs.iter().max_by(|&&a, &b| a.partial_cmp(b).unwrap()).unwrap(),
        *ys.iter().max_by(|&&a, &b| a.partial_cmp(b).unwrap()).unwrap(),
    )
}

struct IntTriangle {
    points: [(u32, u32); 3]
}

impl IntTriangle {
    fn from_points(points: &[SpritePoint; 3]) -> Self {
        Self { points: points.map(|p| (p.x.round() as u32, p.y.round() as u32)) }
    }

    fn interior_points(&self) -> Vec<(u32, u32)> {
        let xs = self.points.map(|(x, _)| x);
        let ys = self.points.map(|(_, y)| y);
        let x0 = *xs.iter().min().unwrap();
        let y0 = *ys.iter().min().unwrap();
        let x1 = *xs.iter().max().unwrap();
        let y1 = *ys.iter().max().unwrap();

        let mut result = vec![];
        for x in x0 ..= x1 {
            for y in y0 ..= y1 {
                if self.contains_point(x, y) {
                    result.push((x, y));
                }
            }
        }
        result
    }

    fn contains_point(&self, x: u32, y: u32) -> bool {
        let p = SpritePoint::new(x as f64, y as f64);
        let [p0, p1, p2] = self.points.map(|(x, y)| SpritePoint::new(x as f64, y as f64));
        let a = 0.5 * (-p1.y * p2.x + p0.y * (-p1.x + p2.x) + p0.x * (p1.y - p2.y) + p1.x * p2.y);
        let sign = if a < 0.0 { -1.0 } else { 1.0 };
        let s = (p0.y * p2.x - p0.x * p2.y + (p2.y - p0.y) * p.x + (p0.x - p2.x) * p.y) * sign;
        let t = (p0.x * p1.y - p0.y * p1.x + (p0.y - p1.y) * p.x + (p1.x - p0.x) * p.y) * sign;
        s > 0.0 && t > 0.0 && (s + t) < 2.0 * a * sign
    }
}

struct RectF64 {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

impl RectF64 {
    fn from_p1_p2(x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        Self {
            x: x1,
            y: y1,
            width: x2 - x1,
            height: y2 - y1,
        }
    }
}