use std::{cell::RefCell, collections::HashMap, fmt::Debug, rc::Rc, rc::Weak, hash::Hash};
use crate::*;

#[iron_data]
pub struct Pop {
    pub id: PopId,
    pub size: isize,
    pub culture: CultureId,
    pub settlement: SettlementId,
    pub province: ProvinceId,
    pub kid_buffer: KidBuffer,
    pub owned_goods: GoodStorage,
    pub satiety: Satiety,
    pub farmed_good: Option<GoodType>,
}

impl Pop {
    pub fn good_satiety(&self, good: GoodType) -> Satiety {
        good.base_satiety()
    }

    pub fn die(&mut self, amount: isize) {
        // println!("die pops: {}", amount);
        self.size = (self.size - amount).max(0);
        // println!("size: {}", self.size);
        if self.size == 0 {
            std::process::exit(0);
        }
    }
}

pub fn harvest(pop: &PopId, world: &World) {
    let pop_rc = pop.get(world);

    let pop = pop_rc.borrow();
    // println!("harvest pop?");
    if let Some(farmed_good) = pop.farmed_good {
        let mut farmed_amount = pop.size as f64;
        let carrying_capacity = pop.settlement.get(world).borrow().carrying_capacity(world);
        if farmed_amount > carrying_capacity {
            farmed_amount = carrying_capacity + (farmed_amount - carrying_capacity).sqrt();
        }
        farmed_amount *= 350.0;
        world.add_command(Box::new(SetGoodsCommand {
            good_type: farmed_good,
            amount: farmed_amount,
            pop: pop.id.clone(),
        }));
    }
}

pub enum CultureFeature {
    Warrior,
    Seafaring,
}

#[iron_data]
pub struct Culture {
    pub id: CultureId,
    pub name: String,
    pub religion: ReligionId,
    pub features: Vec<CultureFeature>,
}

#[iron_data]
pub struct Religion {
    pub id: ReligionId,
    pub name: String,
}
