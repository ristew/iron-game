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
    fn run(&self, world: &World, ui_system: &mut UiSystem);
}

pub struct ShowProvinceInfo(pub ProvinceId);

impl UiCommand for ShowProvinceInfo {
    fn run(&self, world: &World, ui_system: &mut UiSystem) {
        let province = self.0.get(world);
        ui_system.info_panel.clear();
        ui_system.info_panel.add_child(
            InfoContainer::<Province>::new(self.0.clone(), |province, _| format!("{:?}", province.borrow().coordinate)));
        ui_system.info_panel.add_child(
            InfoContainer::<Province>::new(self.0.clone(), |province, w| format!("{:?}", province.borrow().population(w))));
        for settlement_id in province.borrow().settlements.iter() {
            ui_system.info_panel.add_child(
                InfoContainer::<Settlement>::new(settlement_id.clone(), |settlement, _| format!("{}", settlement.borrow().name))
            );
            ui_system.info_panel.add_child(
                InfoContainer::<Settlement>::new(settlement_id.clone(), |settlement, w| format!("{}", settlement.borrow().population(w)))
            );
        }
    }
}

impl UiEvent for MouseButtonDownEvent {
    fn map_event(&self, world: &World, ui_system: &UiSystem) -> Option<Box<dyn UiCommand>> {
        if ui_system.click_obscured(self.0) {
            None
        } else if let Some(province_id) = world.pixel_to_province(self.0) {
            Some(Box::new(ShowProvinceInfo(province_id.clone())))
        } else {
            None
        }
    }
}
