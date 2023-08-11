use image_effects::{
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

    // const IMAGE_LINK: &'static str = "./basic/data/input.png";
    const IMAGE_LINK: &'static str = "https://media.discordapp.net/attachments/818328288836583474/1139275548900737064/cd86962ee4c84277afb4a7ba98e8006dbbf0635d.png?width=676&height=676";
    const IS_URL: bool = true;
    const MAX_DIM: u32 = 1080;

    let image = if IS_URL {
        load_image_from_url_with_max_dim(IMAGE_LINK, MAX_DIM)?
    } else {
        load_image_with_max_dim(IMAGE_LINK, MAX_DIM)?
    };

    image.save("./basic/data/__SOURCE.png")?;

    // let gradient_map = gradient_map![
    //     0.00 => Lch::new(0.0, 100.0, 0.0),
    //     0.20 => Lch::new(60.0, 100.0, 0.0),
    //     0.40 => Lch::new(60.0, 100.0, 90.0),
    //     0.60 => Lch::new(60.0, 100.0, 180.0),
    //     0.80 => Lch::new(80.0, 100.0, 270.0),
    //     1.00 => Lch::new(100.0, 100.0, 360.0)
    // ]
    //     .iter()
    //     .map(|(colour, th)| (Srgb::from_color(*colour), *th))
    //     .collect::<Vec<_>>();

    for (name, palette) in palettes.iter() {
        image
            .clone()
            .apply(&Dither::Bayer(8, palette))
            .save(format!("./basic/data/output-{}.png", name))?;
    }

    // let gradient = generate_hue_gradient(230.0);
    // print_gradient_grid(gradient);

    Ok(())
}
