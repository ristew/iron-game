use ggez::{Context, graphics::{self, Color, DrawMode, DrawParam, FillOptions, Mesh, Rect, StrokeOptions, draw, present}};

use crate::*;

pub fn render_province(province: &Province, world: &World, ctx: &mut Context) {
    let w = SQRT_3 * TILE_SIZE_X;
    let h = 2.0 * TILE_SIZE_Y;
    let hex = Mesh::new_polygon(ctx, DrawMode::Fill(FillOptions::DEFAULT),
                                &[[w / 2.0, 0.0], [w, h / 4.0], [w, 3.0 * h / 4.0], [w / 2.0, h], [0.0, 3.0 * h / 4.0], [0.0, h / 4.0]]
                                , Color::GREEN).unwrap();
    let hex_outline = Mesh::new_polygon(ctx, DrawMode::Stroke(StrokeOptions::DEFAULT),
                                &[[w / 2.0, 0.0], [w, h / 4.0], [w, 3.0 * h / 4.0], [w / 2.0, h], [0.0, 3.0 * h / 4.0], [0.0, h / 4.0]]
                                , Color::BLACK).unwrap();
    let province_pixel_pos = province.coordinate.pixel_pos(&world.camera);
    draw(ctx, &hex, DrawParam::new().dest([province_pixel_pos.x, province_pixel_pos.y])).unwrap();
    draw(ctx, &hex_outline, DrawParam::new().dest([province_pixel_pos.x, province_pixel_pos.y])).unwrap();
}

pub fn render_world(world: &mut World, ctx: &mut Context) {
    for province in world.provinces.rcs.iter().map(|rc| rc.borrow()) {
        render_province(&province, world, ctx);
    }
}

pub fn render_ui(world: &World, ctx: &mut Context) {
    let window_size = graphics::size(ctx);
    let ui_width = window_size.0 / 3.5;
    let ui_height = window_size.1;

    let rect = Mesh::new_rectangle(ctx, DrawMode::Fill(Default::default()), Rect::new(0.0, 0.0, ui_width, ui_height), Color::new(0.9, 0.8, 0.6, 0.9)).unwrap();
    draw(ctx, &rect, DrawParam::default()).unwrap();
}

#[derive(Debug, Clone)]
pub struct Camera {
    pub p: Point2,
    pub zoom: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            p: Point2::new(0.0, 0.0),
            zoom: 1.0,
        }
    }
}

impl Camera {
    pub fn translate(&self, point: Point2) -> Point2 {
        Point2::new((point.x + self.p.x) / self.zoom, (point.y + self.p.y) / self.zoom)
    }
}
