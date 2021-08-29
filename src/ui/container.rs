use ggez::{
    graphics::{
        self, draw, Color, DrawMode, DrawParam, Drawable, Font, Mesh, PxScale, Rect, Text,
        TextFragment,
    },
    Context,
};

use crate::*;

use super::events::UiCommand;

pub struct ContainerId(usize);
pub type ContainerChildren = Vec<Rc<RefCell<dyn Container>>>;

pub trait Container {
    fn size(&self) -> Point2;
    fn render(
        &self,
        ctx: &mut Context,
        ui_system: &UiSystem,
        dest: Point2,
    ) -> Vec<Box<dyn UiCommand>>;
    fn layout(&mut self, ctx: &mut Context, constraints: Constraints, world: &World);
}

pub type BaseUiContainerPtr = Rc<RefCell<BaseUiContainer>>;

pub struct BaseUiContainer {
    pub children: ContainerChildren,
    pub padding: Point2,
    pub layout_size: Point2,
    pub background_color: Color,
    pub constraints: Constraints,
}

impl Container for BaseUiContainer {
    fn render(
        &self,
        ctx: &mut Context,
        ui_system: &UiSystem,
        dest: Point2,
    ) -> Vec<Box<dyn UiCommand>> {
        if self.layout_size.x == 0.0 || self.layout_size.y == 0.0 {
            return Vec::new();
        }
        let rect = Mesh::new_rectangle(
            ctx,
            DrawMode::Fill(Default::default()),
            Rect::new(dest.x, dest.y, self.layout_size.x, self.layout_size.y),
            self.background_color,
        )
        .unwrap();
        draw(ctx, &rect, DrawParam::default()).unwrap();
        let mut base_dest = dest;
        for child in self.children.iter() {
            child
                .borrow()
                .render(ctx, ui_system, base_dest + self.padding);
            base_dest.y += child.borrow().size().y + self.padding.y;
        }
        Vec::new()
    }

    fn size(&self) -> Point2 {
        self.layout_size
    }

    fn layout(&mut self, ctx: &mut Context, parent_constraints: Constraints, world: &World) {
        let constraints = self.constraints.reconcile(parent_constraints, self.padding);
        // println!("constraints: {:?}", constraints);
        self.layout_size = Point2::new(
            self.constraints.min_width,
            self.constraints.min_height + self.padding.y,
        );
        for child in self.children.iter() {
            child.borrow_mut().layout(ctx, constraints, world);
            let child_size = child.borrow().size();
            self.layout_size.y += child_size.y + self.padding.y;
            if child_size.x > self.layout_size.x {
                self.layout_size.x = child_size.x.max(constraints.max_width) + self.padding.x * 2.0;
            }
        }
    }
}

impl BaseUiContainer {
    pub fn add_child(&mut self, child: Rc<RefCell<dyn Container>>) -> &mut Self {
        self.children.push(child);
        self
    }

    pub fn add_children(&mut self, children: Vec<Rc<RefCell<dyn Container>>>) -> &mut Self {
        // TODO: change when compiler updates: for &child in children.iter() {
        for child in children.iter() {
            self.add_child(child.clone());
        }
        self
    }

    pub fn clear(&mut self) -> &mut Self {
        self.children.clear();
        self
    }

    pub fn new_rc(
        padding: Point2,
        background_color: Color,
        constraints: Constraints,
    ) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self::new(
            padding,
            background_color,
            constraints,
        )))
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

pub type ButtonUiContainerPtr = Rc<RefCell<ButtonUiContainer>>;

pub struct ButtonUiContainer {
    inner: BaseUiContainerPtr,
    button_id: ButtonId,
}

impl ButtonUiContainer {
    pub fn new_rc(inner: BaseUiContainerPtr, button_id: ButtonId) -> ButtonUiContainerPtr {
        Rc::new(RefCell::new(Self { inner, button_id }))
    }
}

impl Container for ButtonUiContainer {
    fn size(&self) -> Point2 {
        self.inner.borrow().size()
    }

    fn render(
        &self,
        ctx: &mut Context,
        ui_system: &UiSystem,
        dest: Point2,
    ) -> Vec<Box<dyn UiCommand>> {
        let cmds = self.inner.borrow_mut().render(ctx, ui_system, dest);
        let inner_size = self.inner.borrow().size();
        ui_system
            .mouse_click_tracker
            .button_bounds
            .borrow_mut()
            .insert(
                self.button_id,
                Rect::new(dest.x, dest.y, inner_size.x, inner_size.y),
            );
        cmds
    }

    fn layout(&mut self, ctx: &mut Context, constraints: Constraints, world: &World) {
        self.inner.borrow_mut().layout(ctx, constraints, world);
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

    fn render(
        &self,
        ctx: &mut Context,
        ui_system: &UiSystem,
        dest: Point2,
    ) -> Vec<Box<dyn UiCommand>> {
        self.text
            .draw(ctx, DrawParam::default().dest(dest + self.padding))
            .unwrap();

        Vec::new()
    }

    fn layout(&mut self, ctx: &mut Context, constraints: Constraints, _world: &World) {
        self.text.set_bounds(
            Point2::new(constraints.max_width, f32::INFINITY),
            graphics::Align::Left,
        );
        let computed_dimensions = self.text.dimensions(ctx);
        self.layout_size = Point2::new(computed_dimensions.w, computed_dimensions.h);
    }
}

pub fn new_text(s: String) -> Text {
    Text::new(TextFragment {
        text: s,
        color: Some(Color::BLACK),
        font: Some(Font::default()),
        scale: Some(PxScale::from(14.0)),
    })
}

impl TextContainer {
    pub fn new(text: &str, padding: Point2) -> Self {
        Self {
            padding,
            layout_size: Point2::default(),
            text: new_text(text.to_string()),
        }
    }

    pub fn empty() -> Self {
        Self {
            padding: Point2::default(),
            layout_size: Point2::default(),
            text: new_text("".to_string()),
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

    fn render(
        &self,
        ctx: &mut Context,
        ui_system: &UiSystem,
        dest: Point2,
    ) -> Vec<Box<dyn UiCommand>> {
        self.0.render(ctx, ui_system, dest)
    }

    fn layout(&mut self, ctx: &mut Context, constraints: Constraints, world: &World) {
        self.0.text = new_text(format!("{:?}", world.date));
        self.0.layout(ctx, constraints, world)
    }
}

impl DateContainer {
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(DateContainer(TextContainer::new(
            "",
            Point2::new(1.0, 1.0),
        ))))
    }
}

pub struct WorldInfoContainer {
    pub mapping: Box<dyn Fn(&World) -> String>,
    pub inner: TextContainer,
}

impl WorldInfoContainer {

    pub fn new<F>(mapping: F) -> Rc<RefCell<Self>>
    where
        F: Fn(&World) -> String + 'static,
    {
        Rc::new(RefCell::new(Self {
            mapping: Box::new(mapping),
            inner: TextContainer::empty(),
        }))
    }
}

impl Container for WorldInfoContainer {
    fn size(&self) -> Point2 {
        self.inner.size()
    }

    fn render(
        &self,
        ctx: &mut Context,
        ui_system: &UiSystem,
        dest: Point2,
    ) -> Vec<Box<dyn UiCommand>> {
        self.inner.render(ctx, ui_system, dest)
    }

    fn layout(&mut self, ctx: &mut Context, constraints: Constraints, world: &World) {
        self.inner.text = new_text((*self.mapping)(world));

        self.inner.layout(ctx, constraints, world)
    }
}

pub struct InfoContainer<T>
where
    T: IronData,
{
    pub id: T,
    pub mapping: Box<dyn Fn(T, &World) -> String>,
    pub inner: TextContainer,
}

impl<T> InfoContainer<T>
where
    T: IronData,
{
    pub fn new<F>(id: T, mapping: Box<F>) -> Rc<RefCell<Self>>
    where
        F: Fn(T, &World) -> String + 'static,
    {
        Rc::new(RefCell::new(Self {
            id,
            mapping,
            inner: TextContainer::empty(),
        }))
    }

    pub fn new_world<F>(id: T, mapping: F) -> Rc<RefCell<Self>>
    where
        F: Fn(T, &World) -> String + 'static,
    {
        Rc::new(RefCell::new(Self {
            id,
            mapping: Box::new(mapping),
            inner: TextContainer::empty(),
        }))
    }
}

impl<T> Container for InfoContainer<T>
where
    T: IronData,
{
    fn size(&self) -> Point2 {
        self.inner.size()
    }

    fn render(
        &self,
        ctx: &mut Context,
        ui_system: &UiSystem,
        dest: Point2,
    ) -> Vec<Box<dyn UiCommand>> {
        self.inner.render(ctx, ui_system, dest)
    }

    fn layout(&mut self, ctx: &mut Context, constraints: Constraints, world: &World) {
        self.inner.text = new_text((*self.mapping)(self.id.clone(), world));

        self.inner.layout(ctx, constraints, world)
    }
}
