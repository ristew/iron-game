use ggez::{Context, graphics::{Color, DrawMode, DrawParam, FillOptions, Mesh, Rect, StrokeOptions, draw, present}, mint::Point2};

use crate::{game::Province, world::World};

pub fn render_province(province: &Province, world: &World, ctx: &mut Context) {
    let hex = Mesh::new_polygon(ctx, DrawMode::Stroke(StrokeOptions::DEFAULT),
                                &[[16.0, 0.0], [32.0, 8.0], [32.0, 24.0], [16.0, 32.0], [0.0, 24.0], [0.0, 8.0]]
                                , Color::GREEN).unwrap();
    let province_pixel_pos = province.coordinate.pixel_pos();
    println!("ppp {:?}", province_pixel_pos);
    draw(ctx, &hex, DrawParam::new().dest([province_pixel_pos.0, province_pixel_pos.1])).unwrap();
}

pub fn render_world(world: &World, ctx: &mut Context) {
    for province in world.provinces.rcs.iter().map(|rc| rc.borrow()) {
        render_province(&province, world, ctx);
    }
    present(ctx).unwrap();
}
