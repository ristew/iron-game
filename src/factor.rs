use rand::{Rng, distributions::Slice, prelude::SliceRandom, random, thread_rng};
use rand_distr::Uniform;
use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::fmt::Debug;
use crate::*;
use serde::{Serialize, Deserialize};

pub enum FactorEffectLabel {

}

pub enum FactorEffect {
    Bonus(f32),
    BaseFactor(f32),
    TotalFactor(f32),
}


//TODO: split out into PopFactor eg like FactorRef
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum FactorType {
    SettlementSize,
    SettlementCarryingCapacity,
    SettlementPressure,

    PopDemand(GoodType),
    PopPressure,
    PopSize,
    PopHarvest,
}

impl FactorField for FactorType {}

#[derive(Serialize, Deserialize)]
pub enum FactorDecay {
    Linear(f32),
    Exponential(f32),
    None,
}


impl<T> From<T> for GameId where T: IronId {
    fn from(r: T) -> Self {
        r.gid()
    }
}

pub trait GameIdVecProvider {
    fn gids(&self) -> Vec<GameId>;
}

impl<T> GameIdVecProvider for T where T: IronId {
    fn gids(&self) -> Vec<GameId> {
        vec![self.gid()]
    }
}


#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum GameId {
    Pop(usize),
    Language(usize),
    Polity(usize),
    Province(usize),
    Culture(usize),
    Settlement(usize),
    Character(usize),
    Religion(usize),
}

impl FactorSubject for GameId {

}

// pub enum GameFact {
//     PopSize(PopRef),
//     SettlementSize(SettlementRef),
// }

// impl GameFact {

// }

// pub trait GameFetch {
//     type Out;

//     fn get(&self, g: impl GameRef) -> &Self::Out;
// }

// impl GameFetch for Query<>;

// pub trait GameFact {
//     fn fetch<S>(&self, storage: S) -> &S::Out where S: GameFetch + Sized;
// }

// pub struct PopSize;

// impl GameFact for PopSize {
//     fn fetch<S>(&self, storage: S) -> &S::Out where S: GameFetch {
//         storage.get()
//     }
// }

#[derive(Serialize, Deserialize)]
pub struct Factor {
    level: f32,
    decay: Option<FactorDecay>,
    formula: Option<FormulaId>,
}

impl Factor {
    pub fn decay(&mut self) -> f32 {
        if let Some(decay) = self.decay {
            match decay {
                FactorDecay::Linear(n) => {
                    if self.level > 0.0 {
                        self.level = (self.level - n).max(0.0);
                    }
                },
                FactorDecay::Exponential(n) => {
                    if self.level > 0.0 {
                        self.level = self.level * (1.0 - n);
                    }
                },
                FactorDecay::None => todo!(),
            };
        }
    }
}
