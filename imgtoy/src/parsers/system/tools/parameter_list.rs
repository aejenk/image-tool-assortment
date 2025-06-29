use serde_yaml::Value;

use crate::effects::Log;

/// Represents a parsed list in the configuration.
///
/// When dealing with this method, the "value" should be the list itself, since this doesn't extract any parameters.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct ParameterList<T: Clone + PartialOrd> {
    pub items: Vec<T>,
}

impl<T: Clone + PartialOrd> ParameterList<T> {
    fn _parse_seq<'a>(log: Log, value: &'a Value) -> &'a Vec<Value> {
        let seq = value.as_sequence();
        if seq.is_none() {
            panic!("expected sequence")
        }
        seq.unwrap()
    }

    pub fn parse_as_u64(log: Log, value: &Value) -> ParameterList<u64> {
        let seq = Self::_parse_seq(log, value);

        ParameterList {
            items: seq
                .iter()
                .map(|val| val.as_u64().expect("expected item to be a u64"))
                .collect::<Vec<_>>(),
        }
    }

    pub fn parse_as_f64(log: Log, value: &Value) -> ParameterList<f64> {
        let seq = Self::_parse_seq(log, value);

        ParameterList {
            items: seq
                .iter()
                .map(|val| val.as_f64().expect("expected item to be a f64"))
                .collect::<Vec<_>>(),
        }
    }

    pub fn parse_as_str(log: Log, value: &Value) -> ParameterList<String> {
        let seq = Self::_parse_seq(log, value);

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

    pub fn parse_as_nested_seq<'a>(log: Log, value: &'a Value) -> ParameterList<&'a Vec<Value>> {
        let seq = Self::_parse_seq(log, value);

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

impl ParameterList<&Vec<Value>> {
    pub fn of_u64(self, log: Log) -> ParameterList<ParameterList<u64>> {
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

    pub fn of_f64(self, log: Log) -> ParameterList<ParameterList<f64>> {
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

    pub fn of_str(self, log: Log) -> ParameterList<ParameterList<String>> {
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
