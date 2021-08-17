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
        let pop_rc = world.get_ref::<Pop>(&self.pop);
        let adults = pop_rc.borrow_mut().kid_buffer.spawn(self.babies) as isize;
        pop_rc.borrow_mut().size += adults - self.deaths;
        if pop_rc.borrow().size <= 0 {
            // world.pops.remove
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
        let pop = world.get_ref::<Pop>(&self.pop);
        pop.borrow_mut()
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
        let pop = self.pop.get(world);
        // println!("owned {}", pop.borrow().owned_goods.amount(self.good_type));
        pop.borrow_mut()
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
        let pop = self.0.get(world);
        let mut total_satiety = Satiety {
            base: 0.0,
            luxury: 0.0,
        };
        let pop_size = pop.borrow().size;
        let target_base = 2500.0;
        let consumed_good_order = [Wine, OliveOil, Fish, Wheat, Barley];
        for good in consumed_good_order {
            let good_owned_amount = pop.borrow().owned_goods.amount(good);
            let mut consumed = (good_owned_amount / 2.0)
                .min(good.max_consumed_monthly_per_capita() * pop.borrow().size as f32);
            // println!("{:?}-{:?}: consumed {} good owned amounts {} target base {} for {}", self.0.clone(), good, consumed, good_owned_amount, target_base, pop_size);
            let mut whole_calories =
                total_satiety.base + consumed * pop.borrow().good_satiety(good).base / pop_size as f32;
            // println!("cal: {} goa: {}", whole_calories, good_owned_amount);
            if whole_calories as f32 > target_base {
                consumed = consumed
                    - (whole_calories - target_base) / pop.borrow().good_satiety(good).base;
            }
            // println!("consumed: {}, whole calories: {}", consumed, whole_calories);
            if consumed > 0.01 {
                pop.borrow_mut().owned_goods.consume(good, consumed);
                total_satiety =
                    total_satiety + (consumed / pop_size as f32) * pop.borrow().good_satiety(good);
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
            dead_kids += pop.borrow_mut().kid_buffer.starve();
            if total_satiety.base < target_base * 0.3 {
                dead_kids += pop.borrow_mut().kid_buffer.starve();
                dead_adults += pop.borrow_mut()
                    .die(positive_isample(1 + pop_size / 40, 2 + pop_size / 20))
            }

            world.events.add(Box::new(PopStarveEvent {
                pop: self.0.clone(),
                amount: dead_adults,
                children: dead_kids,
            }));
        }

        pop.borrow_mut().satiety = total_satiety;
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
    pub starved: isize,
}

impl Command for PopSeekMigrationCommand {
    fn run(&self, world: &mut World) {
        let migration_desire = self.starved;
        if migration_desire > 1 {
            let this_province = self.pop.get(world).borrow().settlement.get(world).borrow().province.get(world);
            let coordinate = this_province.borrow().coordinate;
            let random_point = Coordinate::new(
                coordinate.x + thread_rng().sample(Uniform::new(-5, 5)),
                coordinate.y + thread_rng().sample(Uniform::new(-5, 5)),
            );
            if random_point == coordinate {
                // got unlucky, just die
                return;
            }
            if let Some(target_province_id) = world.get_province_coordinate(random_point) {
                let target_province = target_province_id.get(world);
                let mut target_value = target_province.borrow().base_living_target_value();

                for settlement in target_province.borrow().settlements.iter() {
                    if settlement.get(world).borrow().primary_culture != self.pop.get(world).borrow().culture {
                        target_value -= 2.0;
                    }
                }
                if individual_event(logistic(target_value)) {
                    println!("migrate {:?} to {}", self.pop, random_point);
                    let size = self.pop.get(world).borrow().size / 5;
                    self.pop.get(world).borrow_mut().migration_status = Some(MigrationStatus {
                        migrating: size,
                        dest: target_province_id.clone(),
                        date: world.date.day + 60,
                    });
                }
            }
        }
    }
}
