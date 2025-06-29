use image_effects::{
    dither::{
        error::{ErrorPropagator, WithPalette},
        ordered::Ordered,
    },
    effect::Effect,
    filter::filters::{
        Brighten, Contrast, GradientMap, HueRotate, MultiplyHue, QuantizeHue, Saturate,
    },
};
use rand::Rng;
use serde_yaml::Value;

use crate::{
    effects::{BaseResult, Log},
    parsers::{
        effects::{gradient_map::parse_gradient_map, quantize_hue::parse_quantize_hue},
        error_propagator::parse_error_propagator,
        ordered::parse_ordered,
        properties::parse_factor,
    },
};

pub mod gradient_map;
pub mod quantize_hue;

#[derive(Debug)]
pub enum EffectKind {
    HueRotate,
    Contrast,
    Brighten,
    Saturate,
    GradientMap,
    QuantizeHue,
    MultiplyHue,

    Ordered,
    ErrorPropagator,
}

impl From<&str> for EffectKind {
    fn from(value: &str) -> Self {
        match value {
            "hue-rotate" => Self::HueRotate,
            "contrast" => Self::Contrast,
            "brighten" => Self::Brighten,
            "saturate" => Self::Saturate,
            "gradient-map" => Self::GradientMap,
            "quantize-hue" => Self::QuantizeHue,
            "multiply-hue" => Self::MultiplyHue,
            "ordered" => Self::Ordered,
            _ => Self::ErrorPropagator,
        }
    }
}

pub fn parse_effect_kind(effect: &Value) -> EffectKind {
    let effect = effect
        .as_mapping()
        .expect("whoopsies something has gone wrong")
        .keys()
        .next()
        .unwrap()
        .as_str()
        .expect("an effect must start with its name as a string.");

    effect.into()
}

pub fn parse_effects<'a, 'b, T>(
    log: Log,
    rng: &mut impl Rng,
    root_value: &serde_yaml::Value,
) -> BaseResult<Vec<Box<dyn Effect<T>>>>
where
    HueRotate: Effect<T>,
    Contrast: Effect<T>,
    Brighten: Effect<T>,
    Saturate: Effect<T>,
    GradientMap: Effect<T>,
    QuantizeHue: Effect<T>,
    MultiplyHue: Effect<T>,
    Ordered: Effect<T>,
    ErrorPropagator<'static, 'static, WithPalette>: Effect<T>,
{
    let effects = root_value
        .get("effects")
        .expect("[effects] was not present - is required.")
        .as_sequence()
        .expect("[effects] must be a list - wasn't.");

    Ok(effects
        .iter()
        .enumerate()
        .map(|(i, effect)| {
            effect
                .as_mapping()
                .unwrap_or_else(|| panic!("[effects.{i}] must be a map - wasn't."))
        })
        .map(|effect| {
            let keys = effect.keys().len();
            if keys != 1 {
                panic!("only one key [the effect name] is accepted by effect - found {keys}.");
            }
            parse_effect::<T>(log, rng, &Value::Mapping(effect.clone()))
                .expect("failure when parsing effect. please check log.")
        })
        .collect::<Vec<_>>())
}

fn parse_effect<T>(log: Log, rng: &mut impl Rng, effect: &Value) -> BaseResult<Box<dyn Effect<T>>>
where
    HueRotate: Effect<T>,
    Contrast: Effect<T>,
    Brighten: Effect<T>,
    Saturate: Effect<T>,
    GradientMap: Effect<T>,
    QuantizeHue: Effect<T>,
    MultiplyHue: Effect<T>,
    Ordered: Effect<T>,
    ErrorPropagator<'static, 'static, WithPalette>: Effect<T>,
{
    let kind = parse_effect_kind(effect);

    Ok(match kind {
        EffectKind::HueRotate => Box::new(parse_hue_rotate(log, rng, effect)?),
        EffectKind::Contrast => Box::new(parse_contrast(log, rng, effect)?),
        EffectKind::Brighten => Box::new(parse_brighten(log, rng, effect)?),
        EffectKind::Saturate => Box::new(parse_saturate(log, rng, effect)?),
        EffectKind::MultiplyHue => Box::new(parse_multiply_hue(log, rng, effect)?),
        EffectKind::GradientMap => Box::new(parse_gradient_map(log, rng, effect)?),
        EffectKind::QuantizeHue => Box::new(parse_quantize_hue(log, rng, effect)?),
        EffectKind::Ordered => {
            log.begin_category("ordered")?;
            let fx = parse_ordered(log, rng, effect)?;
            log.end_category()?;
            Box::new(fx)
        }
        EffectKind::ErrorPropagator => {
            log.begin_category("error-propagator")?;
            log.alert("error propagator not currently being logged.")?;
            let fx = parse_error_propagator(
                log,
                rng,
                effect,
                effect
                    .as_mapping()
                    .expect("this effect needs to be a mapping you bastard")
                    .keys()
                    .next()
                    .unwrap()
                    .as_str()
                    .unwrap(),
            )?;
            log.end_category()?;
            Box::new(fx)
        }
    })
}

pub fn parse_hue_rotate(log: Log, rng: &mut impl Rng, value: &Value) -> BaseResult<HueRotate> {
    log.begin_category("hue-rotate")?;
    let effect = Ok(HueRotate(parse_factor(
        log,
        rng,
        value.get("hue-rotate").expect("expected [hue-rotate]"),
    )? as f32));
    log.end_category()?;
    effect
}

pub fn parse_contrast(log: Log, rng: &mut impl Rng, value: &Value) -> BaseResult<Contrast> {
    log.begin_category("contrast")?;
    let effect = Ok(Contrast(parse_factor(
        log,
        rng,
        value.get("contrast").expect("expected [contrast]"),
    )? as f32));
    log.end_category()?;
    effect
}

pub fn parse_brighten(log: Log, rng: &mut impl Rng, value: &Value) -> BaseResult<Brighten> {
    log.begin_category("brighten")?;
    let effect = Ok(Brighten(parse_factor(
        log,
        rng,
        value.get("brighten").expect("expected [brighten]"),
    )? as f32));
    log.end_category()?;
    effect
}

pub fn parse_saturate(log: Log, rng: &mut impl Rng, value: &Value) -> BaseResult<Saturate> {
    log.begin_category("saturate")?;
    let effect = Ok(Saturate(parse_factor(
        log,
        rng,
        value.get("saturate").expect("expected [saturate]"),
    )? as f32));
    log.end_category()?;
    effect
}

pub fn parse_multiply_hue(log: Log, rng: &mut impl Rng, value: &Value) -> BaseResult<MultiplyHue> {
    log.begin_category("multiply-hue")?;
    let effect = Ok(MultiplyHue(parse_factor(
        log,
        rng,
        value.get("multiply-hue").expect("expected [multiply-hue]"),
    )? as f32));
    log.end_category()?;
    effect
}
