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
pub enum FormulaName {
    PopMigrationPull(PopId),
}


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

pub struct Formula<S, F> where S: FactorSubject, F: FactorField {
    pub inputs: Vec<(S, F)>,
    pub inner_fn: FormulaFn,
    pub subject: (S, F),
}

impl<S, F> Formula<S, F> where S: FactorSubject, F: FactorField {
    pub fn new(inputs: Vec<(S, F)>, inner_fn: FormulaFn, subject: (S, F)) -> Self {
        Self {
            inputs,
            inner_fn,
            subject,
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

pub struct FormulaSystem<S, F> where S: FactorSubject, F: FactorField {
    factors: DashMap<(S, F), Factor>,
    formulae: Vec<Formula<S, F>>,
    input_map: HashMap<(S, F), Vec<FormulaName>>,
    formula_values: DashMap<FormulaName, FormulaValue>,
}

// TODO: don't propogate onto end nodes
impl<S, F> FormulaSystem<S, F> where S: FactorSubject, F: FactorField {
    pub fn add_factor(&self, f: &(S, F), amount: f32) {
        self.factors.get_mut(f).map(|mut factor| {
            match factor.value_mut() {
                Factor::Constant(n) => *n += amount,
                Factor::Decay(n, _) => *n += amount,
                Factor::Formula(_) => println!("add to factor {:?}", f),
            }
        });
        self.propogate_changes(f);
    }

    pub fn set_factor(&self, f: &(S, F), amount: f32) {
        self.factors.get_mut(f).map(|mut factor| {
            match factor.value_mut() {
                Factor::Constant(n) => *n = amount,
                Factor::Decay(n, _) => *n = amount,
                Factor::Formula(_) => println!("add to factor {:?}", f),
            }
        });
        self.propogate_changes(f);
    }

    pub fn insert_factor(&self, f: &(S, F), amount: f32) {
        self.factors.insert(f.clone(), Factor::Constant(amount));
    }

    pub fn get_factor(&self, f: &(S, F)) -> f32 {
        self.factors.get(f).map(|factor| {
            match factor.value() {
                Factor::Constant(n) => *n,
                Factor::Decay(n, _) => *n,
                Factor::Formula(formula_id) => self.formula_value(*formula_id),
            }
        }).unwrap_or(0.0)
    }

    pub fn get_formula(&self, f: &(S, F)) -> FormulaName {
        let factor = self.factors.get(f).unwrap();
        match factor.value() {
            Factor::Formula(formula_id) => *formula_id,
            _ => panic!("get formula on not formula {:?}", f),
        }
    }

    // retrieve formulae that change as a result of f changing
    pub fn get_formulae(&self, f: &(S, F)) -> Vec<FormulaName> {
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
    fn propogate_changes(&self, f: &(S, F)) {
        for &formula_id in self.get_formulae(f).iter() {
            // println!("update formula {:?}", formula_id);
            let formula = &self.formulae[formula_id.0];
            // only really calc if there are more down the line, otherwise mark dirty
            if self.input_map.get(&formula.subject).map(|v| v.len()).unwrap_or(0) > 0 {
                let before = self.formula_value(formula_id);
                self.calc_formula(formula_id);
                let after = self.formula_value(formula_id);
                // only recalc if value actually changed (highly likely)
                if before != after {
                    self.propogate_changes(&formula.subject);
                }
            } else {
                self.dirty_formula(formula_id);
            }
        }
    }

    fn formula_value(&self, formula_id: FormulaName) -> f32 {
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

    fn fetch_inputs(&self, inputs: &Vec<(S, F)>) -> Vec<f32> {
        let mut res = Vec::new();
        for input in inputs.iter() {
            res.push(self.get_factor(input));
        }
        res
    }

    fn dirty_formula(&self, formula_id: FormulaName) {
        if let Some(mut val) = self.formula_values.get_mut(&formula_id) {
            val.value_mut().dirty = true;
        }
    }

    fn calc_formula(&self, formula_id: FormulaName) -> f32 {
        let formula = &self.formulae[formula_id.0];
        let value = formula.calc(self.fetch_inputs(&formula.inputs));
        value
    }

    fn add_input(&mut self, f: &(S, F), formula_id: FormulaName) {
        self
            .input_map
            .entry(f.clone())
            .or_default()
            .push(formula_id);
    }

    pub fn add_formula(&mut self, formula: Formula<S, F>) -> FormulaName {
        let idx = self.formulae.len();
        let formula_id = FormulaName(idx);
        for input in formula.inputs.iter() {
            self.add_input(input, formula_id);
        }
        self.formula_values.insert(formula_id, FormulaValue {
            cached: 0.0,
            dirty: true,
        });
        self.factors.insert(formula.subject.clone(), Factor::Formula(formula_id));
        self.formulae.push(formula);
        formula_id
    }
}

impl<S, F> Default for FormulaSystem<S, F> where S: FactorSubject, F: FactorField {
    fn default() -> Self {
        Self { factors: Default::default(), formulae: Default::default(), input_map: Default::default(), formula_values: Default::default() }
    }
}
