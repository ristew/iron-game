use std::{collections::HashSet, mem::MaybeUninit};

use crate::*;

#[derive(Debug, Copy, Clone)]
pub enum Sex {
    Male,
    Female,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum CharacterFeature {
    Coward,
    Idiot,
}

pub enum Title {
    PolityLeader(PolityId),
    SettlementLeader(SettlementId),
}

#[derive(IronData)]
pub struct Character {
    pub id: usize,
    pub name: String,
    pub birthday: Date,
    pub sex: Sex,
    pub health: f32,
    pub death: Option<Date>,
    pub features: HashSet<CharacterFeature>,
    pub titles: Vec<Title>,
}

gen_id!(Character, CharacterId);

impl Featured<CharacterFeature> for Character {
    fn has_feature(&self, feature: CharacterFeature) -> bool {
        self.features.contains(&feature)
    }

    fn add_feature(&mut self, feature: CharacterFeature) {
        self.features.insert(feature);
    }

    fn remove_feature(&mut self, feature: CharacterFeature) {
        self.features.remove(&feature);
    }
}

impl Character {
    pub fn title(&self, world: &World) -> String {
        format!("{}, {}", self.name, self.birthday.age(world.date))
    }
}

pub trait Agent {
    fn decide();
}

pub struct AiAgent {

}
