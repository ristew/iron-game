use inflector::cases::titlecase::to_title_case;
use rand::{Rng, distributions::Slice, random, thread_rng};
use rand_distr::Uniform;

use crate::*;
use std::{cell::RefCell, collections::HashMap, fmt::Debug, hash::Hash, rc::Rc, rc::Weak};

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

    pub fn die(&mut self, amount: isize) -> isize {
        // println!("die pops: {}", amount);
        let before = self.size;
        self.size = (self.size - amount).max(0);

        // println!("size: {}", self.size);
        before - self.size
    }
}

pub fn harvest(pop: &PopId, world: &World) {
    let pop_rc = pop.get(world);

    let pop = pop_rc.borrow();
    // println!("harvest pop?");
    if let Some(farmed_good) = pop.farmed_good {
        let mut farmed_amount = pop.size as f32;
        let carrying_capacity = pop.settlement.get(world).borrow().carrying_capacity(world);
        if farmed_amount > carrying_capacity {
            farmed_amount = carrying_capacity + (farmed_amount - carrying_capacity).sqrt();
        }
        if random::<f32>() > 0.9 {
            // println!("failed harvest! halving farmed goods");
            farmed_amount *= 0.5;
        }
        world.add_command(Box::new(SetGoodsCommand {
            good_type: farmed_good,
            amount: farmed_amount * 300.0,
            pop: pop.id.clone(),
        }));
    }
}

#[iron_data]
pub struct Language {
    pub id: LanguageId,
    pub name: String,
    pub vowels: Vec<String>,
    pub initial_consonants: Vec<String>,
    pub middle_consonants: Vec<String>,
    pub end_consonants: Vec<String>,
}

fn list_filter_chance(list: &Vec<String>, chance: f32) -> Vec<String> {
    list.iter()
        .filter_map(|v| {
            if rand::random::<f32>() < chance {
                Some(v.clone())
            } else {
                None
            }
        })
        .collect::<Vec<String>>()
}

fn map_string(list: Vec<&str>) -> Vec<String> {
    list.iter()
        .map(|v| String::from(*v))
        .collect::<Vec<String>>()
}

pub fn sample_list(list: &Vec<String>) -> String {
    thread_rng().sample(Slice::new(list).unwrap()).clone()
}

impl Language {
    pub fn new(id: LanguageId) -> Self {
        let vowel_chance = 0.75;
        let vowels = list_filter_chance(
            &map_string(vec![
                "a", "ae", "e", "i", "ei", "u", "o", "oi", "au", "ou", "ee", "ea", "oa",
            ]),
            0.75,
        );
        let consonants = list_filter_chance(
            &map_string(vec![
                "b", "c", "d", "f", "g", "h", "j", "k", "l", "m", "n", "p", "r", "s", "t", "v",
                "w", "z", "ss", "th", "st", "ch", "sh",
            ]),
            0.75,
        );

        let initial_consonants = list_filter_chance(&consonants, 0.50);
        let middle_consonants = list_filter_chance(&consonants, 0.75);
        let end_consonants = list_filter_chance(&consonants, 0.50);

        Self {
            id,
            name: "".to_owned(),
            vowels,
            initial_consonants,
            middle_consonants,
            end_consonants,
        }
    }

    pub fn maybe_vowel(&self, chance: f32) -> Option<String> {
        if rand::random::<f32>() < chance {
            Some(sample_list(&self.vowels))
        } else {
            None
        }
    }

    pub fn generate_name(&self, max_middle: usize) -> String {
        let mut name: String = String::new();
        name += &self.maybe_vowel(0.3).unwrap_or("".to_owned());
        name += &sample_list(&self.initial_consonants);
        for i in 0..thread_rng().sample(Uniform::new(0, max_middle)) {
            name += &sample_list(&self.vowels);
            name += &sample_list(&self.middle_consonants);
        }
        name += &sample_list(&self.vowels);
        name += &sample_list(&self.end_consonants);
        name += &self.maybe_vowel(0.3).unwrap_or("".to_owned());
        to_title_case(name.as_str())
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
    pub language: LanguageId,
    pub features: Vec<CultureFeature>,
}

#[iron_data]
pub struct Religion {
    pub id: ReligionId,
    pub name: String,
}
