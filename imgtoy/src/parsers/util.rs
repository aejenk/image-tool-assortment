use rand::Rng;
use serde_yaml::Value;

use crate::parsers::{parse_f64_param, parse_u64_param};

pub fn parse_property_as_u64_param(rng: &mut impl Rng, value: &Value, property_name: &str) -> Option<u64> {
    value.get(property_name).map(|prop| parse_u64_param(rng, prop))
}

pub fn parse_property_as_f64_param(rng: &mut impl Rng, value: &Value, property_name: &str) -> Option<f64> {
    value.get(property_name).map(|prop| parse_f64_param(rng, prop))
}

pub fn parse_property_as_str_param(rng: &mut impl Rng, value: &Value, property_name: &str) -> Option<String> {
    value.get(property_name).map(|prop| prop.as_str().expect("[{property_name}] must be a string."))
        .map(|s| s.to_string())
}