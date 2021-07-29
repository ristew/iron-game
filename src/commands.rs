use crate::game::*;
use crate::world::*;

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
