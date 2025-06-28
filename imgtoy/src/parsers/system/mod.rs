use std::vec;

use rand::{distributions::uniform::SampleUniform, seq::SliceRandom, Rng};
use serde_yaml::Value;

use crate::{
    effects::Log,
    parsers::util::logless::{parse_property_as_f64, parse_property_as_i64, parse_property_as_u64},
};

pub mod tools;
