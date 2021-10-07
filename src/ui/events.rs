use crate::*;
use container::*;
use ggez::graphics::Color;
use std::{cell::RefCell, rc::Rc};

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
    fn map_event(&self, world: &World, ui_system: &UiSystem) -> Option<Rc<dyn UiCommand>>;
}

pub trait UiCommand {
    fn run(&self, world: &World, ui_system: &mut UiSystem);
}

pub type InfoContainerPtr<T> = Rc<RefCell<InfoContainer<T>>>;

fn province_coordinate(id: ProvinceId) -> InfoContainerPtr<Province> {
    id.info_container(|province, _| format!("{:?}", province.get().coordinate))
}

fn province_population(id: ProvinceId) -> InfoContainerPtr<Province> {
    id.info_container(|province, w| format!("{:?}", province.get().population(w)))
}

fn province_controller(id: ProvinceId) -> InfoContainerPtr<Province> {
    id.info_container(|province, w| {
        if let Some(controller) = &province.get().controller {
            format!("Controlled by {}", controller.get().name)
        } else {
            "Uncontrolled".to_owned()
        }
    })
}

macro_rules! infotainer {
    ( $id:expr, $path:tt ) => {
        $id.info_container(|data, _| format!("{}", data.get().$path))
    };
}

fn pop_info(pop_id: &PopId, ui_system: &mut UiSystem) -> ButtonUiContainerPtr {
    let info_list = BaseUiContainer::new_rc(
        Point2::new(4.0, 4.0),
        Color::new(0.0, 0.0, 0.0, 0.2),
        Constraints::new(0.0, 0.0, 999.9, 999.9),
    );
    let button_id = ui_system.get_button_id();

    info_list
        .borrow_mut()
        .add_children(vec![
            pop_id.info_container(|pop, w| {
                format!(
                    "{} of {}",
                    pop.get().size,
                    pop.get().culture.get().name
                )
            }),
            pop_id.info_container(|pop, w| {
                format!(
                    "{} kids",
                    pop.get().kid_buffer.size(),
                )
            })
        ]);

    let button_container = ButtonUiContainer::new_rc(info_list, button_id);
    // ui_system.add_button(Button::new(button_id, button_container.clone(), |world, sys| {

    // }))
    button_container
}

fn pop_list(
    settlement: SettlementId,
    world: &World,
    ui_system: &mut UiSystem,
) -> BaseUiContainerPtr {
    let pop_list = BaseUiContainer::new_rc(
        Point2::new(4.0, 4.0),
        Color::new(0.0, 0.0, 0.0, 0.2),
        Constraints::new(0.0, 0.0, 999.9, 999.9),
    );
    for pop_id in settlement.get().pops.iter() {
        pop_list
            .borrow_mut()
            .add_child(pop_info(&pop_id, ui_system));
    }
    pop_list
}

fn settlement_controller(id: SettlementId) -> InfoContainerPtr<Settlement> {
    id.info_container(|settlement, w| format!("Controlled by {}", settlement.get().controller.get().name))
}


pub struct SettlementInfoBuilder(SettlementId);

impl InfoPanelBuilder for SettlementInfoBuilder {
    fn build(&self, world: &World, ui_system: &mut UiSystem) {
        let pop_list = pop_list(self.0.clone(), world, ui_system);
        ui_system.info_panel.clear();
        ui_system.info_panel.add_children(vec![
            DateContainer::new(),
            infotainer!(self.0, name),
            settlement_controller(self.0.clone()),
            self.0.info_container(|settlement, w| settlement.get().features.iter().map(|f| format!("{:?}", f)).collect::<Vec<String>>().join(", ")),
            self.0.info_container(|settlement, w| settlement.get().headman.get().title(w)),

            pop_list,
        ]);
    }
}

pub struct ShowSettlementInfo(pub SettlementId);

impl UiCommand for ShowSettlementInfo {
    fn run(&self, world: &World, ui_system: &mut UiSystem) {
        ui_system.set_info_panel(SettlementInfoBuilder(self.0.clone()))
    }
}

fn settlement_info(settlement_id: SettlementId, ui_system: &mut UiSystem) -> ButtonUiContainerPtr {
    let info_list = BaseUiContainer::new_rc(
        Point2::new(4.0, 4.0),
        Color::new(0.0, 0.0, 0.0, 0.2),
        Constraints::new(0.0, 0.0, 999.9, 999.9),
    );
    let button_id = ui_system.get_button_id();

    info_list.borrow_mut().add_children(vec![
        infotainer!(settlement_id, name),
        settlement_id.info_container(|settlement, w| {
            format!(
                "{:?} of {}",
                settlement.get().level,
                settlement.get().population(w)
            )
        }),
    ]);

    let button_container = ButtonUiContainer::new_rc(info_list, button_id);
    let set_id = settlement_id.num();
    ui_system.add_button(Button::new(
        button_id,
        button_container.clone(),
        move |world, sys| {
            sys.events
                .add(Box::new(CommandEvent(Rc::new(ShowSettlementInfo(
                    settlement_id.clone()
                )))));
        },
    ));
    button_container
}

fn settlement_list(
    province: ProvinceId,
    world: &World,
    ui_system: &mut UiSystem,
) -> BaseUiContainerPtr {
    let settlement_list = BaseUiContainer::new_rc(
        Point2::new(4.0, 4.0),
        Color::new(0.0, 0.0, 0.0, 0.2),
        Constraints::new(0.0, 0.0, 999.9, 999.9),
    );
    if let Some(settlement_id) = &province.get().settlement {
        settlement_list
            .borrow_mut()
            .add_child(settlement_info(settlement_id.clone(), ui_system));
    }
    settlement_list
}

pub struct CommandEvent(Rc<dyn UiCommand>);

impl UiEvent for CommandEvent {
    fn map_event(&self, world: &World, ui_system: &UiSystem) -> Option<Rc<dyn UiCommand>> {
        Some(self.0.clone())
    }
}

pub struct WorldInfoBuilder;

impl InfoPanelBuilder for WorldInfoBuilder {
    fn build(&self, world: &World, ui_system: &mut UiSystem) {
        ui_system.info_panel.clear();
        ui_system.info_panel.add_children(vec![
            DateContainer::new(),
            WorldInfoContainer::new(|world| format!("global population: {}", world.population))
        ]);
    }
}

pub struct ShowWorldInfo;

impl UiCommand for ShowWorldInfo {
    fn run(&self, world: &World, ui_system: &mut UiSystem) {
        ui_system.set_info_panel(WorldInfoBuilder);
    }
}



struct ProvinceInfoBuilder(ProvinceId);

impl InfoPanelBuilder for ProvinceInfoBuilder {
    fn build(&self, world: &World, ui_system: &mut UiSystem) {
        ui_system.mouse_click_tracker.areas = Vec::new();
        let settlement_list = settlement_list(self.0.clone(), world, ui_system);
        ui_system.info_panel.clear();
        ui_system.info_panel.add_children(vec![
            DateContainer::new(),
            infotainer!(self.0, terrain),
            infotainer!(self.0, coordinate),
            infotainer!(self.0, coastal),
            province_controller(self.0.clone()),
            province_population(self.0.clone()),
            settlement_list,
        ]);
    }
}

pub struct ShowProvinceInfo(pub ProvinceId);

impl UiCommand for ShowProvinceInfo {
    fn run(&self, world: &World, ui_system: &mut UiSystem) {
        ui_system.set_info_panel(ProvinceInfoBuilder(self.0.clone()));
    }
}

impl UiEvent for MouseButtonDownEvent {
    fn map_event(&self, world: &World, ui_system: &UiSystem) -> Option<Rc<dyn UiCommand>> {
        if ui_system.click_obscured(self.0) {
            None
        } else if let Some(province_id) = world.pixel_to_province(self.0) {
            Some(Rc::new(ShowProvinceInfo(province_id.clone())))
        } else {
            None
        }
    }
}
