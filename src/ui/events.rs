use std::{cell::RefCell, rc::Rc};
use crate::*;
use container::*;


pub struct UiEvents {
    pub events: Rc<RefCell<Vec<Box<dyn UiEvent>>>>,
}

impl Default for UiEvents {
    fn default() -> Self {
        Self {
            events: Default::default(),
        }
    }
}

impl UiEvents {
    pub fn add(&self, event: Box<dyn UiEvent>) {
        self.events.borrow_mut().push(event);
    }
}

pub trait UiEvent {
    fn map_event(&self, world: &World, ui_system: &UiSystem) -> Option<Box<dyn UiCommand>>;
}

pub trait UiCommand {
    fn run(&self, ui_system: &mut UiSystem);
}

pub struct ShowProvinceInfo(pub ProvinceId);

impl UiCommand for ShowProvinceInfo {
    fn run(&self, ui_system: &mut UiSystem) {
        ui_system.info_panel.clear();
        ui_system.info_panel.add_child(Box::new(ProvinceInfoContainer {
            province: self.0.clone(),
            mapping: Box::new(|province| format!("{:?}", province.borrow().coordinate)),
            inner: TextContainer::empty(),
        }));
    }
}

impl UiEvent for MouseButtonDownEvent {
    fn map_event(&self, world: &World, ui_system: &UiSystem) -> Option<Box<dyn UiCommand>> {
        println!("map click event {:?}", self.0);
        if ui_system.click_obscured(self.0) {
            println!("click obscured");
            None
        } else if let Some(province_id) = world.pixel_to_province(self.0) {
            Some(Box::new(ShowProvinceInfo(province_id.clone())))
        } else {
            None
        }
    }
}
