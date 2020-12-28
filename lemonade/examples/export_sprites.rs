use {
    image::{codecs::png::PngEncoder, ColorType},
    std::{
        env,
        fs::{self, File},
    },
};

fn main() {
    let rom_path = env::args().nth(1).unwrap();
    let mut file = File::open(rom_path).unwrap();

    let ines = ines_parser::Ines::from_reader(&mut file).unwrap();
    let chr_rom = ines.chr_rom.unwrap();
    let sprites = lemonade::Lemonade::new(&chr_rom);

    fs::create_dir("sprites").ok();
    for (index, sprite) in sprites.into_iter().enumerate() {
        let rgb_values = sprite
            .to_rgb(lemonade::ColourPalette::CLASSIC_MARIO)
            .iter()
            .flat_map(|item| item.iter().flat_map(|colour| colour.raw_colour().to_vec()))
            .collect::<Vec<_>>();

        let mut sprite_file = File::create(format!("sprites/{}.png", index)).unwrap();
        let encoder = PngEncoder::new(&mut sprite_file);
        encoder.encode(&rgb_values, 8, 8, ColorType::Rgb8).unwrap();
    }
}
