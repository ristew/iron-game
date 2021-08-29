use hecs::Entity;
use inflector::cases::titlecase::to_title_case;
use rand::{Rng, distributions::Slice, prelude::IteratorRandom, random, thread_rng};
use rand_distr::Uniform;

use crate::*;
use std::{cell::RefCell, collections::{HashMap, HashSet}, fmt::Debug, hash::Hash, rc::Rc, rc::Weak};

#[derive(Clone, Debug)]
pub struct MigrationStatus {
    pub migrating: isize,
    pub dest: Province,
    pub date: usize,
    pub settlement: Option<Settlement>,
}

#[iron_data]
pub struct Pop(pub Entity);
pub struct PopInfo {
    size: isize,
}
pub struct PopSettlement(pub Settlement);
pub struct PopCulture(pub Culture);
pub struct PopPolity(pub Polity);
pub struct FarmedGood(pub GoodType);

impl Pop {
    pub fn good_satiety(&self, good: GoodType) -> Satiety {
        good.base_satiety()
    }

    pub fn die(&mut self, amount: isize) -> isize {
        // println!("die pops: {}", amount);
        let before = self.size;
        self.size = (self.size - amount).max(0);

        // println!("before: {}, size: {}", before, self.size);
        before - self.size
    }

    pub fn settlement_site_threshold(&self) -> f32 {
        10.0
    }

    pub fn evaluate_site(&self, world: &World, site: &Site) -> f32 {
        let mut score = 20.0;
        for feature in site.features.iter() {
            score += match *feature {
                SettlementFeature::Hilltop => 10.0,
                SettlementFeature::Riverside => 10.0,
                SettlementFeature::Oceanside => 10.0,
                SettlementFeature::Harbor => 20.0,
                SettlementFeature::Mines(_) => 10.0,
                SettlementFeature::Fertile => 10.0,
                SettlementFeature::DominantCrop(_) => 0.0,
                SettlementFeature::Infertile => -10.0,
            };
        }
        // println!("evaluate site {:?} score {}", site, score);

        score
    }

    pub fn evaluate_sites(&self, world: &World, sites: Vec<Site>) -> Site {
        let mut max_site = &sites[0];
        let mut max_value = 0.0;
        for site in sites.iter() {
            let value = self.evaluate_site(world, site);
            if value > max_value {
                max_value = value;
                max_site = site;
            }
        }
        max_site.clone()
    }
}

#[macro_export]
macro_rules! id_comp {
	( $world:expr, $id:expr, $comp_t:ident) => {
		$world.hecs.get::<$comp_t>($id.0).unwrap()
	};
}

pub fn harvest(pop: Pop, world: &World) {
    // println!("harvest pop?");
    if let Some(farmed_good) = id_comp!(world, pop, FarmedGood) {
        let mut farmed_amount = id_comp!(world, pop, PopInfo).size as f32;
        let carrying_capacity = id_comp!(world, pop, PopSettlement).0.carrying_capacity(world);
        let comfortable_limit = carrying_capacity / 2.0;
        let pop_size = pop.get().settlement.get().population(world) as f32;
        if pop_size > comfortable_limit {
            // population pressure on available land, seek more
            world.add_command(Box::new(PopSeekMigrationCommand {
                pop: pop.clone(),
                pressure: (pop_size / comfortable_limit).powi(2),
            }))
        }
        if pop_size > carrying_capacity {
            farmed_amount = carrying_capacity + (farmed_amount - carrying_capacity).sqrt();
        }
        // if random::<f32>() > 0.9 {
        //     // println!("failed harvest! halving farmed goods");
        //     farmed_amount *= 0.7;
        // }
        world.add_command(Box::new(SetGoodsCommand {
            good_type: farmed_good,
            amount: farmed_amount * 300.0,
            pop: pop.clone(),
        }));
    }
}

pub struct Language {
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
    pub fn new() -> Self {
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
            id: None,
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

pub struct Culture(pub Entity);

impl Culture {
    pub fn language(&self, world: &World) -> &Language {
        world.hecs.get::<Language>(self).unwrap()
    }
}

pub struct Religion {
    pub name: String,
}

#[derive(Debug)]
pub struct KidBuffer(VecDeque<isize>);

impl KidBuffer {
    pub fn new() -> Self {
        Self(VecDeque::new())
    }

    pub fn size(&self) -> isize {
        self.0.iter().fold(0, |acc, e| acc + e)
    }

    pub fn spawn(&mut self, babies: isize) -> isize {
        // println!("spawn babies {}", babies);
        self.0.push_front(babies);
        // println!("{:?}", self);
        if self.0.len() > 12 {
            self.0.pop_back().unwrap()
        } else {
            babies
        }
    }

    pub fn starve(&mut self) -> isize {
        let cohort = sample(3.0).abs().min(12.0) as usize;
        if self.0.len() > cohort {
            let cohort_size = self.0[cohort];
            let dead_kids = positive_isample(cohort_size / 20 + 2, cohort_size / 5 + 1);
            // println!("cohort {}, size {}, dead {}", cohort, cohort_size, dead_kids);
            self.0[cohort] = (cohort_size - dead_kids).max(0);
            cohort_size - self.0[cohort]
        } else {
            0
        }
    }
}

#[derive(PartialEq)]
pub struct Satiety {
    pub base: f32,
    pub luxury: f32,
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

impl std::ops::Mul<Satiety> for f32 {
    type Output = Satiety;

    fn mul(self, rhs: Satiety) -> Self::Output {
        Satiety {
            base: rhs.base * self,
            luxury: rhs.luxury * self,
        }
    }
}
