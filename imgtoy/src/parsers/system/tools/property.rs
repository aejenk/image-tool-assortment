use std::cell::RefCell;

use serde_yaml::Value;

use crate::{
    effects::Log,
    parsers::system::tools::{complex_primitive::ComplexPrimitive, primitive::Primitive},
};

pub struct Property<'a> {
    name: Option<&'a str>,
    value: Value,
}

impl<'a> Property<'a> {
    pub fn from(value: Value) -> Property<'a> {
        Self { name: None, value }
    }

    pub fn name(&mut self, property_name: &'a str) -> &mut Self {
        self.name = Some(property_name);
        self
    }

    fn get_name(&self) -> &'a str {
        self.name.expect("hidhsidhisd")
    }

    pub fn get_prop(&self, log: Log) -> &Value {
        &self
            .value
            .get(self.get_name())
            .expect(format!("expected property [{}]", self.get_name()).as_str())
    }

    pub fn get_prop_as_u64(&self, log: Log) -> Primitive<u64> {
        let prop = self.get_prop(log);
        Primitive::<u64>::parse_u64(log, prop)
    }

    pub fn get_prop_as_i64(&self, log: Log) -> Primitive<i64> {
        let prop = self.get_prop(log);
        Primitive::<i64>::parse_i64(log, prop)
    }

    pub fn get_prop_as_f64(&self, log: Log) -> Primitive<f64> {
        let prop = self.get_prop(log);
        Primitive::<f64>::parse_f64(log, prop)
    }

    pub fn get_prop_as_str(&self, log: Log) -> Primitive<String> {
        let prop = self.get_prop(log);
        Primitive::<String>::parse_str(log, prop)
    }

    pub fn get_prop_as_complex_u64(&self, log: Log) -> ComplexPrimitive<u64> {
        let prop = self.get_prop(log);
        ComplexPrimitive::<u64>::for_u64(log, prop)
    }

    pub fn get_prop_as_complex_i64(&self, log: Log) -> ComplexPrimitive<i64> {
        let prop = self.get_prop(log);
        ComplexPrimitive::<i64>::for_i64(log, prop)
    }

    pub fn get_prop_as_complex_f64(&self, log: Log) -> ComplexPrimitive<f64> {
        let prop = self.get_prop(log);
        ComplexPrimitive::<f64>::for_f64(log, prop)
    }

    pub fn get_child(&self, log: Log) -> Self {
        Property {
            name: self.name,
            value: self.get_prop(log).clone(),
        }
    }
}
