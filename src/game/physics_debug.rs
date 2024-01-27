use std::cell::RefCell;
use std::rc::Rc;
use box2d_rs::b2_draw::{B2color, B2draw, B2drawShapeFlags, B2drawTrait};
use box2d_rs::b2_math::{B2Transform, B2vec2};
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::pixels::Color;
use sdl2::render::WindowCanvas;
use crate::game::scale::PhysicsScale;

pub struct SdlPhysicsDraw {
    base: B2draw,
    canvas: Rc<RefCell<WindowCanvas>>,
    scale: PhysicsScale
}

impl<'a> SdlPhysicsDraw {
    pub fn new(canvas: Rc<RefCell<WindowCanvas>>, scale: PhysicsScale) -> Self {
        let mut base: B2draw = Default::default();
        base.set_flags(B2drawShapeFlags::all());
        Self { canvas, base, scale }
    }

    fn to_gfx_vertices(&self, vertices: &[B2vec2]) -> (Vec<i16>, Vec<i16>) {
        let mut vx = vec![];
        let mut vy = vec![];
        for v in vertices.into_iter() {
            let (x, y) = self.to_gfx_point(*v);
            vx.push(x);
            vy.push(y);
        }
        (vx, vy)
    }

    fn to_gfx_scalar(&self, value: f32) -> i16 {
        self.scale.b2d_to_sdl(value) as i16
    }

    fn to_gfx_point(&self, v: B2vec2) -> (i16, i16) {
        (self.to_gfx_scalar(v.x), self.to_gfx_scalar(v.y))
    }
}

fn b2_rgb(component: f32) -> u8 {
    (component * 255.0).round() as u8
}

trait ToSdlColor {
    fn to_sdl(&self) -> Color;
}

impl ToSdlColor for B2color {
    fn to_sdl(&self) -> Color {
        Color::RGBA(b2_rgb(self.r), b2_rgb(self.g), b2_rgb(self.b), b2_rgb(self.a))
    }
}

impl B2drawTrait for SdlPhysicsDraw {
    fn get_base(&self) -> &B2draw {
        &self.base
    }

    fn get_base_mut(&mut self) -> &mut B2draw {
        &mut self.base
    }

    fn draw_polygon(&mut self, vertices: &[B2vec2], color: B2color) {
        let canvas = self.canvas.borrow_mut();
        let (vx, vy) = self.to_gfx_vertices(vertices);
        canvas.polygon(&vx, &vy, color.to_sdl()).unwrap();
    }

    fn draw_solid_polygon(&mut self, vertices: &[B2vec2], color: B2color) {
        let canvas = self.canvas.borrow_mut();
        let (vx, vy) = self.to_gfx_vertices(vertices);
        canvas.filled_polygon(&vx, &vy, color.to_sdl()).unwrap();
    }

    fn draw_circle(&mut self, center: B2vec2, radius: f32, color: B2color) {
        let canvas = self.canvas.borrow_mut();
        let (x, y) = self.to_gfx_point(center);
        canvas.circle(x, y, self.to_gfx_scalar(radius), color.to_sdl()).unwrap();
    }

    fn draw_solid_circle(&mut self, center: B2vec2, radius: f32, axis: B2vec2, color: B2color) {
        let canvas = self.canvas.borrow_mut();
        let (x, y) = self.to_gfx_point(center);
        canvas.filled_circle(x, y, self.to_gfx_scalar(radius), color.to_sdl()).unwrap();
    }

    fn draw_segment(&mut self, p1: B2vec2, p2: B2vec2, color: B2color) {

    }

    fn draw_transform(&mut self, xf: B2Transform) {

    }

    fn draw_point(&mut self, p: B2vec2, size: f32, color: B2color) {

    }
}
