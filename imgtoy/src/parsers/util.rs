use rand::Rng;
use serde_yaml::Value;

pub fn parse_property_as_f64_param(rng: &mut impl Rng, value: &Value, property_name: &str) -> Option<f64> {
    value.get(property_name).map(|prop| prop.as_f64().expect("[{property_name}] must be a float."))
}

pub fn parse_property_as_str_param(rng: &mut impl Rng, value: &Value, property_name: &str) -> Option<String> {
    value.get(property_name).map(|prop| prop.as_str().expect("[{property_name}] must be a string."))
        .map(|s| s.to_string())
}