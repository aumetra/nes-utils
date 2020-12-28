use std::{env, fs::File};

fn main() {
    let rom_path = env::args().nth(1).unwrap();
    let mut file = File::open(rom_path).unwrap();

    let ines = ines_parser::Ines::from_reader(&mut file).unwrap();
    let chr_rom = ines.chr_rom.unwrap();
    let sprites = lemonade::Lemonade::new(&chr_rom);

    println!("Sprite count: {}", sprites.num_sprites());
}
