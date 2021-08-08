use std::{cell::{RefCell, RefMut}, collections::{HashMap, VecDeque}, fmt::Debug, hash::Hash, marker::PhantomData, ops::Deref, rc::{Rc, Weak}, thread::{sleep, sleep_ms}, time::Duration};
use ggez::{Context, GameError, event::EventHandler, graphics::{Color, clear, present}, timer};
use lazy_static::lazy_static;
use rand::{prelude::SliceRandom, thread_rng};
use crate::*;

pub const TILE_SIZE_X: f32 = 16.0;
pub const TILE_SIZE_Y: f32 = 16.0;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Coordinate {
    pub x: isize,
    pub y: isize,
}

impl Coordinate {
    pub fn z(&self) -> isize {
        -self.x - self.y
    }

    pub fn base_pixel_pos(&self) -> Point2 {
        let tile_x = TILE_SIZE_X * (SQRT_3 * self.x as f32 + SQRT_3 / 2.0 * self.y as f32);
        let tile_y = TILE_SIZE_Y * 1.5 * self.y as f32;
        Point2::new(tile_x, tile_y)
    }

    pub fn pixel_pos(&self, camera: &Camera) -> Point2 {
        camera.translate(self.base_pixel_pos())
    }

    pub fn from_pixel_pos(point: Point2) -> Self {
        let coord_x = (SQRT_3 / 3.0 * point.x - point.y / 3.0) / TILE_SIZE_X;
        let coord_y = (2.0 * point.y / 3.0) / TILE_SIZE_Y;
        Self::from_cube_round(coord_x, coord_y)
    }

    pub fn from_cube_round(x: f32, y: f32) -> Self {
        let z = -x - y;
        let mut rx = x.round();
        let mut ry = y.round();
        let rz = z.round();
        let xdiff = (rx - x).abs();
        let ydiff = (ry - y).abs();
        let zdiff = (rz - z).abs();
        if xdiff > ydiff + zdiff {
            rx = -ry - rz;
        } else if ydiff > zdiff {
            ry = -rx - rz;
        // } else {
        //     rz = -rx - ry;
        }
        Self {
            x: rx as isize,
            y: ry as isize,
        }
    }

    pub fn new(x: isize, y: isize) -> Self {
        Self {
            x,
            y
        }
    }


    // pub fn from_window_pos(pos: Vec2, ) -> Self {
    //     Self::from_pixel_pos(pos)
    // }

    pub fn neighbors(&self) -> Vec<Coordinate> {
        let mut ns = Vec::new();
        let directions = vec![
            (1, 0),
            (1, -1),
            (0, -1),
            (-1, 0),
            (-1, 1),
            (0, 1)
        ];
        for (dx, dy) in directions {
            ns.push(Coordinate {
                x: self.x + dx,
                y: self.y + dy,
            });
        }
        ns
    }

    pub fn neighbors_shuffled(&self) -> Vec<Coordinate> {
        let mut result = self.neighbors();
        result.shuffle(&mut thread_rng());
        result
    }

    pub fn neighbors_iter(&self) -> CoordinateIter {
        CoordinateIter {
            neighbors: self.neighbors(),
        }
    }

    pub fn neighbors_shuffled_iter(&self) -> CoordinateIter {
        CoordinateIter {
            neighbors: self.neighbors_shuffled(),
        }
    }

    pub fn neighbors_in_radius(&self, radius: isize) -> Vec<Coordinate> {
        let mut items = Vec::new();
        for x in -radius..(radius + 1) {
            let min = (-radius).max(-x - radius);
            let max = radius.min(-x + radius);
            for y in min..(max + 1) {
                items.push(Coordinate { x: self.x + x, y: self.y + y });
            }
        }
        items
    }
    pub fn neighbors_in_radius_iter(&self, radius: isize) -> CoordinateIter {
        CoordinateIter {
            neighbors: self.neighbors_in_radius(radius),
        }
    }
}

pub struct CoordinateIter {
    neighbors: Vec<Coordinate>,
}

impl Iterator for CoordinateIter {
    type Item = Coordinate;

    fn next(&mut self) -> Option<Coordinate> {
        self.neighbors.pop()
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

impl Factored for Terrain {
    fn factor(&self, factor: FactorType) -> Option<Factor> {
        match factor {
            FactorType::CarryingCapacity => Some(match *self {
                Terrain::Plains => Factor::factor(1.0),
                Terrain::Hills => Factor::factor(0.7),
                Terrain::Mountains => Factor::factor(0.2),
                Terrain::Desert => Factor::factor(0.1),
                Terrain::Marsh => Factor::factor(0.5),
                Terrain::Forest => Factor::factor(0.5),
            }),
        }
    }
}

pub enum Climate {
    Tropical,
    Dry,
    Mild,
    Cold,
}

impl Factored for Climate {
    fn factor(&self, factor: FactorType) -> Option<Factor> {
        match factor {
            FactorType::CarryingCapacity => Some(match *self {
                Climate::Tropical => Factor::factor(1.2),
                Climate::Dry => Factor::factor(0.7),
                Climate::Mild => Factor::factor(1.0),
                Climate::Cold => Factor::factor(0.7),
            }),
        }
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
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

pub use GoodType::*;

#[derive(Debug, Copy, Clone)]
pub enum FactorType {
    CarryingCapacity,
}

pub enum FactorOp {
    Add,
    Mul,
}

pub struct Factor {
    amount: f64,
    op: FactorOp,
}

impl Factor {
    pub fn factor(amount: f64) -> Self {
        Self {
            amount,
            op: FactorOp::Mul,
        }
    }

    pub fn bonus(amount: f64) -> Self {
        Self {
            amount,
            op: FactorOp::Add,
        }
    }
}

pub fn apply_maybe_factors(base: f64, factors: Vec<Option<Factor>>) -> f64 {
    let mut bonus = 0.0;
    let mut res = base;
    for factor_opt in factors.iter() {
        if let Some(factor) = factor_opt {
            match factor.op {
                FactorOp::Add => bonus += factor.amount,
                FactorOp::Mul => res *= factor.amount,
            }
        }
    }

    res + bonus
}

pub trait Factored {
    fn factor(&self, factor: FactorType) -> Option<Factor>;
}

lazy_static! {
    pub static ref FOOD_GOODS: Vec<GoodType> = vec![
        Wheat,
        Barley,
        Fish,
        OliveOil,
        Salt,
        Wine,
    ];
}

impl GoodType {
    pub fn base_satiety(&self) -> Satiety {
        match *self {
            Wheat => Satiety { base: 1.0, luxury: 0.1 },
            Barley => Satiety { base: 1.0, luxury: 0.0 },
            OliveOil => Satiety { base: 0.5, luxury: 0.5 },
            Fish => Satiety { base: 0.8, luxury: 0.2 },
            Wine => Satiety { base: 0.1, luxury: 1.0 },
            Salt => Satiety { base: 0.3, luxury: 0.5 },
            _ => Satiety { base: 0.0, luxury: 0.0 },
        }
    }
}

#[iron_data]
pub struct Province {
    pub id: ProvinceId,
    pub settlements: Vec<SettlementId>,
    pub terrain: Terrain,
    pub climate: Climate,
    pub coordinate: Coordinate,
    pub harvest_month: usize,
}

pub enum SettlementFeature {
    Hilltop,
    Riverside,
    Oceanside,
    Harbor,
    Mines(GoodType),
}

impl Factored for SettlementFeature {
    fn factor(&self, factor: FactorType) -> Option<Factor> {
        match factor {
            FactorType::CarryingCapacity => match *self {
                SettlementFeature::Riverside => Some(Factor::factor(1.2)),
                _ => None,
            }
        }
    }
}

pub enum SettlementLevel {
    Hamlet,
    Village,
    Town,
    City,
    Metropolis
}

#[iron_data]
pub struct Settlement {
    pub id: SettlementId,
    pub name: String,
    pub pops: Vec<PopId>,
    pub features: Vec<SettlementFeature>,
    pub primary_culture: CultureId,
    pub coordinate: Coordinate,
    pub level: SettlementLevel,
}

impl Settlement {
    pub fn carrying_capacity(&self, world: &World) -> f64 {
        let province_rc = world.get_province_coordinate(self.coordinate).unwrap().get(world);
        let province = province_rc.borrow();
        let factor = FactorType::CarryingCapacity;
        let mut factors = vec![province.terrain.factor(factor), province.climate.factor(factor)];
        factors.extend(self.features.iter().map(|f| f.factor(factor)));
        apply_maybe_factors(500.0, factors)
    }
}

#[derive(Debug)]
pub struct KidBuffer(VecDeque<isize>);

impl KidBuffer {
    pub fn new() -> Self {
        Self(VecDeque::new())
    }
    pub fn spawn(&mut self, babies: isize) -> isize {
        // println!("spawn babies {}", babies);
        self.0.push_front(babies);
        // println!("{:?}", self);
        if self.0.len() > 12 {
            self.0.pop_back().unwrap()
        } else {
            babies / 2
        }
    }

    pub fn starve(&mut self) {
        let cohort = sample(3.0).abs().min(12.0) as usize;
        if self.0.len() > cohort {
            let cohort_size = self.0[cohort];
            let dead_kids = positive_isample(cohort_size / 20 + 1, cohort_size / 10 + 1);
            // println!("cohort {}, size {}, dead {}", cohort, cohort_size, dead_kids);
            self.0[cohort] = (cohort_size - dead_kids).max(0)
        }
    }
}

#[derive(PartialEq)]
pub struct Satiety {
    pub base: f64,
    pub luxury: f64,
}

impl std::ops::Add for Satiety {
    type Output = Satiety;

    fn add(self, rhs: Self) -> Self::Output {
        Satiety {
            base: self.base + rhs.base,
            luxury: self.luxury + rhs.luxury,
        }
    }
}

impl std::ops::AddAssign for Satiety {
    fn add_assign(&mut self, rhs: Self) {
        *self = Satiety {
            base: self.base + rhs.base,
            luxury: self.luxury + rhs.luxury,
        };
    }
}

impl std::ops::Mul<Satiety> for f64 {
    type Output = Satiety;

    fn mul(self, rhs: Satiety) -> Self::Output {
        Satiety {
            base: rhs.base * self,
            luxury: rhs.luxury * self,
        }
    }
}

pub struct GoodStorage(pub HashMap<GoodType, f64>);

impl GoodStorage {
    pub fn amount(&self, good: GoodType) -> f64 {
        *self.0.get(&good).unwrap_or(&0.0)
    }

    pub fn consume(&mut self, good: GoodType, amount: f64) -> Option<f64> {
        if let Some(mut stored) = self.0.get_mut(&good) {
            if *stored < amount {
                let deficit = amount - *stored;
                *stored = 0.0;
                Some(deficit)
            } else {
                *stored -= amount;
                None
            }
        } else {
            Some(amount)
        }
    }

    pub fn add(&mut self, good: GoodType, amount: f64) -> f64 {
        if let Some(stored) = self.0.get_mut(&good) {
            *stored += amount;
            *stored
        } else {
            self.0.insert(good, amount);
            amount
        }
    }

    pub fn set(&mut self, good: GoodType, amount: f64) {
        self.0.insert(good, amount);
    }

    // pub fn try_eat_diet(&self, diet: Diet) -> Vec<(GoodType, f64)> {
    //     let mut bad_res = Vec::new();

    //     for part in diet.0.iter() {
    //         if self.amount(part.0) < part.1 {
    //             bad_res.push(*part);
    //         }
    //     }

    //     bad_res
    // }
    //
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


pub trait IronId {
    type Target: IronData<IdType = Self>;
    fn try_borrow(&self) -> Option<Rc<RefCell<Self::Target>>>;
    fn set_reference(&self, reference: Rc<RefCell<Self::Target>>);
    fn new(id: usize) -> Self;
    fn num(&self) -> usize;
    fn get(&self, world: &World) -> Rc<RefCell<Self::Target>>;
}


pub trait IronData {
    type DataType;
    type IdType: IronId<Target = Self> + Debug;

    fn id(&self) -> Self::IdType;
}

pub struct MainState {
    world: World,
    ui_system: UiSystem,
}

impl MainState {
    pub fn new(ctx: &mut Context) -> Self {
        let mut world: World = World::new(ctx);
        let mut ui_system = UiSystem::default();

        ui_system.init(ctx);
        create_test_world(&mut world);
        Self {
            world,
            ui_system,
        }
    }
}

pub const FPS: f32 = 120.0;
pub const FRAME_TIME: f32 = 1.0 / FPS;

impl EventHandler<GameError> for MainState {
    fn update(&mut self, ctx: &mut ggez::Context) -> Result<(), GameError> {
        if timer::delta(ctx).as_secs_f32() < FRAME_TIME {
            timer::sleep(Duration::from_secs_f32(FRAME_TIME) - timer::delta(ctx));
        }
        self.world.date.day += 1;
        day_tick(&self.world);

        if self.world.date.is_month() {
            println!("{:?}", self.world.date);
            println!("{:?}", self.world.camera.p);
        }
        self.world.process_events();
        self.world.process_command_queue();
        timer::yield_now();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> Result<(), GameError> {
        clear(ctx, Color::BLACK);
        render_world(&mut self.world, ctx);
        self.ui_system.run(ctx, &self.world);
        present(ctx).unwrap();
        timer::yield_now();
        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut ggez::Context,
        button: ggez::event::MouseButton,
        x: f32,
        y: f32,
    ) {
        let point = Point2::new(x, y);
        self.ui_system.events.add(Box::new(MouseButtonDownEvent(point)));
    }

    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut ggez::Context,
        _button: ggez::event::MouseButton,
        _x: f32,
        _y: f32,
    ) {
    }

    fn mouse_motion_event(&mut self, _ctx: &mut ggez::Context, _x: f32, _y: f32, _dx: f32, _dy: f32) {}

    fn mouse_enter_or_leave(&mut self, _ctx: &mut ggez::Context, _entered: bool) {}

    fn mouse_wheel_event(&mut self, _ctx: &mut ggez::Context, _x: f32, y: f32) {
        if y != 0.0 {
            self.world.events.add(Box::new(MouseWheelEvent(y)))
        }
    }

    fn key_down_event(
        &mut self,
        ctx: &mut ggez::Context,
        keycode: ggez::event::KeyCode,
        keymods: ggez::event::KeyMods,
        repeat: bool,
    ) {
        if keycode == ggez::event::KeyCode::Escape {
            ggez::event::quit(ctx);
        } else {
            self.world.events.add(Box::new(KeyDownEvent {
                keycode,
                keymods,
                repeat,
            }));
            self.world.events.set_key_down(keycode);
        }
    }

    fn key_up_event(&mut self, _ctx: &mut ggez::Context, keycode: ggez::event::KeyCode, keymods: ggez::event::KeyMods) {
        self.world.events.add(Box::new(KeyUpEvent {
            keycode,
            keymods,
        }));
        self.world.events.set_key_up(keycode);
    }

    fn text_input_event(&mut self, _ctx: &mut ggez::Context, _character: char) {}

    fn gamepad_button_down_event(&mut self, _ctx: &mut ggez::Context, _btn: ggez::event::Button, _id: ggez::event::GamepadId) {}

    fn gamepad_button_up_event(&mut self, _ctx: &mut ggez::Context, _btn: ggez::event::Button, _id: ggez::event::GamepadId) {}

    fn gamepad_axis_event(&mut self, _ctx: &mut ggez::Context, _axis: ggez::event::Axis, _value: f32, _id: ggez::event::GamepadId) {
    }

    fn focus_event(&mut self, _ctx: &mut ggez::Context, _gained: bool) {}

    fn quit_event(&mut self, _ctx: &mut ggez::Context) -> bool {
        println!("quit_event() callback called, quitting...");
        false
    }

    fn resize_event(&mut self, _ctx: &mut ggez::Context, _width: f32, _height: f32) {}

    fn on_error(&mut self, _ctx: &mut ggez::Context, _origin: ggez::event::ErrorOrigin, _e: GameError) -> bool {
        true
    }


}
