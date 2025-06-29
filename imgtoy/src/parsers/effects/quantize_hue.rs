use image_effects::filter::filters::QuantizeHue;
use rand::Rng;
use serde_yaml::Value;

use crate::{
    effects::{BaseResult, Log},
    parsers::util::parse_value_as_f64_sequence_complex,
};

pub fn parse_quantize_hue(log: Log, rng: &mut impl Rng, value: &Value) -> BaseResult<QuantizeHue> {
    log.begin_category("quantize-hue")?;

    let param = value.get("quantize-hue").unwrap();

    let hues = parse_value_as_f64_sequence_complex(log, rng, param, "hues")?
        .expect("[quantize-hue.hues] must be a list of hues/hue options.");

    let hues = hues.iter().map(|h| *h as f32).collect::<Vec<_>>();

    let hues_str = hues
        .iter()
        .enumerate()
        .map(|h| (h.0, format!("{:.3}", h.1)))
        .collect::<Vec<_>>();

    log.state_property("hues", format!("{hues_str:?}").as_str())?;

    log.end_category()?;

    Ok(QuantizeHue::with_hues(hues))
}
