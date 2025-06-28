use rand::{distributions::uniform::SampleUniform, seq::SliceRandom, Rng};
use serde_yaml::Value;

use crate::{effects::Log, parsers::system::tools::parameter_range::ParameterRange};

/// Represents a complex primitive - meaning it can either be the exact value, a list, or a range.
///
/// Once `.get()` is called, it's consumed to generate *one* instance.
pub enum ComplexPrimitive<T: Clone + SampleUniform + PartialOrd> {
    Exact(T),
    Choice(Vec<T>),
    Range(ParameterRange<T>),
}

impl<T: Clone + SampleUniform + PartialOrd> ComplexPrimitive<T> {
    pub fn for_u64(log: Log, value: &Value) -> ComplexPrimitive<u64> {
        if let Some(exact) = value.as_u64() {
            ComplexPrimitive::Exact(exact)
        } else if let Some(options) = value.as_sequence() {
            let options = options
                .iter()
                .map(|option| option.as_u64().expect("expected list of u64"))
                .collect::<Vec<_>>();

            ComplexPrimitive::Choice(options)
        } else if value.is_mapping() {
            ComplexPrimitive::Range(ParameterRange::<u64>::parse_as_u64(log, &value))
        } else {
            panic!("expected COMPLEX PRIMITIVE - found unexpected type.");
        }
    }

    pub fn for_i64(log: Log, value: &Value) -> ComplexPrimitive<i64> {
        if let Some(exact) = value.as_i64() {
            ComplexPrimitive::Exact(exact)
        } else if let Some(options) = value.as_sequence() {
            let options = options
                .iter()
                .map(|option| option.as_i64().expect("expected list of u64"))
                .collect::<Vec<_>>();

            ComplexPrimitive::Choice(options)
        } else if value.is_mapping() {
            ComplexPrimitive::Range(ParameterRange::<u64>::parse_as_i64(log, &value))
        } else {
            panic!("expected COMPLEX PRIMITIVE - found unexpected type.");
        }
    }

    pub fn for_f64(log: Log, value: &Value) -> ComplexPrimitive<f64> {
        if let Some(exact) = value.as_f64() {
            ComplexPrimitive::Exact(exact)
        } else if let Some(options) = value.as_sequence() {
            let options = options
                .iter()
                .map(|option| option.as_f64().expect("expected list of u64"))
                .collect::<Vec<_>>();

            ComplexPrimitive::Choice(options)
        } else if value.is_mapping() {
            ComplexPrimitive::Range(ParameterRange::<f64>::parse_as_f64(log, &value))
        } else {
            panic!("expected COMPLEX PRIMITIVE - found unexpected type.");
        }
    }

    pub fn get(self, rng: &mut impl Rng) -> T {
        match self {
            ComplexPrimitive::Exact(exact) => exact,
            ComplexPrimitive::Choice(choices) => choices.choose(rng).unwrap().clone(),
            ComplexPrimitive::Range(range) => range.get(rng),
        }
    }
}
