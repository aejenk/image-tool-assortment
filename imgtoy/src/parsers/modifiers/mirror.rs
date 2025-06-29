use image_effects::dither::ordered::tools::mirror::{MirrorDirection, MirrorLine};
use rand::{seq::SliceRandom, Rng};
use serde_yaml::Value;

use crate::{
    effects::{BaseResult, Log},
    parsers::{properties::parse_chance, util::parse_property_as_f64_complex},
};

/// The option passed should be a *mapping* that represents a mirror option.
/// Such an option has multiple properties:
/// - chance: a float from 0.0 to 1.0 that represents the chance of application
/// - directions: a list of sets of directions to apply. one should be chosen at random.
///     - downright
///     - upright
///     - horizontal
///     - vertical
///
/// This will return a (f64, Vec<MirrorLine>) that represents the chance
/// alongside the Mirrorline.
pub fn parse_mirror(
    log: Log,
    rng: &mut impl Rng,
    value: &Value,
) -> BaseResult<(f64, Vec<MirrorLine>)> {
    let chance = parse_chance(log, value)?;

    log.begin_category("mirror-set")?;
    let mirror_set = parse_mirror_direction_set(log, rng, value)?;
    log.end_category()?;

    Ok((chance, mirror_set))
}

pub fn parse_mirror_direction_set(
    log: Log,
    rng: &mut impl Rng,
    value: &Value,
) -> BaseResult<Vec<MirrorLine>> {
    let directions = value
        .get("directions")
        .expect("[mirror.directions] must specify at least one direction.")
        .as_sequence()
        .expect("[mirror.directions] must be a list.");

    let flip_chance = parse_property_as_f64_complex(log, rng, value, "flip")?.unwrap_or(0.0);
    let thorough_chance =
        parse_property_as_f64_complex(log, rng, value, "thorough")?.unwrap_or(0.0);

    let directions = directions
        .iter()
        .map(|direction_set| {
            let direction_set = direction_set
                .as_sequence()
                .expect("[mirror.directions[$]] should be a sequence of strings.");
            direction_set
                .iter()
                .map(|entry| {
                    entry
                        .as_str()
                        .expect("[mirror.directions[$][$]] must be a string.")
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let directions = directions.choose(rng).unwrap();

    fn get_chance(rng: &mut impl Rng, chance: f64) -> bool {
        rng.gen_range(0.0..1.0) < chance
    }

    let mut mirror_set = vec![];

    for (i, direction_name) in directions.iter().enumerate() {
        let flip = get_chance(rng, flip_chance);
        let thorough = get_chance(rng, thorough_chance);
        let direction = match *direction_name {
            "downright" => MirrorLine {
                direction: MirrorDirection::Downright,
                flip,
                thorough,
            },
            "upright" => MirrorLine {
                direction: MirrorDirection::Upright,
                flip,
                thorough,
            },
            "horizontal" => MirrorLine {
                direction: MirrorDirection::Horizontal,
                flip,
                thorough,
            },
            "vertical" => MirrorLine {
                direction: MirrorDirection::Vertical,
                flip,
                thorough,
            },
            _ => {
                panic!(
                    "[mirror.directions[{i}].direction] found invalid direction [{direction_name}]"
                )
            }
        };

        let flip = direction.flip;
        let thorough = direction.thorough;

        log.state_property(
            format!("#{i:03}"),
            match direction.direction {
                MirrorDirection::Downright => {
                    format!("downright, flip={flip}, thorough={thorough}")
                }
                MirrorDirection::Upright => format!("upright, flip={flip}, thorough={thorough}"),
                MirrorDirection::Horizontal => {
                    format!("horizontal, flip={flip}, thorough={thorough}")
                }
                MirrorDirection::Vertical => format!("vertical, flip={flip}, thorough={thorough}"),
            },
        )?;

        mirror_set.push(direction);
    }

    Ok(mirror_set)
}
