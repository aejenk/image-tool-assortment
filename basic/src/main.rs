use std::{error::Error, vec, fmt::format};

use common_utils::{image::ImageRequest, effectlog::{LogEntry, ExecLog}, palette::generate_n_random_palettes};
use image::DynamicImage;
use image_effects::{
    prelude::*,
    gradient_map, GradientMap, dither::bayer::Bayer,
};
use palette::{Lch, IntoColor, rgb::Rgb, named};
use rand::{rngs::StdRng, SeedableRng, seq::SliceRandom, Rng};

fn main() -> Result<(), Box<dyn Error>> {
    let image = ImageRequest::File {
        file: "./basic/data/_input.png",
        max_dim: Some(720),
    }.perform()?;

    //let image = ImageRequest::Url { 
    //    url: "https://lh3.googleusercontent.com/pw/AIL4fc-WFgChmSRG0RyjLBMEx5JmWrkyXzQRXaNvCNyZzKI2Cv6z0XHzf7lQ4npeipeZ3N95Nov7yWK6Eh-gQnEViPIao2-qfo-ggyWRpj2jV0pLOhoC7KiceovLwySHPPG2QT6iyIIBUC_C2vW7Gq04XhcD8n1_SCno-c9W_mW29RSCAPiMAvSazWt6BWq0RmEgBPECxg4XQKgHpXkZdQdpRYNSJ9lQudpzoEFTq-gL8Swjp9q360p-Ke4i3mci4HzuBz3DNyjcCyKqP62iaYDHYp34gIEhKvTKOIiLkhHlvvBGzYQwniP1sZPQUfcqJG5kXRqSXu0ZTngKOtbgejj1sudSNETK9mY687-JgHBMJYVcbH8Ctj1sbcYDjNwtMMkgbPi6XhjDJ0Vy4zo5scWxe-dFu5t9-Ijixxx4rTF3al0G2xeViGKy-0PP_pakllOIukJof3nvguvEDWZf9O5GrkP4ZcP4I-R7SAeJjLN6OZH5U2246tMLprNhFZT6YdmrGp4IMWxNoLidR5cjuZ3NGou53-zvl5EN9J-oaI__9Xjdy46F_9kcLqijjTjN5nBQ7TzXOqNS0XzVvvkv8MHAR9gIFTPmwGB01w1qtM6eJI3jKDyqdcvgWc6-tNPfIaXFb1M9D6dmuVI9uMRYPDdOgpUUNzxzicWn_oaITX3IiLZqo01sFyi2E_sNphEuiduPUzkp8forZg18R9fl24MbypYhORg9jxcYxeHm47u60AiOgK4ckL5SJ6HH3Oi6TZnu8dAk0wE8x5jMy3LEp5dnyhyTGIwLYPWTEi413-3E_T2yKCtblWMRyJMqoRs3bCl_MlXAm_Gl2ydR0cUU53daCoOgvEsT0RMKlhAuf01SdI9p-Ivj-e8CPeIbPxQPMtld3UnyNJBiO_iyh6MKcaloaNJBEdY=w705-h940-s-no?authuser=0",
    //    // max_dim: None,
    //    // max_dim: Some(1080), 
    //    max_dim: Some(720),
    //    // max_dim: Some(360),
    //}.perform()?;

    image.save("./basic/data/__SOURCE.png")?;

    generate_images_with_n_random_palettes(image, 500)?;
    // generate_images_with_predetermined_palettes(image, common_utils::palette::palettes())?;

    // let gradient = generate_hue_gradient(230.0);
    // print_gradient_grid(gradient);

    Ok(())
}

fn generate_images_with_n_random_palettes(image: DynamicImage, n: usize) -> Result<(), Box<dyn Error>> {
    let mut rng = StdRng::from_entropy();
    let palettes = generate_n_random_palettes(&mut rng, n);

    for (i, (palette, mut log)) in palettes.into_iter().enumerate() {
        println!("palette {i} / {n}");

        let possible_matrix_sizes = [2, 4, 8, 16];
        let matrix_size = possible_matrix_sizes.choose(&mut rng).unwrap();
        let matrix_size = &2;

        let mut effects: Vec<Box<dyn Effect<DynamicImage>>> = Vec::new();

        if rng.gen_bool(0.5) {
            let contrast_factor = rng.gen_range(1.0..3.0);    
            log.add_entry(LogEntry::effect("contrast".into(), contrast_factor.to_string()));
            effects.push(Box::new(filters::Contrast(contrast_factor)));
        }
        if rng.gen_bool(0.5) {
            let luma_factor = rng.gen_range(-0.3..0.3);
            log.add_entry(LogEntry::effect("brighten".into(), luma_factor.to_string()));
            effects.push(Box::new(filters::Brighten(luma_factor)));
        }
        if rng.gen_bool(0.5) {
            let hue_shift = rng.gen_range(0.0..360.0); 
            log.add_entry(LogEntry::effect("hue-rotate".into(), hue_shift.to_string()));
            effects.push(Box::new(filters::HueRotate(hue_shift)));
        }
        if rng.gen_bool(0.5) {
            let chroma_factor = rng.gen_range(-0.3..0.3); 
            log.add_entry(LogEntry::effect("saturate".into(), chroma_factor.to_string()));
            effects.push(Box::new(filters::Saturate(chroma_factor)));
        }

        let mut image = image.clone();

        for effect in effects {
            image = effect.affect(image);
        }

        log.add_entry(LogEntry::effect("dither(bayer)".into(), format!("matrix-size({matrix_size})")));

        image.apply(&Bayer::new(*matrix_size as usize, palette))
            .save(format!("./basic/data/gen-output-{i}.png"))?;

        log.write_to(format!("./basic/data/gen-output-{i}.log.txt").as_str())?;
    }

    Ok(())
}

fn generate_images_with_predetermined_palettes(image: DynamicImage, palettes: Vec<(&str, Vec<Rgb>)>) -> Result<(), Box<dyn Error>> {
    // let palettes = common_utils::palette::palettes();

    for (name, palette) in palettes {
        println!("Palette: {}", name);
        image
            .clone()
            // .apply(&Filter::Brighten( 0.1))
            // .apply(&Filter::Contrast( 2.0))
            // .apply(&Filter::GradientMap(&gradient))
            // .apply(&Filter::MultiplyHue(3.0))
            // .apply(&Filter::RotateHue(60.0))
            .apply(&Bayer::new(8, palette))
            // .apply(&Dither::Atkinson(palette))
            // .apply(&Filter::Saturate(0.2))
            // .apply(&Filter::Contrast(2.5))
            // .apply(&Filter::RotateHue(240.0))
            // .apply(&Filter::MultiplyHue(80.0))
            .save(format!("./basic/data/output-{}.png", name))?;
    }

    Ok(())
}