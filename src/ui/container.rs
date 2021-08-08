use ggez::{Context, graphics::{self, Color, DrawMode, DrawParam, Drawable, Mesh, Rect, Text, draw}};

use crate::*;

pub type ContainerChildren = Vec<Box<dyn Container>>;

pub trait Container {
    fn size(&self) -> Point2;
    fn render(&self, ctx: &mut Context, dest: Point2);
    fn layout(&mut self, ctx: &mut Context, constraints: Constraints, world: &World);
}

pub struct BaseUiContainer {
    pub children: ContainerChildren,
    pub padding: Point2,
    pub layout_size: Point2,
    pub background_color: Color,
    pub constraints: Constraints,
}

impl Container for BaseUiContainer {
    fn render(&self, ctx: &mut Context, dest: Point2) {
        if self.layout_size.zero() {
            return;
        }
        let rect = Mesh::new_rectangle(
            ctx,
            DrawMode::Fill(Default::default()),
            Rect::new(dest.x, dest.y, self.layout_size.x, self.layout_size.y),
            self.background_color).unwrap();
        draw(ctx, &rect, DrawParam::default()).unwrap();
        let mut base_dest = dest;
        for child in self.children.iter() {
            child.render(ctx, base_dest + self.padding);
            base_dest.y += child.size().y;
        }
    }

    fn size(&self) -> Point2 {
        self.layout_size
    }

    fn layout(&mut self, ctx: &mut Context, parent_constraints: Constraints, world: &World) {
        let constraints = self.constraints.reconcile(parent_constraints);
        // println!("constraints: {:?}", constraints);
        self.layout_size = Point2::new(constraints.min_width, constraints.min_height);
        for child in self.children.iter_mut() {
            child.layout(ctx, constraints, world);
            let child_size = child.size();
            self.layout_size.y += child_size.y;
            if child_size.x > self.layout_size.x {
                self.layout_size.x = child_size.x.max(constraints.max_width);
            }
        }
    }
}

impl BaseUiContainer {
    pub fn add_child(&mut self, child: Box<dyn Container>) -> &mut Self {
        self.children.push(child);
        self
    }

    pub fn clear(&mut self) -> &mut Self {
        self.children.clear();
        self
    }

    pub fn new(padding: Point2, background_color: Color, constraints: Constraints) -> Self {
        Self {
            children: Vec::new(),
            padding,
            layout_size: Point2::new(0.0, 0.0),
            background_color,
            constraints,
        }
    }
}

pub struct TextContainer {
    padding: Point2,
    layout_size: Point2,
    text: Text,
}

impl Container for TextContainer {
    fn size(&self) -> Point2 {
        self.layout_size
    }

    fn render(&self, ctx: &mut Context, dest: Point2) {
        self.text.draw(
            ctx,
            DrawParam::default().dest(dest + self.padding)
        ).unwrap();
    }

    fn layout(&mut self, ctx: &mut Context, constraints: Constraints, _world: &World) {
        self.text.set_bounds(Point2::new(constraints.max_width, f32::INFINITY), graphics::Align::Left);
        let computed_dimensions = self.text.dimensions(ctx);
        self.layout_size = Point2::new(computed_dimensions.w, computed_dimensions.h);
    }
}

impl TextContainer {
    pub fn new(text: &str, padding: Point2) -> Self {
        Self {
            padding,
            layout_size: Point2::default(),
            text: Text::new(text),
        }
    }

    pub fn empty() -> Self {
        Self {
            padding: Point2::default(),
            layout_size: Point2::default(),
            text: Text::new(""),
        }
    }
}

// Launch a war targeting $self.target_province.name for the glory of Jebkarbo!
// macro matches $elf
            // if child_size.x > self_size.x || child_size.y > self_size.y {
            //     self_size = Point2::new(
            //         self_size.x.max(child_size.x).max(constraints.max_width),
            //         self_size.y.max(child_size.y).max(constraints.max_height),
            //     ));
            // }

pub struct DateContainer(pub TextContainer);

impl Container for DateContainer {
    fn size(&self) -> Point2 {
        self.0.size()
    }

    fn render(&self, ctx: &mut Context, dest: Point2) {
        self.0.render(ctx, dest)
    }

    fn layout(&mut self, ctx: &mut Context, constraints: Constraints, world: &World) {
        self.0.text = Text::new(format!("{:?}", world.date));
        self.0.layout(ctx, constraints, world)
    }
}

pub struct ProvinceInfoContainer {
    pub province: ProvinceId,
    pub mapping: Box::<dyn Fn(Rc<RefCell<Province>>) -> String>,
    pub inner: TextContainer,
}

impl Container for ProvinceInfoContainer {
    fn size(&self) -> Point2 {
        self.inner.size()
    }

    fn render(&self, ctx: &mut Context, dest: Point2) {
        self.inner.render(ctx, dest)
    }

    fn layout(&mut self, ctx: &mut Context, constraints: Constraints, world: &World) {
        self.inner.text = Text::new((*self.mapping)(self.province.get(world)));
        self.inner.layout(ctx, constraints, world)
    }
}
