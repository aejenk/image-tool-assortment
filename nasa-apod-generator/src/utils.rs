use std::{io::Cursor, error::Error};

use base64::Engine;
use image::DynamicImage;
use rand::{distributions::Uniform, prelude::Distribution, rngs::ThreadRng};

type Date = (u32, u8, u8);

pub fn generate_random_date_between(mut rng: &mut ThreadRng, start: Date, end: Date) -> Date {
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

pub fn image_to_b64(image: &DynamicImage) -> Result<String, Box<dyn Error>> {
    let mut buf = Vec::new();
    image.write_to(&mut Cursor::new(&mut buf), image::ImageOutputFormat::Png).unwrap();
    Ok(base64::engine::general_purpose::STANDARD_NO_PAD.encode(buf))
}