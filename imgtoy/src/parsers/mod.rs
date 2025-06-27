use rand::{seq::SliceRandom, Rng};

pub mod palette;
pub mod ordered;
pub mod util;

// generic parsers
pub fn parse_u64_param(rng: &mut impl Rng, param: &serde_yaml::Value) -> u64 {
    if let Some(exact) = param.as_u64() {
        exact
    } else if let Some(range) = param.as_mapping() {
        let min = range.get("min")
            .expect("expected [brighten.min] due to mapping - not present.")
            .as_u64().expect("[brighten.min] must be a valid float - wasn't.");

        let max = range.get("max")
            .expect("expected [brighten.max] due to mapping - not present.")
            .as_u64().expect("[brighten.max] must be a valid float - wasn't.");

        rng.gen_range(min..max)
    } else if let Some(options) = param.as_sequence() {
        let picked = options.choose(rng).unwrap();
        

        picked.as_u64().expect("[brighten] options should be valid floats.")
    } else {
        todo!()
    }
}

pub fn parse_f64_param(rng: &mut impl Rng, param: &serde_yaml::Value) -> f64 {
    if let Some(exact) = param.as_f64() {
        exact
    } else if let Some(range) = param.as_mapping() {
        let min = range.get("min")
            .expect("expected [brighten.min] due to mapping - not present.")
            .as_f64().expect("[brighten.min] must be a valid float - wasn't.");

        let max = range.get("max")
            .expect("expected [brighten.max] due to mapping - not present.")
            .as_f64().expect("[brighten.max] must be a valid float - wasn't.");

        rng.gen_range(min..max)
    } else if let Some(options) = param.as_sequence() {
        let picked = options.choose(rng).unwrap();
        

        picked.as_f64().expect("[brighten] options should be valid floats.")
    } else {
        todo!()
    }
}