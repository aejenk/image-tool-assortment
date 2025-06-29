use image_effects::dither::ordered::tools::properties::{CheckerType, Factor, Source};
use rand::Rng;
use serde_yaml::Value;

use crate::{
    effects::{BaseResult, Log},
    parsers::{
        modifiers::simple::parse_u64_factor,
        properties::process_chance,
        util::{
            parse_property_as_f64, parse_property_as_str,
            parse_property_as_u64_complex, parse_property_as_u64_tuple_param,
        },
    },
};

fn parse_source(log: Log, rng: &mut impl Rng, value: &Value) -> BaseResult<Source> {
    let source = value.get("source").expect("expected [source]");

    let source_type = parse_property_as_str(log, source, "type")?.expect("expected [type]");

    Ok(match source_type.as_str() {
        "center" => Source::Center,
        "fixed" => {
            let (y, x) = parse_property_as_u64_tuple_param(log, rng, value, "fixed", ("y", "x"))?;

            Source::Fixed(
                y.expect("[y] expected") as usize,
                x.expect("[x] expected") as usize,
            )
        }
        _ => panic!("something's wrong with the source"),
    })
}

fn parse_factor(log: Log, rng: &mut impl Rng, value: &Value) -> BaseResult<Factor> {
    let factor = value.get("factor").expect("expected [factor]");

    let factor_type = parse_property_as_str(log, factor, "type")?.expect("expected [type]");

    Ok(match factor_type.as_str() {
        "linear" => Factor::Linear,
        "exponential" => Factor::Exponential(
            parse_property_as_f64(log, value, "factor", Some(0.95))?.expect("expected factor"),
        ),
        _ => panic!("somethin went wrong when parsin the factor"),
    })
}

pub fn parse_checker(
    log: Log,
    rng: &mut impl Rng,
    value: &Value,
) -> BaseResult<Option<CheckerType>> {
    log.begin_category("checker")?;
    let checker = value.get("checker").expect("expected [checker]");

    let enabled = process_chance(log, rng, checker)?;

    if !enabled {
        log.end_category()?;
        return Ok(None);
    }

    let checker_type = parse_property_as_str(log, checker, "type")?.expect("expected [type]");

    let result = Ok(Some(match checker_type.as_str() {
        "iter" => CheckerType::Iter(parse_u64_factor(log, rng, value)? as usize),
        "from" => {
            let source = parse_source(log, rng, checker)?;
            let factor = parse_factor(log, rng, checker)?;
            let modulo =
                parse_property_as_u64_complex(log, rng, checker, "modulo")?.map(|v| v as usize);

            CheckerType::From {
                source,
                factor,
                modulo,
            }
        }
        _ => panic!("somethin went wrong with the checker type bud"),
    }));

    log.end_category()?;
    result
}
