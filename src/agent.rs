use std::{collections::HashSet, mem::MaybeUninit};
use serde::{Serialize, Deserialize};

use crate::*;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Sex {
    Male,
    Female,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum CharacterFeature {
    Coward,
    Idiot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Title {
    PolityLeader(PolityId),
    SettlementLeader(SettlementId),
}

impl Title {
    pub fn name(&self) -> String {
        match self {
            Title::PolityLeader(polity_id) => {
                let polity = polity_id.get();
                format!("{} of {}", polity.level.leader_title(), polity.name)
            },
            Title::SettlementLeader(settlement_id) => {
                let settlement = settlement_id.get();
                format!("Mayor of {}", settlement.name)
            },
        }
    }
}

#[derive(IronData, Serialize, Deserialize)]
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
        let titles = self.titles.iter().map(|t| t.name()).collect::<Vec<_>>();
        let titles_str = if titles.len() > 0 {
            format!(", {},", titles.join(", "))
        } else {
            "".to_owned()
        };
        // no reanimation!!
        format!("{}{}", self.name, titles_str)
    }
}

pub trait Agent {
    fn decide();
}

pub struct AiAgent {

}
