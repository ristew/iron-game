use crate::*;

#[derive(Debug, Copy, Clone)]
pub enum Sex {
    Male,
    Female,
}

#[iron_data]
pub struct Character {
    pub id: Option<CharacterId>,
    pub name: String,
    pub birthday: Date,
    pub sex: Sex,
    pub health: f32,
}

pub trait Agent {
    fn decide();
}

pub struct AiAgent {

}
