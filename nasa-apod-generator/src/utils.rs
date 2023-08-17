use std::error::Error;

use rand::{distributions::Uniform, prelude::Distribution, rngs::ThreadRng, Rng};

type Date = (u32, u8, u8);

pub fn generate_random_date_between(mut rng: &mut impl Rng, start: Date, end: Date) -> Date {
    let earliest = start;
    let latest = end;

    let year = Uniform::from(earliest.0..latest.0);
    let month = Uniform::from(1..=12);
    let day = Uniform::from(1..=31);

    let (year, mut month, mut day) = (
        year.sample(&mut rng),
        month.sample(&mut rng),
        day.sample(&mut rng),
    );

    if year == earliest.0 {
        month = month.max(earliest.1);
        day = day.max(earliest.2);
    } else if year == latest.0 {
        month = month.min(latest.1);
        day = day.min(latest.2);
    }

    day = if month == 2 {
        if year % 4 == 0 {
            day.min(29)
        } else {
            day.min(28)
        }
    } else if (month <= 7 && month % 2 == 1) || (month > 7 && month % 2 == 0) {
        day.min(31)
    } else {
        day.min(30)
    };

    (year, month, day)
}

#[derive(Clone, Debug)]
pub(crate) struct NoneError;

impl std::fmt::Display for NoneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Expected Some(..) but got None")
    }
}

impl Error for NoneError {
    fn cause(&self) -> Option<&dyn Error> {
        None
    }

    fn description(&self) -> &str {
        ""
    }

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}