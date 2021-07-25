use std::{cell::{RefCell, RefMut}, collections::{HashMap, VecDeque}, fmt::Debug, hash::Hash, ops::Deref, rc::{Rc, Weak}};
pub struct Coordinate {
    x: isize,
    y: isize,
}

impl Coordinate {
    pub fn new(x: isize, y: isize) -> Self {
        Self {
            x,
            y
        }
    }
}

pub enum Terrain {
    Plains,
    Hills,
    Mountains,
    Desert,
    Marsh,
    Forest,
}

pub enum Climate {
    Tropical,
    Dry,
    Mild,
    Cold,
}

pub enum GoodType {
    Wheat,
    Barley,
    OliveOil,
    Fish,
    Wine,
    Iron,
    Copper,
    Tin,
    Bronze,
    Silver,
    Gold,
    Lead,
    Salt,
    PurpleDye,
    Marble,
    Wood,
    Textiles,
    LuxuryClothes,
    Slaves, // ?? how to handle
}

#[iron_data]
pub struct Province {
    id: ProvinceId,
    settlements: Vec<SettlementId>,
    terrain: Terrain,
    climate: Climate,
    coordinate: Coordinate,
}

enum SettlementFeature {
    Hilltop,
    Riverside,
    Oceanside,
    Harbor,
    Mines(GoodType),
}

enum SettlementLevel {
    Hamlet,
    Village,
    Town,
    City,
    Metropolis
}

#[iron_data]
pub struct Settlement {
    id: SettlementId,
    name: String,
    pops: Vec<PopId>,
    features: Vec<SettlementFeature>,
    coordinate: Coordinate,
    level: SettlementLevel,
}

pub struct KidBuffer(VecDeque<usize>);

impl KidBuffer {
    pub fn new() -> Self {
        let mut deque = VecDeque::new();
        Self(deque)
    }
    pub fn spawn(&mut self, babies: usize) -> usize {
        self.0.push_front(babies);
        if (self.0.len() > 12) {
            self.0.pop_back().unwrap()
        } else {
            babies / 2
        }
    }
}

pub struct GoodStorage(HashMap<GoodType, f64>);

#[iron_data]
pub struct Pop {
    id: PopId,
    size: isize,
    culture: CultureId,
    settlement: SettlementId,
    coordinate: Coordinate,
    kid_buffer: KidBuffer,
    owned_goods: GoodStorage,
}

enum CultureFeature {
    Warrior,
    Seafaring,
}

#[iron_data]
pub struct Culture {
    id: CultureId,
    name: String,
    religion: ReligionId,
    features: Vec<CultureFeature>,
}

#[iron_data]
pub struct Religion {
    id: ReligionId,
    name: String,
}

pub enum Technology {
    Farming,
}

pub struct TechLevel(usize);

impl Debug for TechLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("#name_id({})", self.0).as_str())
    }
}

impl Hash for TechLevel {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

// impl IronId for ProvinceId {
//     type Target = Province;

//     fn borrow(&self) -> Rc<RefCell<Self::Target>> {
//         self.1.borrow().as_ref().unwrap().upgrade().unwrap()
//     }
// }

pub trait IronId {
    type Target;
    fn try_borrow(&self) -> Option<Rc<RefCell<Self::Target>>>;
    fn set_reference(&self, reference: Rc<RefCell<Self::Target>>);
    fn new(id: usize) -> Self;
}

pub trait IronData<Id> {
    type DataType;

    fn id(&self) -> Id;
}

pub struct Storage<T, Id> where T: IronData<Id>, Id: Eq + Hash {
    id_ctr: usize,
    rcs: Vec<Rc<RefCell<T>>>,
    id_map: HashMap<Id, Weak<RefCell<T>>>,
}

impl<T, Id> Storage<T, Id> where T: IronData<Id>, Id: IronId + Eq + Hash + IronId<Target = T> {
    fn insert(&mut self, item: T) -> Weak<RefCell<T>> {
        let rc = Rc::new(RefCell::new(item));
        self.rcs.push(rc.clone());
        self.id_map.insert((*rc).borrow().id(), Rc::downgrade(&rc));
        Rc::downgrade(&rc)
    }

    fn get_id(&mut self) -> Id {
        self.id_ctr += 1;
        Id::new(self.id_ctr)
    }

    fn get_ref(&self, id: &Id) -> Rc<RefCell<T>> {
        if let Some(rc) = id.try_borrow() {
            rc.clone()
        } else {
            let rc = self.id_map.get(&id).unwrap().upgrade().unwrap();
            id.set_reference(rc.clone());
            rc
        }
        // if id.1.borrow().is_none() {
        // } else {

        // }
    }
}

impl<T, Id> Default for Storage<T, Id> where T: IronData<Id>, Id: Eq + Hash {
    fn default() -> Self {
        Self {
            id_ctr: 0,
            rcs: Vec::new(),
            id_map: HashMap::new(),
        }
    }
}

pub struct World {
    day: usize,
    provinces: Storage<Province, ProvinceId>,
    cultures: Storage<Culture, CultureId>,
    religions: Storage<Religion, ReligionId>,
    settlements: Storage<Settlement, SettlementId>,
    pops: Storage<Pop, PopId>,
    commands: Rc<RefCell<Vec<Box<dyn Command>>>>
}

impl World {
    pub fn add_command(&self, command: Box<dyn Command>) {
        self.commands.borrow_mut().push(command);
    }

    pub fn is_month(&self) -> bool {
        self.day % 30 == 0
    }

    pub fn is_year(&self) -> bool {
        self.day % 360 == 0
    }

    pub fn process_command_queue(&mut self) {
        let commands = self.commands.replace(Vec::new());
        for command in commands {
            command.run(self);
        }
    }

    pub fn insert_province(&mut self, settlements: Vec<SettlementId>, terrain: Terrain, climate: Climate, coordinate: Coordinate) {
        let id = self.provinces.get_id();
        self.provinces.insert(Province {
            id,
            settlements,
            terrain,
            climate,
            coordinate,
        });
    }

    pub fn insert_settlement(&mut self, settlement: Settlement) {
        self.settlements.insert(settlement);
    }
}

impl Default for World {
    fn default() -> Self {
        Self {
            day: 0,
            provinces: Default::default(),
            cultures: Default::default(),
            religions: Default::default(),
            settlements: Default::default(),
            pops: Default::default(),
            commands: Rc::new(RefCell::new(Vec::new())),
        }
    }
}

pub trait Command {
    fn run(&self, world: &mut World);
}

pub struct PopGrowthCommand {
    amount: usize,
    pop: PopId,
}

impl Command for PopGrowthCommand {
    fn run(&self, world: &mut World) {
        let pop_rc = world.pops.get_ref(&self.pop);
        let adults = pop_rc.borrow_mut().kid_buffer.spawn(self.amount) as isize;
        pop_rc.borrow_mut().size += adults;
    }
}

pub fn pops_yearly_growth(world: &World) {
    for pop_ref in world.pops.id_map.values() {
        let pop_rc = pop_ref.upgrade().unwrap();
        println!("pop size: {}", pop_rc.borrow().size);
        let babies = pop_rc.borrow().size * 4 / 100;
        world.add_command(Box::new(PopGrowthCommand {
            amount: babies.unsigned_abs(),
            pop: pop_rc.borrow().id.clone(),
        }));
    }
}

pub fn create_test_world(world: &mut World) {
    // create provinces
    for i in 0..100 {
        for j in 0..100 {
            world.insert_province(Vec::new(), Terrain::Hills, Climate::Mild, Coordinate::new(i, j))
        }
    }

    let settlement_id = world.settlements.get_id();
    let pop_id = world.pops.get_id();
    let culture_id = world.cultures.get_id();
    let religion_id = world.religions.get_id();

    world.religions.insert(Religion {
        id: religion_id.clone(),
        name: "Test Religion".to_owned(),
    });

    world.cultures.insert(Culture {
        id: culture_id.clone(),
        name: "Test People".to_owned(),
        religion: religion_id.clone(),
        features: Vec::new(),
    });

    // let default_pop = world.insert_
    world.pops.insert(Pop {
        id: pop_id.clone(),
        size: 100,
        culture: culture_id.clone(),
        settlement: settlement_id.clone(),
        coordinate: Coordinate::new(10, 10),
        kid_buffer: KidBuffer::new(),
    });

    world.settlements.insert(Settlement {
        id: settlement_id.clone(),
        name: "Test Town".to_owned(),
        pops: vec![pop_id.clone()],
        features: Vec::new(),
        coordinate: Coordinate::new(10, 10),
        level: SettlementLevel::Village,
    });
}

pub fn day_tick(world: &World) {
    if world.is_year() {
        pops_yearly_growth(world);
    }
}

pub fn game_loop() {
    let mut world: World = Default::default();

    create_test_world(&mut world);

    loop {
        world.day += 1;
        // println!("day {}", world.day);
        day_tick(&world);
        world.process_command_queue();
    }
}
