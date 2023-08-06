use image_filters::{
    prelude::*,
    utils::{
        image::{load_image_from_url_with_max_dim, load_image_with_max_dim},
        ImageFilterResult,
    },
};

mod palettes;

// this file is essentially for testing / running the code, more than providing an actual reusable binary

fn main() -> ImageFilterResult<()> {
    let palettes = palettes::palettes();

    // const IMAGE_LINK: &'static str = "./basic/data/image.png";
    const IMAGE_LINK: &'static str = "https://www.thoughtco.com/thmb/QIHGvOYobApZ_sY6xRjIkBdhcqg=/1500x0/filters:no_upscale():max_bytes(150000):strip_icc()/521928855-56a9e2925f9b58b7d0ffac0a.jpg";
    const IS_URL: bool = true;
    const MAX_DIM: u32 = 1080;

    let image = if IS_URL {
        load_image_from_url_with_max_dim(IMAGE_LINK, MAX_DIM)?
    } else {
        load_image_with_max_dim(IMAGE_LINK, MAX_DIM)?
    };

    image.save("./basic/data/__SOURCE.png")?;

    // let gradient_map: Vec<(Srgb, f32)> = [
    //     (Lch::new(0.0, 100.0, 0.0), 0.00),
    //     (Lch::new(60.0, 100.0, 0.0), 0.20),
    //     (Lch::new(60.0, 100.0, 90.0), 0.40),
    //     (Lch::new(60.0, 100.0, 180.0), 0.60),
    //     (Lch::new(80.0, 100.0, 270.0), 0.80),
    //     (Lch::new(100.0, 100.0, 360.0), 1.00),
    // ]
    //     .iter()
    //     .map(|(colour, th)| (Srgb::from_color(*colour), *th))
    //     .collect::<Vec<_>>();

    for (name, palette) in palettes.into_iter() {
        image
            .clone()
            .apply(Filter::Contrast( 1.2))
            .apply(Dither::Bayer(8, &palette))
            .save(format!("./basic/data/output-{}.png", name))?;
    }

    Ok(())
}
