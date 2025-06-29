use std::error::Error;

use crate::logging::alt::SystemLog;

pub type Log<'a> = &'a mut SystemLog;
pub type BaseResult<T> = Result<T, Box<dyn Error>>;
