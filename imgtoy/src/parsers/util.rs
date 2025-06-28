use rand::{seq::SliceRandom, Rng};
use serde_yaml::{Mapping, Value};

use crate::{
    effects::{BaseResult, Log},
    parsers::util::logless::{parse_f64_complex, parse_property_as_mapping, parse_u64_complex},
};

pub mod logless {
    use rand::{seq::SliceRandom, Rng};
    use serde_yaml::{Mapping, Value};

    pub fn parse_property_as_u64(
        value: &Value,
        property_name: &str,
        default: Option<u64>,
    ) -> Option<u64> {
        value
            .get(property_name)
            .map(|prop| prop.as_u64())
            .unwrap_or(default)
    }

    pub fn parse_property_as_i64(
        value: &Value,
        property_name: &str,
        default: Option<i64>,
    ) -> Option<i64> {
        value
            .get(property_name)
            .map(|prop| prop.as_i64())
            .unwrap_or(default)
    }

    pub fn parse_property_as_f64(
        value: &Value,
        property_name: &str,
        default: Option<f64>,
    ) -> Option<f64> {
        value
            .get(property_name)
            .map(|prop| prop.as_f64())
            .unwrap_or(default)
    }

    pub fn parse_u64_complex(rng: &mut impl Rng, param: &serde_yaml::Value) -> u64 {
        if let Some(exact) = param.as_u64() {
            exact
        } else if let Some(range) = param.as_mapping() {
            let min = range
                .get("min")
                .expect("expected [brighten.min] due to mapping - not present.")
                .as_u64()
                .expect("[brighten.min] must be a valid float - wasn't.");

            let max = range
                .get("max")
                .expect("expected [brighten.max] due to mapping - not present.")
                .as_u64()
                .expect("[brighten.max] must be a valid float - wasn't.");

            rng.gen_range(min..max)
        } else if let Some(options) = param.as_sequence() {
            let picked = options.choose(rng).unwrap();

            picked
                .as_u64()
                .expect("[brighten] options should be valid floats.")
        } else {
            todo!()
        }
    }

    pub fn parse_f64_complex(rng: &mut impl Rng, param: &serde_yaml::Value) -> f64 {
        if let Some(exact) = param.as_f64() {
            exact
        } else if let Some(range) = param.as_mapping() {
            let min = range
                .get("min")
                .expect("expected [brighten.min] due to mapping - not present.")
                .as_f64()
                .expect("[brighten.min] must be a valid float - wasn't.");

            let max = range
                .get("max")
                .expect("expected [brighten.max] due to mapping - not present.")
                .as_f64()
                .expect("[brighten.max] must be a valid float - wasn't.");

            rng.gen_range(min..max)
        } else if let Some(options) = param.as_sequence() {
            let picked = options.choose(rng).unwrap();

            picked
                .as_f64()
                .expect("[brighten] options should be valid floats.")
        } else {
            todo!()
        }
    }

    pub fn parse_property_as_mapping(value: &Value, property_name: &str) -> Option<Mapping> {
        value.get(property_name).map(|prop| match prop.clone() {
            Value::Mapping(mapping) => mapping,
            _ => panic!("[{property_name}] must be a mapping."),
        })
    }
}

pub fn parse_property_as_u64(
    log: Log,
    value: &Value,
    property_name: &str,
    default: Option<u64>,
) -> BaseResult<Option<u64>> {
    let u = value
        .get(property_name)
        .map(|prop| prop.as_u64())
        .unwrap_or(default);

    log.state_property(property_name, format!("{u:?}"))?;

    Ok(u)
}

pub fn parse_property_as_u64_complex(
    log: Log,
    rng: &mut impl Rng,
    value: &Value,
    property_name: &str,
) -> BaseResult<Option<u64>> {
    let u = value
        .get(property_name)
        .map(|prop| parse_u64_complex(rng, prop));

    log.state_property(property_name, format!("{u:?}"))?;

    Ok(u)
}

pub fn parse_property_as_f64(
    log: Log,
    value: &Value,
    property_name: &str,
    default: Option<f64>,
) -> BaseResult<Option<f64>> {
    let u = value
        .get(property_name)
        .map(|prop| prop.as_f64())
        .unwrap_or(default);

    log.state_property(property_name, format!("{u:?}"))?;

    Ok(u)
}

pub fn parse_property_as_f64_complex(
    log: Log,
    rng: &mut impl Rng,
    value: &Value,
    property_name: &str,
) -> BaseResult<Option<f64>> {
    let f = value
        .get(property_name)
        .map(|prop| logless::parse_f64_complex(rng, prop));

    log.state_property(property_name, format!("{f:?}"))?;

    Ok(f)
}

pub fn parse_value_as_f64_sequence_complex(
    log: Log,
    rng: &mut impl Rng,
    value: &Value,
    property_name: &str,
) -> BaseResult<Option<Vec<f64>>> {
    let sequence = value.get(property_name).map(|seq| {
        seq.as_sequence()
            .iter()
            .flat_map(|seq| {
                seq.iter()
                    .enumerate()
                    .map(|(i, n)| {
                        let f = parse_f64_complex(rng, n);

                        // TODO: resolve this thing
                        log.state_property(format!("#{i:03}"), f);
                        f
                    })
                    .collect::<Vec<f64>>()
            })
            .collect()
    });

    Ok(sequence)
}

pub fn parse_property_as_str(
    log: Log,
    value: &Value,
    property_name: &str,
) -> BaseResult<Option<String>> {
    let s = value
        .get(property_name)
        .map(|prop| {
            prop.as_str()
                .unwrap_or_else(|| panic!("[{property_name}] must be a string."))
        })
        .map(|s| s.to_string());

    log.state_property(property_name, format!("{s:?}"))?;

    Ok(s)
}

pub fn parse_property_as_f64_tuple_param(
    log: Log,
    rng: &mut impl Rng,
    value: &Value,
    property_name: &str,
    subprop_names: (&str, &str),
) -> BaseResult<(Option<f64>, Option<f64>)> {
    log.begin_category(property_name)?;
    let mapping = parse_property_as_mapping(value, property_name)
        .unwrap_or_else(|| panic!("[{property_name}] must be a mapping"));
    let mapping = Value::Mapping(mapping);

    let prop1 = parse_property_as_f64_complex(log, rng, &mapping, subprop_names.0)?;
    let prop2 = parse_property_as_f64_complex(log, rng, &mapping, subprop_names.1)?.or(prop1);
    log.end_category();

    Ok((prop1, prop2))
}
