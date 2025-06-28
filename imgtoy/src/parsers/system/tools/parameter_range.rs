use rand::{distributions::uniform::SampleUniform, Rng};
use serde_yaml::Value;

use crate::{
    effects::Log,
    parsers::util::{parse_property_as_f64, parse_property_as_i64, parse_property_as_u64},
};

/// Represents a parameter range.
///
/// Mostly used in the ComplexPrimitive<T> but can be used elsewhere.
///
/// Supports generation of a random value within the range.
pub struct ParameterRange<T: Clone + SampleUniform + PartialOrd> {
    min: T,
    max: T,
}

impl<T: Clone + SampleUniform + PartialOrd> ParameterRange<T> {
    pub fn parse_as_u64(log: Log, value: &Value) -> ParameterRange<u64> {
        let min = parse_property_as_u64(log, &value, "min", None)
            .unwrap()
            .expect("[.min] expected as u64");
        let max = parse_property_as_u64(log, &value, "max", None)
            .unwrap()
            .expect("[.max] expected as u64");

        ParameterRange { min, max }
    }

    pub fn parse_as_i64(log: Log, value: &Value) -> ParameterRange<i64> {
        let min = parse_property_as_i64(log, &value, "min", None)
            .unwrap()
            .expect("[.min] expected as i64");
        let max = parse_property_as_i64(log, &value, "max", None)
            .unwrap()
            .expect("[.max] expected as i64");

        ParameterRange { min, max }
    }

    pub fn parse_as_f64(log: Log, value: &Value) -> ParameterRange<f64> {
        let min = parse_property_as_f64(log, &value, "min", None)
            .unwrap()
            .expect("[.min] expected as u64");
        let max = parse_property_as_f64(log, &value, "max", None)
            .unwrap()
            .expect("[.max] expected as u64");

        ParameterRange { min, max }
    }

    pub fn get(self, rng: &mut impl Rng) -> T {
        rng.gen_range(self.min..self.max)
    }
}
