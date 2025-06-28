use std::vec;

use rand::{distributions::uniform::SampleUniform, seq::SliceRandom, Rng};
use serde_yaml::Value;

use crate::{
    effects::Log,
    parsers::util::logless::{parse_property_as_f64, parse_property_as_i64, parse_property_as_u64},
};

struct Property {
    name: String,
    value: Value,
}

impl Property {
    fn from(value: Value, property_name: String) -> Property {
        Property {
            name: property_name,
            value,
        }
    }

    fn get_prop(&self, log: Log) -> &Value {
        &self
            .value
            .get(&self.value)
            .expect(format!("expected property [{}]", self.name).as_str())
    }

    fn get_prop_as_u64(&self, log: Log) -> Primitive<u64> {
        let prop = self.get_prop(log);
        Primitive::<u64>::parse_u64(log, prop)
    }

    fn get_prop_as_i64(&self, log: Log) -> Primitive<i64> {
        let prop = self.get_prop(log);
        Primitive::<i64>::parse_i64(log, prop)
    }

    fn get_prop_as_f64(&self, log: Log) -> Primitive<f64> {
        let prop = self.get_prop(log);
        Primitive::<f64>::parse_f64(log, prop)
    }

    fn get_prop_as_str(&self, log: Log) -> Primitive<String> {
        let prop = self.get_prop(log);
        Primitive::<String>::parse_str(log, prop)
    }

    fn get_prop_as_complex_u64(&self, log: Log) -> ComplexPrimitive<u64> {
        let prop = self.get_prop(log);
        ComplexPrimitive::<u64>::for_u64(log, prop)
    }

    fn get_prop_as_complex_i64(&self, log: Log) -> ComplexPrimitive<i64> {
        let prop = self.get_prop(log);
        ComplexPrimitive::<i64>::for_i64(log, prop)
    }

    fn get_prop_as_complex_f64(&self, log: Log) -> ComplexPrimitive<f64> {
        let prop = self.get_prop(log);
        ComplexPrimitive::<f64>::for_f64(log, prop)
    }

    fn copy_sub_property(&self, log: Log) -> Self {
        Property {
            name: self.name.clone(),
            value: self.get_prop(log).clone(),
        }
    }
}

struct Primitive<T: Clone + PartialOrd> {
    item: T,
}

impl<T: Clone + PartialOrd> Primitive<T> {
    fn parse_u64(log: Log, value: &Value) -> Primitive<u64> {
        Primitive {
            item: value.as_u64().expect("cxpected u64"),
        }
    }

    fn parse_i64(log: Log, value: &Value) -> Primitive<i64> {
        Primitive {
            item: value.as_i64().expect("cxpected i64"),
        }
    }

    fn parse_f64(log: Log, value: &Value) -> Primitive<f64> {
        Primitive {
            item: value.as_f64().expect("cxpected u64"),
        }
    }

    fn parse_str(log: Log, value: &Value) -> Primitive<String> {
        Primitive {
            item: value.as_str().expect("cxpected u64").to_string(),
        }
    }

    fn check(&self, log: Log) -> &T {
        &self.item
    }

    fn get(self, log: Log) -> T {
        self.item
    }
}

/// Represents a complex primitive - meaning it can either be the exact value, a list, or a range.
///
/// Once `.get()` is called, it's consumed to generate *one* instance.
enum ComplexPrimitive<T: Clone + SampleUniform + PartialOrd> {
    Exact(T),
    Choice(Vec<T>),
    Range(ParameterRange<T>),
}

impl<T: Clone + SampleUniform + PartialOrd> ComplexPrimitive<T> {
    pub fn for_u64(log: Log, value: &Value) -> ComplexPrimitive<u64> {
        if let Some(exact) = value.as_u64() {
            ComplexPrimitive::Exact(exact)
        } else if let Some(options) = value.as_sequence() {
            let options = options
                .iter()
                .map(|option| option.as_u64().expect("expected list of u64"))
                .collect::<Vec<_>>();

            ComplexPrimitive::Choice(options)
        } else if value.is_mapping() {
            ComplexPrimitive::Range(ParameterRange::<u64>::parse_as_u64(log, &value))
        } else {
            panic!("expected COMPLEX PRIMITIVE - found unexpected type.");
        }
    }

    pub fn for_i64(log: Log, value: &Value) -> ComplexPrimitive<i64> {
        if let Some(exact) = value.as_i64() {
            ComplexPrimitive::Exact(exact)
        } else if let Some(options) = value.as_sequence() {
            let options = options
                .iter()
                .map(|option| option.as_i64().expect("expected list of u64"))
                .collect::<Vec<_>>();

            ComplexPrimitive::Choice(options)
        } else if value.is_mapping() {
            ComplexPrimitive::Range(ParameterRange::<u64>::parse_as_i64(log, &value))
        } else {
            panic!("expected COMPLEX PRIMITIVE - found unexpected type.");
        }
    }

    pub fn for_f64(log: Log, value: &Value) -> ComplexPrimitive<f64> {
        if let Some(exact) = value.as_f64() {
            ComplexPrimitive::Exact(exact)
        } else if let Some(options) = value.as_sequence() {
            let options = options
                .iter()
                .map(|option| option.as_f64().expect("expected list of u64"))
                .collect::<Vec<_>>();

            ComplexPrimitive::Choice(options)
        } else if value.is_mapping() {
            ComplexPrimitive::Range(ParameterRange::<f64>::parse_as_f64(log, &value))
        } else {
            panic!("expected COMPLEX PRIMITIVE - found unexpected type.");
        }
    }

    pub fn get(self, rng: &mut impl Rng) -> T {
        match self {
            ComplexPrimitive::Exact(exact) => exact,
            ComplexPrimitive::Choice(choices) => choices.choose(rng).unwrap().clone(),
            ComplexPrimitive::Range(range) => range.get(rng),
        }
    }
}

/// Represents a parameter range.
///
/// Mostly used in the ComplexPrimitive<T> but can be used elsewhere.
///
/// Supports generation of a random value within the range.
pub struct ParameterRange<T: Clone + SampleUniform + PartialOrd> {
    min: T,
    max: T,
}

impl<T: Clone + SampleUniform + PartialOrd> ParameterRange<T> {
    fn parse_as_u64(log: Log, value: &Value) -> ParameterRange<u64> {
        let min = parse_property_as_u64(&value, "min", None).expect("[.min] expected as u64");
        let max = parse_property_as_u64(&value, "max", None).expect("[.max] expected as u64");

        ParameterRange { min, max }
    }

    fn parse_as_i64(log: Log, value: &Value) -> ParameterRange<i64> {
        let min = parse_property_as_i64(&value, "min", None).expect("[.min] expected as i64");
        let max = parse_property_as_i64(&value, "max", None).expect("[.max] expected as i64");

        ParameterRange { min, max }
    }

    fn parse_as_f64(log: Log, value: &Value) -> ParameterRange<f64> {
        let min = parse_property_as_f64(&value, "min", None).expect("[.min] expected as u64");
        let max = parse_property_as_f64(&value, "max", None).expect("[.max] expected as u64");

        ParameterRange { min, max }
    }

    fn get(self, rng: &mut impl Rng) -> T {
        rng.gen_range(self.min..self.max)
    }
}

/// Represents a parsed list in the configuration.
///
/// When dealing with this method, the "value" should be the list itself, since this doesn't extract any parameters.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct ParameterList<T: Clone + PartialOrd> {
    items: Vec<T>,
}

impl<T: Clone + PartialOrd> ParameterList<T> {
    fn parse_as_seq<'a>(log: Log, value: &'a Value) -> &'a Vec<Value> {
        let seq = value.as_sequence();
        if seq.is_none() {
            panic!("expected sequence")
        }
        seq.unwrap()
    }

    fn parse_as_u64(log: Log, value: &Value) -> ParameterList<u64> {
        let seq = Self::parse_as_seq(log, value);

        ParameterList {
            items: seq
                .iter()
                .map(|val| val.as_u64().expect("expected item to be a u64"))
                .collect::<Vec<_>>(),
        }
    }

    fn parse_as_f64(log: Log, value: &Value) -> ParameterList<f64> {
        let seq = Self::parse_as_seq(log, value);

        ParameterList {
            items: seq
                .iter()
                .map(|val| val.as_f64().expect("expected item to be a f64"))
                .collect::<Vec<_>>(),
        }
    }

    fn parse_as_str(log: Log, value: &Value) -> ParameterList<String> {
        let seq = Self::parse_as_seq(log, value);

        ParameterList {
            items: seq
                .iter()
                .map(|val| {
                    val.as_str()
                        .expect("expected item to be a string")
                        .to_string()
                })
                .collect::<Vec<_>>(),
        }
    }

    fn parse_as_nested_seq<'a>(log: Log, value: &'a Value) -> ParameterList<&'a Vec<Value>> {
        let seq = Self::parse_as_seq(log, value);

        ParameterList {
            items: seq
                .iter()
                .map(|val| val.as_sequence().expect("expected item to be a sequence"))
                .collect::<Vec<_>>(),
        }
    }
}

impl<T: Clone + PartialOrd> From<Vec<T>> for ParameterList<T> {
    fn from(items: Vec<T>) -> Self {
        ParameterList { items }
    }
}

impl<'a> ParameterList<&'a Vec<Value>> {
    fn of_u64(self, log: Log) -> ParameterList<ParameterList<u64>> {
        let mut fulllist = vec![];
        for list in self.items {
            let mut sublist = vec![];
            for entry in list.iter() {
                sublist.push(entry.as_u64().expect("expected u64"));
            }
            fulllist.push(sublist.into());
        }
        fulllist.into()
    }

    fn of_f64(self, log: Log) -> ParameterList<ParameterList<f64>> {
        let mut fulllist = vec![];
        for list in self.items {
            let mut sublist = vec![];
            for entry in list.iter() {
                sublist.push(entry.as_f64().expect("expected f64"));
            }
            fulllist.push(sublist.into());
        }
        fulllist.into()
    }

    fn of_str(self, log: Log) -> ParameterList<ParameterList<String>> {
        let mut fulllist = vec![];
        for list in self.items {
            let mut sublist = vec![];
            for entry in list.iter() {
                sublist.push(entry.as_str().expect("expected f64").to_string());
            }
            fulllist.push(sublist.into());
        }
        fulllist.into()
    }
}
