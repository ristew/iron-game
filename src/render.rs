use std::collections::{HashMap, HashSet};

use ggez::{
    graphics::{
        self, draw, present, BlendMode, Color, DrawMode, DrawParam, FillOptions, Mesh, MeshBatch,
        Rect, StrokeOptions, Transform,
    },
    mint::ColumnMatrix4,
    Context,
};

use crate::{
    IronData, IronId, Point2, Province, ProvinceId, World, SQRT_3, TILE_SIZE_X, TILE_SIZE_Y,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OverlayKind {
    Population,
}

pub trait Overlay {
    fn new(ctx: &mut Context) -> Self
    where
        Self: Sized;
    fn kind(&self) -> OverlayKind;
    fn update(&mut self, world: &World);
    fn map(&mut self) -> &mut MeshBatch;
    fn render(&mut self, transform: ColumnMatrix4<f32>, ctx: &mut Context) {
        self.map().set_blend_mode(Some(BlendMode::Alpha));
        self.map()
            .draw(ctx, DrawParam::new().transform(transform))
            .unwrap();
    }
}

struct PopOverlay {
    map: MeshBatch,
}

impl Overlay for PopOverlay {
    fn new(ctx: &mut Context) -> Self
    where
        Self: Sized,
    {
        let hex = hex_mesh(ctx, Color::new(1.0, 1.0, 1.0, 1.0));
        Self {
            map: MeshBatch::new(hex).unwrap(),
        }
    }
    fn update(&mut self, world: &World) {
        self.map.clear();
        let mut province_pops: HashMap<ProvinceId, isize> = HashMap::new();
        let (w, h) = tile_sizes();
        let mut max_pop = 0;
        for province in world.storages.get_storage::<Province>().rcs.iter() {
            let pop = province.borrow().population(world);
            max_pop = max_pop.max(pop);
            province_pops.insert(province.borrow().id(), pop);
        }
        for (province_id, population) in province_pops.iter() {
            // println!("add hex to overlay map");
            let province_pixel_pos = province_id.get(world).borrow().coordinate.base_pixel_pos();
            let hex_dest = [
                province_pixel_pos.x - w / 2.0,
                province_pixel_pos.y - h / 2.0,
            ];
            let map_pct = 0.2 + 0.8 * *population as f32 / max_pop as f32;
            self.map.add(
                DrawParam::new()
                    .dest(hex_dest)
                    .color(Color::new(map_pct, 0.0, 0.0, 0.7)),
            );
        }
    }

    fn map(&mut self) -> &mut MeshBatch {
        &mut self.map
    }

    fn kind(&self) -> OverlayKind {
        OverlayKind::Population
    }
}

pub struct RenderContext {
    province_meshes: HashSet<ProvinceId>,
    mesh_map: MeshBatch,
    outline_map: MeshBatch,
    pub overlay: Option<Box<dyn Overlay>>,
}

fn tile_sizes() -> (f32, f32) {
    let w = SQRT_3 * TILE_SIZE_X;
    let h = 2.0 * TILE_SIZE_Y;
    (w, h)
}

fn hex_mesh(ctx: &mut Context, color: Color) -> Mesh {
    let (w, h) = tile_sizes();
    Mesh::new_polygon(
        ctx,
        DrawMode::Fill(FillOptions::DEFAULT),
        &[
            [w / 2.0, 0.0],
            [w, h / 4.0],
            [w, 3.0 * h / 4.0],
            [w / 2.0, h],
            [0.0, 3.0 * h / 4.0],
            [0.0, h / 4.0],
        ],
        color,
    )
    .unwrap()
}

impl RenderContext {
    pub fn new(ctx: &mut Context) -> Self {
        let (w, h) = tile_sizes();
        let hex_outline = Mesh::new_polygon(
            ctx,
            DrawMode::Stroke(StrokeOptions::default().with_line_width(2.0)),
            &[
                [w / 2.0, 0.0],
                [w, h / 4.0],
                [w, 3.0 * h / 4.0],
                [w / 2.0, h],
                [0.0, 3.0 * h / 4.0],
                [0.0, h / 4.0],
            ],
            Color::BLACK,
        )
        .unwrap();
        let hex = hex_mesh(ctx, Color::WHITE);
        let mesh_map = MeshBatch::new(hex).unwrap();
        let outline_map = MeshBatch::new(hex_outline).unwrap();
        Self {
            province_meshes: HashSet::new(),
            mesh_map,
            outline_map,
            overlay: None,
        }
    }

    pub fn toggle_overlay(&mut self, ctx: &mut Context, kind: OverlayKind) {
        // println!("toggle overlay {:?}", kind);
        if let Some(overlay) = &self.overlay {
            if overlay.kind() == kind {
                self.overlay = None;
                return;
            }
        }

        self.overlay = Some(Box::new(match kind {
            OverlayKind::Population => PopOverlay::new(ctx),
        }));
    }

    pub fn generate_province_meshes(&mut self, world: &World, ctx: &mut Context) {
        for province in world
            .storages
            .get_storage::<Province>()
            .rcs
            .iter()
            .map(|rc| rc.borrow())
        {
            let mesh_key = province.id();
            if !self.province_meshes.contains(&mesh_key) {
                self.generate_province_mesh(&province, world, ctx);
            }
        }
    }
    fn generate_province_mesh(&mut self, province: &Province, world: &World, ctx: &mut Context) {
        let (w, h) = tile_sizes();
        let province_pixel_pos = province.coordinate.base_pixel_pos();
        let hex_dest = [
            province_pixel_pos.x - w / 2.0,
            province_pixel_pos.y - h / 2.0,
        ];
        self.mesh_map.add(
            DrawParam::new()
                .dest(hex_dest)
                .color(province.terrain.color()),
        );
        self.outline_map.add(DrawParam::new().dest(hex_dest));
        self.province_meshes.insert(province.id());
    }

    fn camera_matrix(&self, world: &World) -> ColumnMatrix4<f32> {
        self.camera_matrix_dest(world, [0.0, 0.0])
    }

    fn camera_matrix_dest<T>(&self, world: &World, dest: T) -> ColumnMatrix4<f32>
    where
        T: Into<ggez::mint::Point2<f32>>,
    {
        let dp = dest.into();
        let transform = Transform::Values {
            dest: [dp.x / world.camera.zoom, dp.y / world.camera.zoom].into(),
            rotation: 0.0,
            scale: [1.0 / world.camera.zoom, 1.0 / world.camera.zoom].into(),
            // scale: [world.camera.zoom, world.camera.zoom].into(),
            offset: [-world.camera.p.x, -world.camera.p.y].into(),
        };
        transform.to_bare_matrix()
    }

    pub fn render_world(&mut self, world: &mut World, ctx: &mut Context) {
        let transform = self.camera_matrix(world);
        self.mesh_map
            .draw(ctx, DrawParam::new().transform(transform))
            .unwrap();
        if let Some(overlay) = self.overlay.as_mut() {
            overlay.render(transform, ctx);
        }
        self.outline_map
            .draw(ctx, DrawParam::new().transform(transform))
            .unwrap();
        if let Some(province_id) = &world.selected_province {
            let (w, h) = tile_sizes();
            let selected_hex = hex_mesh(ctx, Color::new(0.0, 0.0, 0.0, 0.2));
            let province_pixel_pos = province_id.get(world).borrow().coordinate.base_pixel_pos();
            let hex_dest = [
                province_pixel_pos.x - w / 2.0,
                province_pixel_pos.y - h / 2.0,
            ];
            let camera_matrix = self.camera_matrix_dest(world, hex_dest);
            draw(
                ctx,
                &selected_hex,
                DrawParam::new().transform(camera_matrix),
            )
            .unwrap();
        }
    }

    pub fn render_ui(&mut self, world: &World, ctx: &mut Context) {
        let window_size = graphics::size(ctx);
        let ui_width = window_size.0 / 3.5;
        let ui_height = window_size.1;

        let rect = Mesh::new_rectangle(
            ctx,
            DrawMode::Fill(Default::default()),
            Rect::new(0.0, 0.0, ui_width, ui_height),
            Color::new(0.9, 0.8, 0.6, 0.9),
        )
        .unwrap();
        draw(ctx, &rect, DrawParam::default()).unwrap();
    }
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
        Point2::new(
            (point.x + self.p.x) / self.zoom,
            (point.y + self.p.y) / self.zoom,
        )
    }

    pub fn reverse_translate(&self, point: Point2) -> Point2 {
        Point2::new(
            self.zoom * point.x - self.p.x,
            self.zoom * point.y - self.p.y,
        )
    }
}
