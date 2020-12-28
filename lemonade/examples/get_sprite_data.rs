use std::{env, fs::File};

fn main() {
    let rom_path = env::args().nth(1).unwrap();
    let mut file = File::open(rom_path).unwrap();

    let ines = ines_parser::Ines::from_reader(&mut file).unwrap();
    let chr_rom = ines.chr_rom.unwrap();
    let sprites = lemonade::Sprites::new(&chr_rom);

    for sprite in sprites.clone() {
        println!("{:?}\n", sprite.buffer());
    }

    println!("As RGB:");
    for sprite in sprites {
        let rgb_values = sprite
            .to_rgb(lemonade::ColourPalette::CLASSIC_MARIO)
            .into_iter()
            .map(|val| {
                val.to_vec()
                    .into_iter()
                    .map(|colour| -> [u8; 3] { colour.into() })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        println!("{:?}\n", rgb_values,);
    }
}
