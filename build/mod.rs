use crate::geometry::SpritePoint;

mod opaque;
mod geometry;

fn main() {
    build_info_build::build_script();

    embed_resource::compile("icon.rc", embed_resource::NONE);
    let point = SpritePoint::new(0.0, 0.0);
    println!("{}", point.x())
}