use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    rc::Rc,
};

use rand::{Rng, thread_rng};
use rand_distr::Uniform;

use crate::*;

pub trait Command {
    fn run(&self, world: &mut World);
}

pub struct PopGrowthCommand {
    pub babies: isize,
    pub deaths: isize,
    pub pop: PopId,
}

impl Command for PopGrowthCommand {
    fn run(&self, world: &mut World) {
        let adults = self.pop.get_mut().kid_buffer.spawn(self.babies) as isize;
        self.pop.get_mut().size += adults;
        self.pop.get_mut().die(self.deaths);
        if self.pop.get().size <= 0 {
            // world.pops.remove
            world.events.add(Rc::new(PopDestroyedEvent(self.pop.clone())));
        }
    }
}

pub struct AddGoodsCommand {
    pub good_type: GoodType,
    pub amount: f32,
    pub pop: PopId,
}

impl Command for AddGoodsCommand {
    fn run(&self, world: &mut World) {
        // println!("add goods {:?} {} {:?}", self.good_type, self.amount, self.pop);
        self.pop.get_mut()
            .owned_goods
            .add(self.good_type, self.amount);
        // println!("owned {}", pop.borrow().owned_goods.amount(self.good_type));
    }
}

pub struct SetGoodsCommand {
    pub good_type: GoodType,
    pub amount: f32,
    pub pop: PopId,
}

impl Command for SetGoodsCommand {
    fn run(&self, world: &mut World) {
        // println!("set goods {:?} {} {:?}", self.good_type, self.amount, self.pop);
        // println!("owned {}", pop.borrow().owned_goods.amount(self.good_type));
        self.pop.get_mut()
            .owned_goods
            .set(self.good_type, self.amount);
    }
}

pub struct PopEatCommand(pub PopId);

/*
 * challenge of matching available food to a consistent "diet" for a pop
 * we define two kinds of satiety, base and luxury - base is not starving, luxury is living the good life
 * pops with higher base
 *
 * fulfill ideally grains + meat/oil + luxuries
 * t1 - wheat, barley?
 *  - sustenance, survival
 * t2 - sheep, Fish, olives
 *  - relative comfort, health
 * t3 - wine, salt
 *  - enjoyment, luxury
 *
*/
impl Command for PopEatCommand {
    fn run(&self, world: &mut World) {
        let pop = &self.0;
        let mut total_satiety = Satiety {
            base: 0.0,
            luxury: 0.0,
        };
        let pop_size = pop.get().size;
        let target_base = 2500.0;
        let consumed_good_order = [Wine, OliveOil, Fish, Wheat, Barley];
        for good in consumed_good_order {
            let good_owned_amount = pop.get().owned_goods.amount(good);
            let mut consumed = (good_owned_amount / 2.0)
                .min(good.max_consumed_monthly_per_capita() * pop.get().size as f32);
            // println!("{:?}-{:?}: consumed {} good owned amounts {} target base {} for {}", self.0.clone(), good, consumed, good_owned_amount, target_base, pop_size);
            let mut whole_calories =
                total_satiety.base + consumed * pop.get().good_satiety(good).base / pop_size as f32;
            // println!("cal: {} goa: {}", whole_calories, good_owned_amount);
            if whole_calories as f32 > target_base {
                consumed = consumed
                    - (whole_calories - target_base) / pop.get().good_satiety(good).base;
            }
            // println!("consumed: {}, whole calories: {}", consumed, whole_calories);
            if consumed > 0.01 {
                pop.get_mut().owned_goods.consume(good, consumed);
                total_satiety =
                    total_satiety + (consumed / pop_size as f32) * pop.get().good_satiety(good);
                if total_satiety.base > target_base {
                    break;
                }
            }
        }
        // println!("total_satiety base {}", total_satiety.base);

        if total_satiety.base < target_base * 0.6 {
            let mut dead_kids = 0;
            let mut dead_adults = 0;
            // println!("hungry! {}", total_satiety.base);
            dead_kids += pop.get_mut().kid_buffer.starve();
            if total_satiety.base < target_base * 0.3 {
                dead_kids += pop.get_mut().kid_buffer.starve();
                dead_kids += pop.get_mut().kid_buffer.starve();
                dead_kids += pop.get_mut().kid_buffer.starve();
                let dead_adult_mean = pop_size as f32 * (0.3 - total_satiety.base / target_base) / 50.0;
                dead_adults += pop.get_mut()
                    .die(positive_isample(1 + pop_size / 40, 2 + dead_adult_mean as isize));
                if pop.get().size == 0 {

                }
            }
            if dead_adults < 0 {
                let dead_adult_mean = pop_size as f32 * (0.3 - total_satiety.base / target_base) / 50.0;
                // println!("dead adults: {} - {}", dead_adults, dead_adult_mean);
            }

            if dead_kids > 0 || dead_adults > 0 {
                world.events.add(Rc::new(PopStarveEvent {
                    pop: self.0.clone(),
                    amount: dead_adults,
                    children: dead_kids,
                }));
            }
        }

        if pop.get().size == 0 {
            world.events.add(Rc::new(PopDestroyedEvent(pop.clone())));
        }

        pop.get_mut().satiety = total_satiety;
    }
}

pub struct KillCharacterCommand(pub CharacterId);

impl Command for KillCharacterCommand {
    fn run(&self, world: &mut World) {
        world.remove(&self.0);
    }
}

pub struct DestroySettlementCommand(pub SettlementId);

impl Command for DestroySettlementCommand {
    fn run(&self, world: &mut World) {
        // where do we keep track of settlements?
        self.0.get().province.get_mut().settlements.retain(|s| *s != self.0);
        if Some(self.0.clone()) == self.0.get().controller.get().capital {
            println!("polity over? move capital?");
        }
        world.remove(&self.0);
    }
}

pub struct DestroyPopCommand(pub PopId);

impl Command for DestroyPopCommand {
    fn run(&self, world: &mut World) {
        self.0.get().settlement.get_mut().pops.retain(|p| *p != self.0);
        if self.0.get().settlement.get().pops.len() == 0 {
            // println!("settlement abandoned! {}", self.0.get().settlement.get().name);
            DestroySettlementCommand(self.0.get().settlement.clone()).run(world);
        }
        world.remove(&self.0);
    }
}

pub struct MoveCameraCommand(pub Point2);

impl Command for MoveCameraCommand {
    fn run(&self, world: &mut World) {
        world.camera.p += self.0;
        // println!("world.camera.p {:?}", world.camera.p);
    }
}

pub struct ZoomCameraCommand(pub f32);

impl Command for ZoomCameraCommand {
    fn run(&self, world: &mut World) {
        world.camera.zoom = (world.camera.zoom + self.0).max(0.25).min(2.0);
    }
}

pub struct PopSeekMigrationCommand {
    pub pop: PopId,
    pub pressure: f32,
}

impl Command for PopSeekMigrationCommand {
    fn run(&self, world: &mut World) {
        let migration_desire = self.pressure;
        if migration_desire > 1.0 {
            let this_province = self.pop.get().settlement.get().province.clone();
            let coordinate = this_province.get().coordinate;
            let random_point = coordinate.random_local();
            if random_point == coordinate {
                // got unlucky, just die
                return;
            }
            if let Some(target_province_id) = world.get_province_coordinate(random_point) {
                let mut target_value = target_province_id.get().base_living_target_value();

                for settlement in target_province_id.get().settlements.iter() {
                    target_value -= 1.0;
                    if settlement.get().primary_culture != self.pop.get().culture {
                        target_value -= 2.0;
                    } else {
                        let settlement_carrying_capacity = settlement.get().carrying_capacity(world);
                        if (settlement.get().population(world) as f32) < settlement_carrying_capacity / 4.0 {
                            let size = (self.pop.get().size / 4).min((settlement_carrying_capacity / 4.0).round() as isize);
                            self.pop.get_mut().migration_status = Some(MigrationStatus {
                                migrating: size,
                                dest: target_province_id.clone(),
                                date: world.date.day + 60,
                                settlement: Some(settlement.clone()),
                            });
                            world.events.add_deferred(Rc::new(MigrationDoneEvent(self.pop.clone())), world.date.day + 60);
                            return;
                        }
                    }
                }
                if individual_event(logistic(target_value)) {
                    // println!("migrate {:?} to {}", self.pop, random_point);
                    let size = self.pop.get().size / 5;
                    self.pop.get_mut().migration_status = Some(MigrationStatus {
                        migrating: size,
                        dest: target_province_id.clone(),
                        date: world.date.day + 60,
                        settlement: None,
                    });
                    world.events.add_deferred(Rc::new(MigrationDoneEvent(self.pop.clone())), world.date.day + 60);
                }
            }
        }
    }
}

pub struct PopMigrateCommand {
    pub pop: PopId,
    pub dest: ProvinceId,
    pub migrating: isize,
    pub settlement: Option<SettlementId>,
}

impl Command for PopMigrateCommand {
    fn run(&self, world: &mut World) {
        // println!("finally migrate {:?}", self.pop);
        if let Some(settlement_id) = self.settlement.clone() {
            settlement_id.get_mut().accept_migrants(world, self.pop.clone(), self.migrating);
        } else {
            add_settlement(world, self.pop.get().culture.clone(), self.dest.clone(), self.pop.get().polity.clone(), self.migrating);
        }
        self.pop.get_mut().size -= self.migrating;
    }
}

pub struct UpdateWorldPopulation;

impl Command for UpdateWorldPopulation {
    fn run(&self, world: &mut World) {
        let new_total = world.storages.get_storage::<Pop>().ids.iter().fold(0, |acc, pop| acc + pop.get().size);
        world.population = new_total;
    }
}
