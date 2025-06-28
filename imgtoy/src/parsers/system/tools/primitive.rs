use serde_yaml::Value;

use crate::effects::Log;

pub struct Primitive<T: Clone + PartialOrd> {
    item: T,
}

impl<T: Clone + PartialOrd> Primitive<T> {
    pub fn parse_u64(log: Log, value: &Value) -> Primitive<u64> {
        Primitive {
            item: value.as_u64().expect("cxpected u64"),
        }
    }

    pub fn parse_i64(log: Log, value: &Value) -> Primitive<i64> {
        Primitive {
            item: value.as_i64().expect("cxpected i64"),
        }
    }

    pub fn parse_f64(log: Log, value: &Value) -> Primitive<f64> {
        Primitive {
            item: value.as_f64().expect("cxpected u64"),
        }
    }

    pub fn parse_str(log: Log, value: &Value) -> Primitive<String> {
        Primitive {
            item: value.as_str().expect("cxpected u64").to_string(),
        }
    }

    pub fn check(&self, log: Log) -> &T {
        &self.item
    }

    pub fn get(self, log: Log) -> T {
        self.item
    }
}
