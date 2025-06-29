use image_effects::dither::ordered::algorithms::properties::Wrapping;
use rand::Rng;
use serde_yaml::Value;

use crate::effects::{BaseResult, Log};

pub fn parse_wrapping_set(log: Log, _: &mut impl Rng, value: &Value) -> BaseResult<Vec<Wrapping>> {
    let wrappings = value
        .get("wrappings")
        .expect("[wrappings] must specify at least one direction.")
        .as_sequence()
        .expect("[wrappings] must be a list.");

    let wrappings = wrappings
        .iter()
        .map(|wrapping| wrapping.as_str().expect("[wrappings[$]] must be a string."))
        .collect::<Vec<_>>();

    log.state_property("wrappings", format!("{wrappings:?}"))?;

    let mut parsed_wrappings = vec![];

    for wrapping in wrappings {
        parsed_wrappings.push(match wrapping {
            "horizontal" => Wrapping::Horizontal,
            "vertical" => Wrapping::Vertical,
            "all" => Wrapping::All,
            "none" => Wrapping::None,
            _ => panic!("[wrappings[$]] found invalid wrapping [{wrapping}]"),
        });
    }

    Ok(parsed_wrappings)
}
