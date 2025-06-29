use image_effects::filter::filters::GradientMap;
use palette::{IntoColor, Lch, Srgb};
use rand::Rng;
use serde_yaml::Value;

use crate::{
    effects::{BaseResult, Log},
    parsers::{
        palette::gen_with_lightness,
        util::{parse_property_as_f64_complex, parse_property_as_u64_complex},
    },
};

pub fn parse_gradient_map(log: Log, rng: &mut impl Rng, value: &Value) -> BaseResult<GradientMap> {
    log.begin_category("gradient-map")?;

    let param = value.get("gradient-map").unwrap();

    let effect = Ok(if param.is_mapping() {
        let amnt = parse_property_as_u64_complex(log, rng, param, "amnt")?
            .expect("[gradient-map] as a mapping needs an [amnt] property.");
        let noise = parse_property_as_f64_complex(log, rng, param, "noise")?.unwrap_or(0.0);
        let noise_chance =
            parse_property_as_f64_complex(log, rng, param, "noise-chance")?.unwrap_or(1.0);
        let min_brightness =
            parse_property_as_f64_complex(log, rng, param, "min-brightness")?.unwrap_or(0.0);
        let max_brightness =
            parse_property_as_f64_complex(log, rng, param, "max-brightness")?.unwrap_or(100.0);

        let generated_map = generate_gradient_map(
            rng,
            amnt,
            noise,
            noise_chance,
            min_brightness,
            max_brightness,
        );

        GradientMap::with_map(generated_map)
    } else {
        log.alert("too tired. don't wanna do gradient map right now. sry.");

        let map = param
            .as_sequence()
            .expect("[gradient-map] must be a list of valid mappings (or a mapping itself).");
        let map = map
            .iter()
            .map(|entry| {
                let entry = entry
                    .as_mapping()
                    .expect("entries in [gradient-map] must be mappings.");
                let luma = entry
                    .get("luma")
                    .expect("[gradient-map.?.luma] is required.")
                    .as_f64()
                    .expect("[gradient-map.?.luma] must be a valid positive float.")
                    as f32;

                let col = entry
                    .get("colour")
                    .expect("[gradient-map.?.colour] is required.")
                    .as_mapping()
                    .expect("[gradient-map.?.colour] must be a mapping.");

                // (*parse_colour(log, rng, col)?.choose(rng).unwrap(), luma)
                panic!("seems like you tried to make a gradient map! whoops! no way! we're doing a refactor and this thing is FUCKED!");
            })
            .collect::<Vec<_>>();

        GradientMap::with_map(map)
    });

    log.end_category()?;

    effect
}

fn generate_gradient_map(
    rng: &mut impl Rng,
    amnt: u64,
    noise: f64,
    noise_chance: f64,
    min_brightness: f64,
    max_brightness: f64,
) -> Vec<(Srgb, f32)> {
    let step_size = (max_brightness - min_brightness) / ((amnt - 1) as f64);

    // must always start from the minimum brightness.
    let mut step_loc = min_brightness as f32;

    let mut palette = Vec::new();

    for _ in 0..amnt {
        palette.push((gen_with_lightness(rng, step_loc), step_loc / 100.0));
        step_loc += step_size as f32;
    }

    if noise != 0.0 && noise_chance != 0.0 {
        palette.iter_mut().for_each(|color| {
            if rng.gen_range(0.0..1.0) <= noise_chance {
                let unbounded_l = color.0.l + rng.gen_range(-noise..noise) as f32;
                color.0.l = unbounded_l.clamp(0.0, 100.0);
            }
        });
    }

    palette
        .into_iter()
        .map(|(color, step)| (color.into_color(), step))
        .collect::<Vec<_>>()
}
