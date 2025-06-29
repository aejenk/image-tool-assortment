use image_effects::dither::ordered::tools::exponentiate;
use rand::Rng;
use serde_yaml::Value;

use crate::{
    effects::{BaseResult, Log},
    parsers::{
        properties::process_chance,
        util::{parse_property_as_f64_complex, parse_property_as_u64_complex},
    },
};
// with props
pub fn parse_blur(log: Log, rng: &mut impl Rng, value: &Value) -> BaseResult<Option<u64>> {
    let blur = value.get("blur");
    if blur.is_none() {
        return Ok(None);
    }
    let blur = blur.unwrap();

    log.begin_category("blur")?;
    let enabled = process_chance(log, rng, blur)?;
    let factor = Ok(if enabled {
        Some(parse_u64_factor(log, rng, blur)?)
    } else {
        None
    });
    log.end_category()?;
    factor
}

pub fn parse_exponentiate(log: Log, rng: &mut impl Rng, value: &Value) -> BaseResult<Option<f64>> {
    let exponentiate = value.get("exponentiate");
    if exponentiate.is_none() {
        return Ok(None);
    }
    let exponentiate = exponentiate.unwrap();

    log.begin_category("exponentiate")?;
    let enabled = process_chance(log, rng, exponentiate)?;
    let factor = Ok(if enabled {
        Some(parse_f64_factor(log, rng, exponentiate)?)
    } else {
        None
    });
    log.end_category()?;
    factor
}

// simpler
pub fn parse_f64_factor(log: Log, rng: &mut impl Rng, value: &Value) -> BaseResult<f64> {
    Ok(parse_property_as_f64_complex(log, rng, value, "factor")?.expect("[factor] expected"))
}
pub fn parse_u64_factor(log: Log, rng: &mut impl Rng, value: &Value) -> BaseResult<u64> {
    Ok(parse_property_as_u64_complex(log, rng, value, "factor")?.expect("[factor] expected"))
}
