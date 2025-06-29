use image_effects::dither::ordered::tools::properties::Rotation;
use rand::{seq::SliceRandom, Rng};
use serde_yaml::Value;

use crate::{
    effects::{BaseResult, Log},
    parsers::properties::process_chance,
};

pub fn parse_rotation(log: Log, rng: &mut impl Rng, value: &Value) -> BaseResult<Option<Rotation>> {
    let rotation = value.get("rotation");
    if rotation.is_none() {
        return Ok(None);
    }
    let rotation = rotation.unwrap();

    log.begin_category("rotation")?;
    let enabled = process_chance(log, rng, rotation)?;

    let result = Ok(if enabled {
        let rotation = rotation
            .get("values")
            .expect("expected [values]")
            .as_sequence()
            .expect("expected rot-seq")
            .iter()
            .map(|prop| prop.as_str().expect("expected str"))
            .collect::<Vec<_>>();

        Some(match *rotation.choose(rng).unwrap() {
            "right" => Rotation::Right,
            "half" => Rotation::Half,
            "left" => Rotation::Left,
            "none" => Rotation::None,
            _ => panic!("whoops"),
        })
    } else {
        None
    });

    log.end_category()?;

    result
}
