use std::{collections::HashMap, sync::Arc};
use std::fmt::Debug;
use std::hash::Hash;
use dashmap::DashMap;
use dashmap::mapref::one::RefMut;
use parking_lot::{Mutex, RwLock};
use serde::{Serialize, Deserialize};

use crate::*;


pub trait FactorSubject: Clone + Eq + Hash + Debug {
}

pub trait FactorField: Clone + Eq + Hash + Debug {
}

// #[derive(Copy, Clone, Debug)]
// pub struct F32Hash(f32);
// impl Hash for F32Hash {
//     fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//         self.0.as_bytes().hash(state);
//     }
// }

// impl PartialEq for F32Hash {
//     fn eq(&self, other: &Self) -> bool {
//         self.0 == other.0
//     }
// }

// impl Eq for F32Hash {
//     fn assert_receiver_is_total_eq(&self) {}
// }

// impl Into<f32> for F32Hash {
//     fn into(self) -> f32 {
//         self.0
//     }
// }

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FormulaId(usize);


pub enum FormulaFn {
    VecArgs(Arc<dyn Fn(Vec<f32>) -> f32 + Send + Sync>),
    OneArgs(Arc<dyn Fn(f32) -> f32 + Send + Sync>),
    TwoArgs(Arc<dyn Fn(f32, f32) -> f32 + Send + Sync>),
    ThreeArgs(Arc<dyn Fn(f32, f32, f32) -> f32 + Send + Sync>),
}

impl<F> From<F> for FormulaFn where F: Fn(f32) -> f32 + Send + Sync + 'static {
    fn from(f: F) -> Self {
        Self::OneArgs(Arc::new(f))
    }
}

impl FormulaFn {
    pub fn new_one<F>(inner: F) -> Self where F: Fn(f32) -> f32 + Send + Sync + 'static {
        Self::OneArgs(Arc::new(inner))
    }

    pub fn new_two<F>(inner: F) -> Self where F: Fn(f32, f32) -> f32 + Send + Sync + 'static {
        Self::TwoArgs(Arc::new(inner))
    }

    pub fn new_three<F>(inner: F) -> Self where F: Fn(f32, f32, f32) -> f32 + Send + Sync + 'static {
        Self::ThreeArgs(Arc::new(inner))
    }

    pub fn run(&self, args: Vec<f32>) -> f32 {
        match self {
            FormulaFn::VecArgs(f) => (*f)(args),
            FormulaFn::OneArgs(f) => (*f)(args[0]),
            FormulaFn::TwoArgs(f) => (*f)(args[0], args[1]),
            FormulaFn::ThreeArgs(f) => (*f)(args[0], args[1], args[2]),
        }
    }
}

pub struct Formula {
    pub inputs: Vec<FactorType>,
    pub inner_fn: FormulaFn,
}

impl Formula {
    pub fn new(inputs: Vec<FactorType>, inner_fn: FormulaFn) -> Self {
        Self {
            inputs,
            inner_fn,
        }
    }

    pub fn calc(&self, args: Vec<f32>) -> f32 {
        self.inner_fn.run(args)
    }
}

pub struct FormulaValue {
    pub cached: f32,
    pub dirty: bool,
}

pub struct FormulaSystem {
    factors: DashMap<FactorType, Factor>,
    formulae: Vec<Formula>,
    input_map: HashMap<FactorType, Vec<FormulaId>>,
    formula_values: DashMap<FormulaId, FormulaValue>,
    formula_subjects: DashMap<FormulaId, FactorType>,
}

// TODO: don't propogate onto end nodes
impl FormulaSystem {
    pub fn add_factor(&self, f: &FactorType, amount: f32) {
        self.factors.get_mut(f).map(|mut factor| {
            factor.value_mut().level += amount;
        });
        self.propogate_changes(f);
    }

    pub fn set_factor(&self, f: &FactorType, amount: f32) {
        self.factors.get_mut(f).map(|mut factor| {
            factor.value_mut().level = amount;
        });
        self.propogate_changes(f);
    }

    pub fn insert_factor(&self, f: &FactorType, amount: f32) {
        self.factors.insert(f.clone(), Factor::new_amount(amount));
    }

    pub fn get_factor(&self, f: &FactorType) -> f32 {
        self.factors.get(f).map(|factor| {
            let f = factor.value();
            if let Some(formula_id) = f.formula {
                self.formula_value(formula_id);
            }
        }).unwrap_or(0.0)
    }

    pub fn get_formula(&self, f: &FactorType) -> FormulaId {
        let factor = self.factors.get(f).unwrap();
        match factor.value() {
            Factor::Formula(formula_id) => *formula_id,
            _ => panic!("get formula on not formula {:?}", f),
        }
    }

    // retrieve formulae that change as a result of f changing
    pub fn get_formulae(&self, f: &FactorType) -> Vec<FormulaId> {
        self
            .input_map
            .get(f)
            .map(|fs|
                 fs.iter()
                 .map(|fid| *fid)
                 .collect::<Vec<_>>()
            ).unwrap_or(Vec::new())
    }

    // given that f changed, update values of all descendant formulae
    fn propogate_changes(&self, f: &FactorType) {
        for &formula_id in self.get_formulae(f).iter() {
            // println!("update formula {:?}", formula_id);
            let formula = &self.formulae[formula_id.0];
            // only really calc if there are more down the line, otherwise mark dirty
            if self.input_map.get(&formula.subject).map(|v| v.len()).unwrap_or(0) > 0 {
                if true {
                    self.dirty_formula(formula_id);
                } else {
                    let before = self.formula_value(formula_id);
                    self.calc_formula(formula_id);
                    let after = self.formula_value(formula_id);
                }
                // only recalc if value actually changed (highly likely)
                self.propogate_changes(&formula.subject);
            } else {
                self.dirty_formula(formula_id);
            }
        }
    }

    fn formula_value(&self, formula_id: FormulaId) -> f32 {
        {
            if let Some(val) = self.formula_values.get(&formula_id) {
                if !val.dirty {
                    println!("cached formula {:?} {}", formula_id, val.cached);
                    return val.cached
                }
            } else {
                println!("BAD: Formula without value! {:?}", formula_id);
                return 0.0;
            }
        }
        {
            let mut val = self.formula_values.get_mut(&formula_id).unwrap();
            val.cached = self.calc_formula(formula_id);
            val.dirty = false;
            val.cached
        }
    }

    fn fetch_inputs(&self, inputs: &Vec<FactorType>) -> Vec<f32> {
        let mut res = Vec::new();
        for input in inputs.iter() {
            res.push(self.get_factor(input));
        }
        res
    }

    fn dirty_formula(&self, formula_id: FormulaId) {
        if let Some(mut val) = self.formula_values.get_mut(&formula_id) {
            val.value_mut().dirty = true;
        }
    }

    fn calc_formula(&self, formula_id: FormulaId) -> f32 {
        let formula = &self.formulae[formula_id.0];
        let value = formula.calc(self.fetch_inputs(&formula.inputs));
        value
    }

    fn add_input(&mut self, f: &FactorType, formula_id: FormulaId) {
        self
            .input_map
            .entry(f.clone())
            .or_default()
            .push(formula_id);
    }

    pub fn add_formula(&mut self, subject: &FactorType, formula: Formula) -> FormulaId {
        let idx = self.formulae.len();
        let formula_id = FormulaId(idx);
        for input in formula.inputs.iter() {
            self.add_input(input, formula_id);
        }
        self.formula_values.insert(formula_id, FormulaValue {
            cached: 0.0,
            dirty: true,
        });
        self.factors.insert(subject.clone(), Factor::new_formula(formula_id));
        self.formulae.push(formula);
        formula_id
    }
}

impl Default for FormulaSystem {
    fn default() -> Self {
        Self { factors: Default::default(), formulae: Default::default(), input_map: Default::default(), formula_values: Default::default() }
    }
}
