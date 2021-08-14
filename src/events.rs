use std::collections::{HashMap, HashSet};

use ggez::event::KeyCode;

use crate::*;

#[derive(Hash, PartialEq, Eq, Debug, Copy, Clone)]
pub enum EventKind {
    KeyDown,
    KeyUp,
    KeyHeld,
    MouseWheel,
    MouseButtonDown,
}

pub trait Event {
    fn kind(&self) -> EventKind;
    fn map_event(&self, world: &World) -> Option<Box<dyn Command>>;
}

// #[derive(Default)]
// pub struct EventCommandMapper {
//     event_mappers: HashMap<EventKind, Box<dyn Mapper>>,
// }

// impl EventCommandMapper {
//     pub fn map_event(&self, event: Box<dyn Event>) -> Option<Box<dyn Command>> {
//         if let Some(event_mapper) = self.event_mappers.get(&event.kind()) {
//             event_mapper.map_event(event)
//         } else {
//             None
//         }
//     }
// }

pub struct Events {
    // pub event_command_mapper: EventCommandMapper, //
    pub events: Rc<RefCell<Vec<Box<dyn Event>>>>,
    pub down_keys: HashSet<KeyCode>,
}

impl Default for Events {
    fn default() -> Self {
        Self {
            // event_command_mapper: Default::default(),
            events: Default::default(),
            down_keys: Default::default(),
        }
    }
}

impl Events {
    pub fn add(&self, event: Box<dyn Event>) {
        self.events.borrow_mut().push(event);
    }

    pub fn set_key_down(&mut self, key: KeyCode) {
        self.down_keys.insert(key);
    }

    pub fn set_key_up(&mut self, key: KeyCode) {
        self.down_keys.remove(&key);
    }

    pub fn is_key_down(&self, key: KeyCode) -> bool {
        self.down_keys.contains(&key)
    }

    pub fn spawn_held_events(&self) {
        for down_key in self.down_keys.iter() {
            self.add(Box::new(KeyHeldEvent { keycode: *down_key }))
        }
    }

    pub fn update(&mut self) {
        self.spawn_held_events();
    }
}

pub struct KeyDownEvent {
    pub keycode: ggez::event::KeyCode,
    pub keymods: ggez::event::KeyMods,
    pub repeat: bool,
}

impl Event for KeyDownEvent {
    fn kind(&self) -> EventKind {
        EventKind::KeyDown
    }

    fn map_event(&self, world: &World) -> Option<Box<dyn Command>> {
        None
    }
}

pub struct KeyUpEvent {
    pub keycode: ggez::event::KeyCode,
    pub keymods: ggez::event::KeyMods,
}

impl Event for KeyUpEvent {
    fn kind(&self) -> EventKind {
        EventKind::KeyUp
    }

    fn map_event(&self, world: &World) -> Option<Box<dyn Command>> {
        None
    }
}

pub struct KeyHeldEvent {
    pub keycode: ggez::event::KeyCode,
}

impl Event for KeyHeldEvent {
    fn kind(&self) -> EventKind {
        EventKind::KeyHeld
    }

    fn map_event(&self, world: &World) -> Option<Box<dyn Command>> {
        let d = 2.0 * world.camera.zoom;
        match self.keycode {
            KeyCode::W => Some(Box::new(MoveCameraCommand(Point2::new(0.0, d)))),
            KeyCode::A => Some(Box::new(MoveCameraCommand(Point2::new(d, 0.0)))),
            KeyCode::S => Some(Box::new(MoveCameraCommand(Point2::new(0.0, -d)))),
            KeyCode::D => Some(Box::new(MoveCameraCommand(Point2::new(-d, 0.0)))),
            _ => None,
        }
    }
}

pub struct MouseWheelEvent(pub f32);

impl Event for MouseWheelEvent {
    fn kind(&self) -> EventKind {
        EventKind::MouseWheel
    }

    fn map_event(&self, world: &World) -> Option<Box<dyn Command>> {
        Some(Box::new(ZoomCameraCommand(-self.0 * 0.05)))
    }
}

pub struct SelectProvince(ProvinceId);

impl Command for SelectProvince {
    fn run(&self, world: &mut World) {
        world.selected_province = Some(self.0.clone());
    }
}

pub struct MouseButtonDownEvent(pub Point2);

impl Event for MouseButtonDownEvent {
    fn kind(&self) -> EventKind {
        EventKind::MouseButtonDown
    }

    fn map_event(&self, world: &World) -> Option<Box<dyn Command>> {
        if let Some(province_id) = world.pixel_to_province(self.0) {
            Some(Box::new(SelectProvince(province_id.clone())))
        } else {
            None
        }
    }
}
