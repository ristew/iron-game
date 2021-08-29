use std::collections::HashSet;

use hecs::{Entity, EntityRef};

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

pub struct Character(Entity);

pub struct CharacterInfo {
    pub birthday: Date,
    pub sex: Sex,
    pub death: Option<Date>,
}

pub struct Health(pub f32);

pub fn title(world: &World, character_ref: CharacterRef) -> String {
    let character_info = character_ref.get::<Character>();
    format!("{}, {}", character_info.name, character_info.birthday.age(world.date))
}

pub trait Agent {
    fn decide();
}

pub struct AiAgent {

}
