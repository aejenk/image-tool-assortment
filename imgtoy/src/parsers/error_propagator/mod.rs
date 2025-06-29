use image_effects::dither::{
    error::{ErrorPropagator, WithPalette},
    ATKINSON, BURKES, FLOYD_STEINBERG, JARVIS_JUDICE_NINKE, SIERRA, SIERRA_LITE, SIERRA_TWO_ROW,
    STUCKI,
};
use rand::Rng;
use serde_yaml::Value;

use crate::{
    effects::{BaseResult, Log},
    parsers::palette::parse_palette,
};

pub fn parse_error_propagator<'a, 'b>(
    log: Log,
    rng: &mut impl Rng,
    effect: &Value,
    algorithm_name: &str,
) -> BaseResult<ErrorPropagator<'a, 'b, WithPalette>> {
    let config = effect.get(algorithm_name).unwrap();

    let propagator = match algorithm_name.to_lowercase().as_str() {
        "floydsteinberg" | "floyd-steinberg" | "floyd_steinberg" => FLOYD_STEINBERG,
        "jarvisjudiceninke" | "jarvis-judice-ninke" | "jarvis_judice_ninke" => JARVIS_JUDICE_NINKE,
        "atkinson" => ATKINSON,
        "burkes" => BURKES,
        "stucki" => STUCKI,
        "sierra" => SIERRA,
        "sierra-two-row" | "sierra_two_row" => SIERRA_TWO_ROW,
        "sierra-lite" | "sierra_to_row" => SIERRA_LITE,
        _ => panic!("{algorithm_name} is not a supported effect."),
    };

    let palette = config
        .get("palette")
        .unwrap_or_else(|| panic!("[{algorithm_name}] requires a [palette] to be set."));

    let palette = parse_palette(log, rng, palette)?;

    Ok(propagator.with_palette(palette))
}
