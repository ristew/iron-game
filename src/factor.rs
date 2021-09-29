use rand::{Rng, distributions::Slice, prelude::SliceRandom, random, thread_rng};
use rand_distr::Uniform;
use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::fmt::Debug;
use crate::*;

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

pub enum FactorDecay {
    Linear(f32),
    Exponential(f32),
    None,
}



#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum FactorRef {
    Pop(PopId),
    Language(LanguageId),
    Polity(PolityId),
    Province(ProvinceId),
    Culture(CultureId),
    Settlement(SettlementId),
    Character(CharacterId),
    Religion(ReligionId),
}

impl FactorSubject for FactorRef {

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

pub enum Factor {
    Constant(f32),
    Decay(f32, FactorDecay),
    Formula(FormulaId),
}

impl Factor {
    pub fn decay(&mut self) -> f32 {
        match self {
            Factor::Decay(amount, decay) => {
                let this_decay = match decay {
                    FactorDecay::Linear(n) => *n,
                    FactorDecay::Exponential(ref n) => *amount * n,
                    FactorDecay::None => 0.0,
                };
                *amount = (*amount - this_decay).max(0.0);
                this_decay
            },
            _ => 0.0,
        }
    }
}
