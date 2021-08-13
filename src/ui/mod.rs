pub mod container;
pub mod events;

use std::rc::{Rc, Weak};
use container::*;
pub use container::InfoContainer;
use events::*;
use ggez::{Context, graphics::{self, Color, DrawMode, DrawParam, Drawable, Mesh, Rect, Text, draw}};

use crate::*;

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

#[derive(PartialEq, Eq, Debug)]
struct ButtonId(usize);

struct Button {
    id: ButtonId,
    bounds: Rect,
    callback: Box::<dyn Fn(&World, &Self)>,
    container: Weak<RefCell<dyn Container>>,
}

pub struct MouseClickTracker {
    areas: Vec<Button>
}

impl MouseClickTracker {
    pub fn click_buttons(&self, x: f32, y: f32, world: &World) {
        for area in self.areas.iter() {
            if area.bounds.contains([x, y]) {
                (*area.callback)(world, &area);
            }
        }
    }

    fn remove_button(&mut self, button_id: ButtonId) -> Option<Button> {
        self.areas.drain_filter(|button| button.id == button_id).next()
    }
}

/**
 * UI Binding - a hell in your menus
 * 1. Direct binding - UI component references a game object, that is consulted every draw
 * 2. Reactive updates - Object changes send Changed(Id) messages to the UI system, which updates the values referred to accordingly
 * 3. Dirty flags - dirty state is stored for each object updated, and UI checks after that for values to update
 *
 * important to note - UI is not showing a million things at once, cache misses are okay
 *
 * Reactive
 * wrap borrow_mut in the big pointer in id to queue a changed message for id
 * send changed messages to ui system
 * ui system for each changed object checks if it contains reference too it and updates the references
 * references are to an object's field ??
 * let pop: PopId = ...;
 * let text = format!("{} people", fieldread!(pop size));
 *
 * Dirtyish binding
 * UI Component trait defines a method which runs on UI redraw + world criteria
 * updates component inner state based on world state
*/
pub struct UiSystem {
    pub info_panel: BaseUiContainer,
    pub events: UiEvents,
    pub mouse_click_tracker: MouseClickTracker,
}

impl UiSystem {
    pub fn run(&mut self, ctx: &mut Context, world: &World) {
        let events = self.events.events.replace(Vec::new());
        for event in events {
            if let Some(command) = event.map_event(world, self) {
                command.run(world, self);
            }
        }
        let window_size = graphics::size(ctx);
        self.info_panel.layout(ctx, Constraints {
            min_width: 0.0,
            min_height: 0.0,
            max_width: window_size.0,
            max_height: window_size.1,
        }, world);
        self.info_panel.render(ctx, Point2::new(0.0, 0.0));
    }

    pub fn init(&mut self, ctx: &Context) {
        let text_child = TextContainer::new("test layout", Point2::new(1.0, 1.0));
        let text_child_2 = TextContainer::new("test layout dos", Point2::new(1.0, 1.0));
        let window_size = graphics::size(ctx);
        let window_w = window_size.0 / 3.5;
        let window_h = window_size.1;
        let mut info_panel = BaseUiContainer::new(Point2::new(5.0, 5.0), Color::new(0.9, 0.8, 0.7, 0.9), Constraints::new(window_w, window_h, window_w, window_h));
        info_panel.add_child(Rc::new(RefCell::new(DateContainer(TextContainer::new("", Point2::new(1.0, 1.0))))));
        info_panel.add_child(Rc::new(RefCell::new(text_child)));
        info_panel.add_child(Rc::new(RefCell::new(text_child_2)));
        self.info_panel = info_panel;
    }

    pub fn click_obscured(&self, point: Point2) -> bool {
        // println!("root_node layout_size {:?}", self.root_node.layout_size);
        point.x < self.info_panel.size().x && point.y < self.info_panel.size().y
    }
}

impl Default for UiSystem {
    fn default() -> Self {
        let info_panel = BaseUiContainer {
            children: vec![],
            padding: Point2::new(0.0, 0.0),
            layout_size: Default::default(),
            background_color: Color::new(0.0, 0.0, 0.0, 0.0),
            constraints: Constraints::new(0.0, 0.0, f32::INFINITY, f32::INFINITY),
        };
        Self {
            info_panel,
            events: UiEvents::default(),
            mouse_click_tracker: MouseClickTracker {
                areas: Default::default(),
            }
        }
    }
}
