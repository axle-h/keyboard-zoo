use image::Pixel;
use itertools::Itertools;
use crate::geometry::{SpritePoint, SpriteRect, SpriteTriangle};
use crate::sprite::{ContourSpriteSnip, SpriteImage};

pub fn triangulate(sprite: &ContourSpriteSnip, source_image: &SpriteImage) -> Result<(Vec<SpriteTriangle>, f64), String> {

    // 1. simplify the contour, this produces far fewer triangles, TODO check this
    let simplify_points = sprite.contour()
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
            let [r, g, b] = source_image.rgba_img().get_pixel(point.x() as u32, point.y() as u32).to_rgb().0;
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
