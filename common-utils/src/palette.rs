use image_effects::{prelude::{SrgbColour as RGB, IntoGradient, IntoGradientLch}, colour::gradient::GradientMethod};
use palette::{rgb::Rgb, Lch, named, IntoColor};
use rand::{rngs::StdRng, Rng};

use crate::effectlog::{ExecLog, LogEntry};

pub fn palettes<'a>() -> Vec<(&'a str, Vec<Rgb>)> { 
    const GRADIENT_METHOD: GradientMethod = GradientMethod::LCH;
    let palettes = vec![
        (
            "pastel",
            [
                RGB::CYAN.build_gradient(10, GRADIENT_METHOD),
                RGB::PINK.build_gradient(10, GRADIENT_METHOD),
                vec![RGB::BLACK, RGB::WHITE],
            ]
            .concat(),
        ),
        (
            "nightlife",
            [
                RGB::BLUE.build_gradient(10, GRADIENT_METHOD),
                RGB::CYAN.build_gradient(10, GRADIENT_METHOD),
                RGB::PINK.build_gradient(10, GRADIENT_METHOD),
                RGB::ROSE.build_gradient(10, GRADIENT_METHOD),
                RGB::YELLOW.build_gradient(10, GRADIENT_METHOD),
                RGB::GOLD.build_gradient(10, GRADIENT_METHOD),
                vec![RGB::BLACK, RGB::WHITE],
            ]
            .concat(),
        ),
        (
            "crisp-nightlife",
            [
                vec![RGB::CYAN, RGB::PINK, RGB::YELLOW, RGB::GOLD, RGB::BLUE, RGB::PURPLE, RGB::WHITE, RGB::BLACK],
            ]
            .concat(),
        ),
        (
            "carrot",
            [
                RGB::ORANGE.build_gradient(10, GRADIENT_METHOD),
                RGB::GREEN.build_gradient(10, GRADIENT_METHOD),
                vec![RGB::BLACK, RGB::WHITE],
            ]
            .concat(),
        ),
        (
            "nb",
            [
                RGB::GOLD.build_gradient(10, GRADIENT_METHOD),
                RGB::PURPLE.build_gradient(30, GRADIENT_METHOD),
                vec![RGB::BLACK, RGB::WHITE],
            ]
            .concat(),
        ),
        (
            "nblofi",
            [
                vec![RGB::BLACK, RGB::WHITE, RGB::GOLD, RGB::YELLOW, RGB::PURPLE,],
            ]
            .concat(),
        ),
        (
            "sunsky",
            [
                RGB::ORANGE.build_gradient(10, GRADIENT_METHOD),
                RGB::BLUE.build_gradient(10, GRADIENT_METHOD),
                vec![RGB::BLACK, RGB::WHITE],
            ]
            .concat(),
        ),
        (
            "depth",
            [
                RGB::BLUE.build_gradient(10, GRADIENT_METHOD),
                RGB::PURPLE.build_gradient(10, GRADIENT_METHOD),
                vec![RGB::BLACK, RGB::WHITE],
            ]
            .concat(),
        ),
        (
            "refresh",
            [
                RGB::BLUE.build_gradient(10, GRADIENT_METHOD),
                RGB::CYAN.build_gradient(10, GRADIENT_METHOD),
                RGB::AQUAMARINE.build_gradient(10, GRADIENT_METHOD),
                RGB::GREEN.build_gradient(10, GRADIENT_METHOD),
                vec![RGB::BLACK, RGB::WHITE],
            ]
            .concat()
        ),
        (
            "nebula",
            [
                RGB::RED.build_gradient(10, GRADIENT_METHOD),
                RGB::ROSE.build_gradient(10, GRADIENT_METHOD),
                RGB::PURPLE.build_gradient(10, GRADIENT_METHOD),
                vec![RGB::BLACK, RGB::WHITE],
            ]
            .concat()
        ),
        (
            "dragon",
            [
                RGB::RED.build_gradient(40, GRADIENT_METHOD),
                {
                    let mut red = RGB::RED.clone();
                    red.red = 0.4;
                    red 
                }.build_gradient(10, GRADIENT_METHOD),
                // vec![RGB::BLACK, RGB::WHITE],
            ]
            .concat()
        ),
        (
            "minty",
            [
                RGB::GREEN.build_gradient(40, GRADIENT_METHOD),
                RGB::GOLD.build_gradient(4, GRADIENT_METHOD),
                vec![RGB::BLACK, RGB::WHITE],
            ]
            .concat()
        ),
        (
            "corru",
            [
                vec![RGB::BLACK, RGB::WHITE, RGB::CYAN, RGB::MAGENTA, RGB::YELLOW],
            ].concat()
        ),
        (
            "zx",
            [
                vec![RGB::BLACK, RGB::WHITE, RGB::CYAN, RGB::MAGENTA],
            ].concat()
        ),
        (
            "mono",
            [
                vec![RGB::BLACK, RGB::WHITE],
            ].concat()
        ),
        (
            "orangurple",
            [
                vec![RGB::BLACK, RGB::WHITE, RGB::PURPLE, RGB::ORANGE],
            ].concat()
        ),
        (
            "calmfire",
            [
                vec![RGB::WHITE, RGB::ROSE, RGB::ORANGE, RGB::BLACK],
            ].concat()
        ),
        (
            "rcgmby",
            [
                vec![RGB::RED, RGB::CYAN, RGB::GREEN, RGB::MAGENTA, RGB::BLUE, RGB::YELLOW],
            ].concat()
        ),
        (
            "eight-bit",
            [
                vec![RGB::RED, RGB::BLUE, RGB::GREEN, RGB::WHITE, RGB::BLACK],
            ].concat(),
        ),
        (
            "deep-crushed-ocean",
            [
                vec![RGB::BLACK, RGB::PURPLE, RGB::BLUE, RGB::CYAN, RGB::WHITE],
            ].concat(),
        ),
        (
            "falling-bitsun",
            [
                vec![RGB::BLACK, RGB::PURPLE, RGB::RED, RGB::ORANGE, RGB::GOLD, RGB::WHITE],
            ].concat(),
        ),
        (
            "pixeleaf",
            [
                RGB::GREEN.build_gradient(4, GRADIENT_METHOD),
                vec![RGB::BLACK, RGB::WHITE],
            ].concat(),
        ),
        (
            "cold-comfort",
            [
                vec![
                    Rgb::new(0.8757157, 0.89244765, 0.6861303),
                    Rgb::new(0.39427757, 0.27967575, 0.0),
                    RGB::BLACK, RGB::WHITE,
                ],
                Rgb::new(0.30509338, 0.0, 0.0).build_gradient_lch(8),
                Rgb::new(0.26752675, 0.6986885, 0.8242886).build_gradient_lch(5),
            ].concat(),
        ),
        (
            "red-glow",
            [
                vec![
                    Rgb::new(0.88807595, 0.72662807, 1.0),
                    Rgb::new(1.0, 0.06328131, 0.090972625),
                    Rgb::new(0.47994733, 0.0, 0.0),
                    RGB::BLACK, RGB::WHITE,
                ],
            ].concat(),
        ),
        (
            "purple-gold",
            [
                vec![
                    Rgb::new(0.96833843, 0.92517495, 0.74125403),
                    Rgb::new(1.0, 0.68283576, 0.0),
                    Rgb::new(0.14215139, 0.0, 0.55182195),
                    RGB::BLACK, RGB::WHITE,
                ],
                Rgb::new(0.66233885, 0.39216012, 0.0).build_gradient_lch(4),
            ].concat(),
        ),
        (
            "purple-sunset",
            [
                vec![
                    Rgb::new(1.0, 0.8190029, 0.7159059),
                    Rgb::new(0.8197559, 0.0, 0.7211565),
                    Rgb::new(0.8983417, 0.7693191, 0.9346823),
                    Rgb::new(0.7556307, 0.6901328, 0.038954336),
                    Rgb::new(0.0, 0.36144787, 0.7547207),
                    Rgb::new(0.43315938, 0.2536253, 0.26589835),
                ],
                Rgb::new(0.282966, 0.6108818, 0.6515609).build_gradient_lch(9),
                Rgb::new(0.0, 0.063025326, 0.41698724).build_gradient_lch(6),
            ].concat(),
        ),
        (
            "muddy-aqua",
            [
                vec![
                    Rgb::new(0.0, 0.9178139, 0.8974363),
                    Rgb::new(0.0, 0.20158169, 0.28371152),
                    Rgb::new(0.035051122, 0.95013696, 0.9157564),
                    Rgb::new(0.31381243, 0.3960095, 1.0),
                    Rgb::new(0.30482486, 0.34771544, 0.868871),
                    Rgb::new(0.90769166, 0.7069615, 0.6081852),
                ],
                Rgb::new(0.14610958, 0.115649216, 0.6593825).build_gradient_lch(6),
                Rgb::new(1.0, 0.6862935, 0.2892494).build_gradient_lch(2),
            ].concat(),
        ),
        (
            "peachy",
            [
                vec![
                    Rgb::new(0.9914246, 0.93015397, 0.0),
                    Rgb::new(0.0, 0.5146738, 0.0),
                    Rgb::new(0.9262706, 0.67115414, 0.2871154),
                    Rgb::new(0.95351595, 0.70660895, 0.8328703),
                    Rgb::new(0.7003081, 0.73479205, 0.5038984),
                    Rgb::new(0.0, 0.4295476, 0.26493788),
                    Rgb::new(1.0, 0.0, 0.48583663),
                ],
                Rgb::new(0.0, 0.40176412, 0.22147202).build_gradient_lch(7),
                Rgb::new(0.0, 0.19702128, 0.50473636).build_gradient_lch(5),
            ].concat(),
        )
    ];
    palettes
}

pub type LoggedPalette = (Vec<Rgb>, ExecLog);

pub fn generate_n_random_palettes(rng: &mut StdRng, n: usize) -> Vec<LoggedPalette> {
    let mut palettes = Vec::with_capacity(n);

    for _ in 0..n {
        let logged_palette = generate_random_palette(rng);

        palettes.push(logged_palette);
    }

    palettes
}

pub fn generate_random_palette(mut rng: &mut impl Rng) -> LoggedPalette {
    fn gen_with_lightness(rng: &mut impl Rng, min: f32, max: f32) -> Lch {
        Lch::new(
            rng.gen_range(min..=max),
            rng.gen_range(0.0..128.0),
            rng.gen_range(0.0..360.0),        
        )
    }

    let mut palette = vec![
        gen_with_lightness(rng, 80.0, 100.0),
        gen_with_lightness(rng, 20.0,  80.0),
        gen_with_lightness(rng,  0.0,  20.0),
    ];

    for _ in 0..rng.gen_range(0..10) {
        palette.push(gen_with_lightness(&mut rng,  0.0,  100.0));
    }

    let mut log = ExecLog::new();
    
    let mut palette = palette
        .into_iter()
        .map(|col| {
            let col: Rgb = col.into_color();
            col
        })
        .map(|col| {
            if rng.gen_bool(0.10) {
                let amnt = rng.gen_range(2..=10);
                log.add_entry(LogEntry::gradient(col, amnt));
                col.build_gradient_lch(amnt)
            } else {
                log.add_entry(LogEntry::colour(col));
                vec![col]
            }
        })
        .collect::<Vec<_>>()
        .concat();

    if rng.gen_bool(0.75) {
        palette.push(named::BLACK.into_format());
        log.add_entry(LogEntry::colour(named::BLACK.into_format()));
        palette.push(named::WHITE.into_format());
        log.add_entry(LogEntry::colour(named::WHITE.into_format()));
    }

    (palette, log)
}

pub fn generate_palette_html(gradient: Vec<Rgb>) -> String {
    let palette_html = gradient.iter().map(|colour| {
        let (r, g, b) = colour.into_format::<u8>().into_components();
        format!("<div style=\"height: 100%; background: rgb({r},{g},{b}); flex-grow: 1; padding:3px;\"></div>")
    }).collect::<Vec<_>>().concat();

    vec![
        "<div style:\"width: 100%; display: flex; flex-wrap: wrap;\">",
        &palette_html,
        "</div>",
    ].concat()
}

pub fn generate_hue_gradient(hue: f32) -> Vec<Vec<Lch>> {
    (0..8).into_iter().map(|chroma| Lch::new(0.0, 128.0 - (chroma*16) as f32, hue).build_gradient_lch(10)).collect()
}

pub fn print_gradient_grid(gradient: Vec<Vec<Lch>>) {
    let div = gradient.iter().map(|chroma_variant| {
        let html_lines = chroma_variant.iter().map(|color| {
            let (l, c, h) = color.into_components();
            let h = h.into_degrees();
            format!("<div style=\"height: 100%; background: lch({l}% {c} {h}); flex-grow: 1; padding:3px;\"></div>")
        }).collect::<Vec<_>>().join("\n");

        vec![
            "<div style=\"width: 100%; height:50px; display: flex;\">",
            html_lines.as_str(),
            "</div>",
        ].join("\n")
    }).collect::<Vec<_>>().join("\n");

    println!("{div}");
}