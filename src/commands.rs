use std::{cell::RefCell, collections::{HashMap, VecDeque}, rc::Rc};

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
        let pop_rc = world.pops.get_ref(&self.pop);
        let adults = pop_rc.borrow_mut().kid_buffer.spawn(self.babies) as isize;
        pop_rc.borrow_mut().size += adults - self.deaths;
        if pop_rc.borrow().size <= 0 {
            // world.pops.remove
        }
    }
}

pub struct AddGoodsCommand {
    pub good_type: GoodType,
    pub amount: f64,
    pub pop: PopId,
}

impl Command for AddGoodsCommand {
    fn run(&self, world: &mut World) {
        println!("add goods {:?} {} {:?}", self.good_type, self.amount, self.pop);
        let pop = world.pops.get_ref(&self.pop);
        pop.borrow_mut().owned_goods.add(self.good_type, self.amount);
        println!("owned {}", pop.borrow().owned_goods.amount(self.good_type));
    }
}

pub struct SetGoodsCommand {
    pub good_type: GoodType,
    pub amount: f64,
    pub pop: PopId,
}

impl Command for SetGoodsCommand {
    fn run(&self, world: &mut World) {
        println!("set goods {:?} {} {:?}", self.good_type, self.amount, self.pop);
        let pop = world.pops.get_ref(&self.pop);
        println!("owned {}", pop.borrow().owned_goods.amount(self.good_type));
        pop.borrow_mut().owned_goods.set(self.good_type, self.amount);
    }
}

pub struct PopEatCommand(pub PopId);

impl Command for PopEatCommand {
    fn run(&self, world: &mut World) {
        let pop = world.pops.get_ref(&self.0);
        let mut total_satiety = Satiety {
            base: 0.0,
            luxury: 0.0,
        };
        let pop_size = pop.borrow().size;
        let target_base = 23.0;
        while total_satiety.base < target_base {
            let mut added = 0.0;
            for good in FOOD_GOODS.iter().rev() {
                let mut amt = pop_size as f64;
                if let Some(deficit) = pop.borrow_mut().owned_goods.consume(*good, amt) {
                    amt -= deficit;
                }
                added += amt;
                total_satiety = total_satiety + (amt / pop_size as f64) * pop.borrow().good_satiety(*good);

                if total_satiety.base > target_base {
                    break;
                }
            }
            if added < 0.01 {
                break;
            }
        }

        if total_satiety.base < 20.0 {
            pop.borrow_mut().kid_buffer.starve();
            if pop.borrow().satiety.base < 10.0 {
                pop.borrow_mut().kid_buffer.starve();
                pop.borrow_mut().die(positive_isample(1 + pop_size / 40, 2 + pop_size / 20))
            }
        }

        pop.borrow_mut().satiety = total_satiety;
    }
}

pub struct MoveCameraCommand(pub Point2);

impl Command for MoveCameraCommand {
    fn run(&self, world: &mut World) {
        world.camera.p += self.0;
    }
}

