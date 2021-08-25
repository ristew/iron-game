use crate::*;

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub enum SettlementFeature {
    Hilltop,
    Riverside,
    Oceanside,
    Harbor,
    Mines(GoodType),
    Fertile,
    DominantCrop(GoodType),
    Infertile,
}

impl Factored for SettlementFeature {
    fn factor(&self, world: &World, factor: FactorType) -> Option<Factor> {
        match factor {
            FactorType::CarryingCapacity => match *self {
                SettlementFeature::Riverside => Some(Factor::factor(1.5)),
                SettlementFeature::Fertile => Some(Factor::factor(1.4)),
                SettlementFeature::Infertile => Some(Factor::factor(0.5)),
                _ => None,
            },
            FactorType::SettlementRating => match *self {
                SettlementFeature::Hilltop => Some(Factor::factor(1.5)),
                SettlementFeature::Riverside => Some(Factor::factor(1.5)),
                SettlementFeature::Oceanside => Some(Factor::factor(1.1)),
                SettlementFeature::Harbor => Some(Factor::factor(1.5)),
                SettlementFeature::Mines(_) => Some(Factor::factor(1.3)),
                SettlementFeature::Fertile => Some(Factor::factor(1.4)),
                SettlementFeature::Infertile => Some(Factor::factor(0.5)),
                _ => None,
            },
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SettlementLevel {
    Hamlet,
    Village,
    Town,
    City,
    Metropolis,
}

impl SettlementLevel {
    pub fn rating(&self) -> f32 {
        match *self {
            SettlementLevel::Hamlet => 5.0,
            SettlementLevel::Village => 10.0,
            SettlementLevel::Town => 20.0,
            SettlementLevel::City => 40.0,
            SettlementLevel::Metropolis => 80.0,
        }
    }
}

pub struct SettlementRef(pub Entity);

impl SettlementRef {
    pub fn carrying_capacity(&self, world: &World) -> f32 {
        self.factor(world, FactorType::CarryingCapacity, 100.0)
    }
}

impl Factored for SettlementRef {
    fn factor(&self, world: &World, factor: FactorType) -> Option<Factor> {
    }
}

pub struct Settlement{
    pub name: String,
    pub primary_culture: CultureId,
    pub level: SettlementLevel,
}

pub struct SettlementFeatures(HashSet<SettlementFeature>);

impl Settlement {
    pub fn factor(&self, world: &World, ftype: FactorType, base: f32) -> f32 {
        let mut factors = vec![
            self.province.get().terrain.factor(world, ftype),
            self.province.get().climate.factor(world, ftype),
        ];
        factors.extend(self.features.iter().map(|f| f.factor(world, ftype)));
        apply_maybe_factors(base, factors)
    }

    pub fn carrying_capacity(&self, world: &World) -> f32 {
        self.factor(world, FactorType::CarryingCapacity, 100.0)
    }

    pub fn population(&self, world: &World) -> isize {
        let mut total_pop = 0;
        for pop in world.hecs.get::<Pops>(self).unwrap().0.iter() {
            total_pop += world.hecs.get::<Pop>();
        }
        total_pop
    }

    // rating is a measure of how attractive a settlement is
    pub fn rating(&self, world: &World) -> f32 {
        self.factor(world, FactorType::SettlementRating, self.level.rating())
    }

    pub fn accept_migrants(&mut self, world: &mut World, pop: PopId, amount: isize) {
        println!("accept_migrants {} {} of {}", self.name, amount, self.population(world));
        if let Some(dpop) = self.pops.iter().find(|p| p.get().culture == pop.get().culture) {
            dpop.get_mut().size += amount;
        } else {
            let pop_id = world.insert(Pop {
                id: None,
                size: amount,
                farmed_good: Some(Wheat),
                culture: pop.get().culture.clone(),
                settlement: self.id().clone(),
                province: self.province.clone(),
                satiety: Satiety {
                    base: 0.0,
                    luxury: 0.0,
                },
                kid_buffer: KidBuffer::new(),
                owned_goods: GoodStorage(HashMap::new()),
                migration_status: None,
                polity: self.controller.clone(),
            });
        }
    }
}
