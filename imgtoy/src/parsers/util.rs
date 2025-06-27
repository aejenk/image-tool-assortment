use rand::Rng;
use serde_yaml::{Mapping, Value};

use crate::parsers::{parse_f64_param, parse_u64_param};

pub fn parse_property_as_u64_param(rng: &mut impl Rng, value: &Value, property_name: &str) -> Option<u64> {
    value.get(property_name).map(|prop| parse_u64_param(rng, prop))
}

pub fn parse_property_as_f64_param(rng: &mut impl Rng, value: &Value, property_name: &str) -> Option<f64> {
    value.get(property_name).map(|prop| parse_f64_param(rng, prop))
}

pub fn parse_property_as_str_param(rng: &mut impl Rng, value: &Value, property_name: &str) -> Option<String> {
    value.get(property_name).map(|prop| prop.as_str().unwrap_or_else(|| panic!("[{property_name}] must be a string.")))
        .map(|s| s.to_string())
}

pub fn parse_property_as_mapping_param(rng: &mut impl Rng, value: &Value, property_name: &str) -> Option<Mapping> {
    value.get(property_name).map(|prop| {
        match prop.clone() {
            Value::Mapping(mapping) => mapping,
            _ => panic!("[{property_name}] must be a mapping."),
        }
    })
}

pub fn parse_property_as_f64_tuple_param(rng: &mut impl Rng, value: &Value, property_name: &str, subprop_names: (&str, &str)) -> (Option<f64>, Option<f64>) {
    let mapping = parse_property_as_mapping_param(rng, value, property_name).unwrap_or_else(|| panic!("[{property_name}] must be a mapping"));
    let mapping = Value::Mapping(mapping);

    let prop1 = parse_property_as_f64_param(rng, &mapping, subprop_names.0);
    let prop2 = parse_property_as_f64_param(rng, &mapping, subprop_names.1).or(prop1);

    (prop1, prop2)
}