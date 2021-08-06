use std::rc::Rc;

use ggez::{Context, graphics::{self, Color, DrawMode, DrawParam, Drawable, Mesh, Rect, Text, draw}};

use crate::Point2;

#[derive(Copy, Clone, Debug, Default)]
pub struct Constraints {
    min_width: f32,
    min_height: f32,
    max_width: f32,
    max_height: f32,
}

impl Constraints {
    pub fn new(min_width: f32, min_height: f32, max_width: f32, max_height: f32) -> Self {
        Self {
            min_width,
            min_height,
            max_width,
            max_height,
        }
    }

    pub fn reconcile(&self, other: Constraints) -> Self {
        // println!("self: {:?}", self);
        // println!("other: {:?}", other);
        Self {
            min_width: self.min_width.max(other.min_width),
            min_height: self.min_height.max(other.min_height),
            max_width: self.max_width.min(other.max_width),
            max_height: self.max_height.min(other.max_height),
        }
    }
}

pub type ContainerChildren = Vec<Box<dyn Container>>;

pub trait Container {
    fn size(&self) -> Point2;
    fn render(&self, ctx: &mut Context, dest: Point2);
    fn layout(&mut self, ctx: &mut Context, constraints: Constraints);
}

pub struct BaseUiContainer {
    children: ContainerChildren,
    padding: Point2,
    layout_size: Point2,
    background_color: Color,
    constraints: Constraints,
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

    fn layout(&mut self, ctx: &mut Context, parent_constraints: Constraints) {
        let constraints = self.constraints.reconcile(parent_constraints);
        println!("constraints: {:?}", constraints);
        self.layout_size = Point2::new(constraints.min_width, constraints.min_height);
        for child in self.children.iter_mut() {
            child.layout(ctx, constraints);
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

    fn layout(&mut self, ctx: &mut Context, constraints: Constraints) {
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
}

            // if child_size.x > self_size.x || child_size.y > self_size.y {
            //     self_size = Point2::new(
            //         self_size.x.max(child_size.x).max(constraints.max_width),
            //         self_size.y.max(child_size.y).max(constraints.max_height),
            //     ));
            // }

pub struct UiSystem {
    root_node: BaseUiContainer,
}
impl UiSystem {
    pub fn run(&mut self, ctx: &mut Context) {
        let window_size = graphics::size(ctx);
        self.root_node.layout(ctx, Constraints {
            min_width: 0.0,
            min_height: 0.0,
            max_width: window_size.0,
            max_height: window_size.1,
        });
        self.root_node.render(ctx, Point2::new(0.0, 0.0));
    }

    pub fn init(&mut self, ctx: &Context) {
        let text_child = TextContainer::new("test layout", Point2::new(1.0, 1.0));
        let text_child_2 = TextContainer::new("test layout dos", Point2::new(1.0, 1.0));
        let window_size = graphics::size(ctx);
        let window_w = window_size.0 / 3.5;
        let window_h = window_size.1;
        let mut info_panel = BaseUiContainer::new(Point2::new(5.0, 5.0), Color::new(0.9, 0.8, 0.7, 0.9), Constraints::new(window_w, window_h, window_w, window_h));
        info_panel.add_child(Box::new(text_child));
        info_panel.add_child(Box::new(text_child_2));
        self.root_node.add_child(Box::new(info_panel));
    }
}

impl Default for UiSystem {
    fn default() -> Self {
        let mut root_node = BaseUiContainer {
            children: vec![],
            padding: Point2::new(0.0, 0.0),
            layout_size: Default::default(),
            background_color: Color::new(0.0, 0.0, 0.0, 0.0),
            constraints: Constraints::new(0.0, 0.0, f32::INFINITY, f32::INFINITY),
        };
        Self {
            root_node,
        }
    }
}
