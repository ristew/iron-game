use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use ggez::event::KeyCode;

use crate::*;

#[derive(Hash, PartialEq, Eq, Debug, Copy, Clone)]
pub enum EventKind {
    KeyDown,
    KeyUp,
    KeyHeld,
    MouseWheel,
    MouseButtonDown,
    PopStarve,
    MigrationDone,
    PopDestroyed,
    CharacterDied,
}

pub trait Event {
    fn kind(&self) -> EventKind;
    fn map_event(&self, world: &World) -> Vec<Box<dyn Command>>;
    fn subjects(&self) -> Vec<GameId>;
    fn short_description(&self) -> String {
        format!("{:?} {:?}", self.kind(), self.subjects())
    }
}

// #[derive(Default)]
// pub struct EventCommandMapper {
//     event_mappers: HashMap<EventKind, Box<dyn Mapper>>,
// }

// impl EventCommandMapper {
//     pub fn map_event(&self, event: Rc<dyn Event>) -> Vec<Box<dyn Command>> {
//         if let Some(event_mapper) = self.event_mappers.get(&event.kind()) {
//             event_mapper.map_event(event)
//         } else {
//             None
//         }
//     }
// }

pub struct EventChannel {
    pub events: RefCell<Vec<Rc<dyn Event>>>,
}

impl EventChannel {
    pub fn new() -> Self {
        Self {
            events: RefCell::new(Vec::new()),
        }
    }
}

pub struct EventIdChannels<Id> where Id: IronId {
    channels: HashMap<Id, Vec<Rc<dyn Event>>>,
}

impl<Id> EventIdChannels<Id> where Id: IronId + Eq + Hash + Clone {
    pub fn new() -> Self {
        Self {
            channels: HashMap::new(),
        }
    }

    pub fn publish(&mut self, id: Id, event: Rc<dyn Event>) {
        if !self.channels.contains_key(&id) {
            self.channels.insert(id, vec![event]);
        } else {
            self.channels.get_mut(&id).unwrap().push(event);
        }
    }

    pub fn iter(&mut self, id: Id) -> impl Iterator<Item = &Rc<dyn Event>> {
        if !self.channels.contains_key(&id) {
            self.channels.insert(id.clone(), vec![]);
        }

        self.channels.get(&id).unwrap().iter()
    }

    pub fn clear(&mut self) {

    }
}

pub struct EventChannels {
    event_id_channels: HashMap<TypeId, Box<dyn Any>>,
}

impl EventChannels {
    pub fn get_by_id<Id>(&self) -> &EventIdChannels<Id> where Id: IronId + 'static {
        self.event_id_channels
            .get(&TypeId::of::<Id>())
            .unwrap()
            .downcast_ref::<EventIdChannels<Id>>()
            .unwrap()
    }

    pub fn get_by_id_mut<Id>(&mut self) -> &mut EventIdChannels<Id> where Id: IronId + 'static {
        self.event_id_channels
            .get_mut(&TypeId::of::<Id>())
            .unwrap()
            .downcast_mut::<EventIdChannels<Id>>()
            .unwrap()
    }

    pub fn iter<Id>(&mut self, id: Id) -> impl Iterator<Item = &Rc<dyn Event>> where Id: IronId + Eq + Hash + Clone + 'static {
        self.get_by_id_mut::<Id>().iter(id)
    }

    pub fn publish<Id>(&mut self, id: Id, event: Rc<dyn Event>) where Id: IronId + Eq + Hash + Clone + 'static {
        self.get_by_id_mut::<Id>().publish(id, event);
    }
}

impl Default for EventChannels {
    fn default() -> Self {
        let mut eid_channels: HashMap<TypeId, Box<dyn Any>> = HashMap::new();
        macro_rules! init_eid_channel {
            ( $typ:ident ) => {
                eid_channels.insert(
                    TypeId::of::<$typ>(),
                    Box::new(EventIdChannels::<$typ>::new()),
                );
            }
        }
        init_eid_channel!(ProvinceId);
        init_eid_channel!(PopId);
        init_eid_channel!(SettlementId);
        init_eid_channel!(CultureId);
        init_eid_channel!(ReligionId);
        init_eid_channel!(LanguageId);
        init_eid_channel!(PolityId);
        init_eid_channel!(CharacterId);
        Self { event_id_channels: eid_channels }
    }
}

pub struct Events {
    // pub event_command_mapper: EventCommandMapper, //
    pub events:RefCell<Vec<Rc<dyn Event>>>,
    pub deferred: RefCell<HashMap<usize, Vec<Rc<dyn Event>>>>,
    pub down_keys: HashSet<KeyCode>,
}

impl Default for Events {
    fn default() -> Self {
        Self {
            // event_command_mapper: Default::default(),
            events: Default::default(),
            down_keys: Default::default(),
            deferred: Default::default(),
        }
    }
}

impl Events {
    pub fn add(&self, event: Rc<dyn Event>) {
        self.events.borrow_mut().push(event);
    }

    pub fn add_list(&self, events: Vec<Rc<dyn Event>>) {
        self.events.borrow_mut().extend(events.into_iter());
    }

    pub fn add_deferred(&self, event: Rc<dyn Event>, date: usize) {
        if self.deferred.borrow().contains_key(&date) {
            self.deferred.borrow_mut().get_mut(&date).unwrap().push(event);
        } else {
            self.deferred.borrow_mut().insert(date, vec![event]);
        }

    }

    pub fn get_deferred(&self, date: Date) -> Vec<Rc<dyn Event>> {
        if let Some(events) = self.deferred.borrow_mut().remove(&date.day) {
            events
        } else {
            Vec::new()
        }
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
            self.add(Rc::new(KeyHeldEvent { keycode: *down_key }))
        }
    }

    pub fn update(&self, world: &World) {
        self.add_list(self.get_deferred(world.date));
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

    fn map_event(&self, world: &World) -> Vec<Box<dyn Command>> {
        vec![]
    }

    fn subjects(&self) -> Vec<GameId> {
        vec![]
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

    fn map_event(&self, world: &World) -> Vec<Box<dyn Command>> {
        vec![]
    }

    fn subjects(&self) -> Vec<GameId> {
        vec![]
    }
}

pub struct KeyHeldEvent {
    pub keycode: ggez::event::KeyCode,
}

impl Event for KeyHeldEvent {
    fn kind(&self) -> EventKind {
        EventKind::KeyHeld
    }

    fn map_event(&self, world: &World) -> Vec<Box<dyn Command>> {
        let d = 2.0 * world.camera.zoom;
        match self.keycode {
            KeyCode::W => vec![Box::new(MoveCameraCommand(Point2::new(0.0, d)))],
            KeyCode::A => vec![Box::new(MoveCameraCommand(Point2::new(d, 0.0)))],
            KeyCode::S => vec![Box::new(MoveCameraCommand(Point2::new(0.0, -d)))],
            KeyCode::D => vec![Box::new(MoveCameraCommand(Point2::new(-d, 0.0)))],
            _ => vec![],
        }
    }

    fn subjects(&self) -> Vec<GameId> {
        vec![]
    }
}

pub struct MouseWheelEvent(pub f32);

impl Event for MouseWheelEvent {
    fn kind(&self) -> EventKind {
        EventKind::MouseWheel
    }

    fn map_event(&self, world: &World) -> Vec<Box<dyn Command>> {
        vec![Box::new(ZoomCameraCommand(1.0 - self.0 * 0.1))]
    }

    fn subjects(&self) -> Vec<GameId> {
        vec![]
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

    fn map_event(&self, world: &World) -> Vec<Box<dyn Command>> {
        if let Some(province_id) = world.pixel_to_province(self.0) {
            vec![Box::new(SelectProvince(province_id.clone()))]
        } else {
            vec![]
        }
    }

    fn subjects(&self) -> Vec<GameId> {
        vec![]
    }
}

pub struct PopStarveEvent {
    pub pop: PopId,
    pub amount: isize,
    pub children: isize,
}

impl Event for PopStarveEvent {
    fn kind(&self) -> EventKind {
        EventKind::PopStarve
    }

    fn map_event(&self, world: &World) -> Vec<Box<dyn Command>> {
        // println!("pop starve: {:?}: {}/{}, amount: {}, kids: {}", world.date, self.pop.get().size, self.pop.get().settlement.get().carrying_capacity(world), self.amount, self.children);
        vec![Box::new(PopSeekMigrationCommand {
            pop: self.pop.clone(),
            pressure: (self.amount + self.children / 2) as f32,
        })]
    }

    fn subjects(&self) -> Vec<GameId> {
        self.pop.gids()
    }
}

pub struct MigrationDoneEvent(pub PopId);

impl Event for MigrationDoneEvent {
    fn kind(&self) -> EventKind {
        EventKind::MigrationDone
    }

    fn map_event(&self, world: &World) -> Vec<Box<dyn Command>> {
        if let Some(migration_status) = self.0.get().migration_status.as_ref() {
            if migration_status.date < world.date.day {
                vec![]
            } else {
                vec![Box::new(PopMigrateCommand {
                    pop: self.0.clone(),
                    dest: migration_status.dest.clone(),
                    migrating: migration_status.migrating.min(self.0.get().size),
                    settlement: migration_status.settlement.clone(),
                })]
            }

        } else {
            vec![]
        }


    }

    fn subjects(&self) -> Vec<GameId> {
        self.0.gids()
    }
}

pub struct CharacterDiedEvent(pub CharacterId);

impl Event for CharacterDiedEvent {
    fn kind(&self) -> EventKind {
        EventKind::CharacterDied
    }

    fn map_event(&self, world: &World) -> Vec<Box<dyn Command>> {
        // println!("character died: {}", self.0.get().title(world));
        let mut commands: Vec<Box<dyn Command>> = Vec::new();
        for title in self.0.get().titles.iter() {
            match title {
                Title::PolityLeader(polity) => {
                    if polity.get().leader == self.0 {
                        commands.push(Box::new(PolityUpdateLeaderCommand(polity.clone())));
                    }
                },
                Title::SettlementLeader(settlement) => {
                    if settlement.get().headman == self.0 {
                        commands.push(Box::new(SettlementUpdateHeadmanCommand(settlement.clone())));
                    }
                },
            }
        }
        commands.push(Box::new(KillCharacterCommand(self.0.clone())));
        commands
    }

    fn subjects(&self) -> Vec<GameId> {
        self.0.gids()
    }
}

pub struct PopDestroyedEvent(pub PopId);

impl Event for PopDestroyedEvent {
    fn kind(&self) -> EventKind {
        EventKind::PopDestroyed
    }

    fn map_event(&self, world: &World) -> Vec<Box<dyn Command>> {
        vec![Box::new(DestroyPopCommand(self.0.clone()))]
    }

    fn subjects(&self) -> Vec<GameId> {
        self.0.gids()
    }
}
