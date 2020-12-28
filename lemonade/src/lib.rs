#![no_std]
#![warn(clippy::all, clippy::pedantic)]

use core::slice::ChunksExact;

// One sprite has the size of 16 bytes
const SPRITE_SIZE: usize = 16;
const SPRITE_WIDTH_HEIGHT: usize = 8;

pub type RgbSprite = [[Colour; SPRITE_WIDTH_HEIGHT]; SPRITE_WIDTH_HEIGHT];

#[derive(Clone, Copy, Debug)]
pub struct ColourPalette {
    background: Colour,
    colours: [Colour; 3],
}

impl ColourPalette {
    pub const CLASSIC_MARIO: ColourPalette = ColourPalette::new(
        Colour::new(0, 0, 0),
        [
            Colour::new(189, 8, 8),
            Colour::new(217, 167, 57),
            Colour::new(126, 153, 83),
        ],
    );

    #[must_use]
    pub const fn new(background: Colour, colours: [Colour; 3]) -> Self {
        Self {
            background,
            colours,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Colour {
    r: u8,
    g: u8,
    b: u8,
}

impl Colour {
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    #[must_use]
    pub const fn raw_colour(self) -> [u8; 3] {
        [self.r, self.g, self.b]
    }
}

impl From<[u8; 3]> for Colour {
    fn from(raw_rgb: [u8; 3]) -> Self {
        Self::new(raw_rgb[0], raw_rgb[1], raw_rgb[2])
    }
}

impl Into<[u8; 3]> for Colour {
    fn into(self) -> [u8; 3] {
        [self.r, self.g, self.b]
    }
}

fn bit_at(num: u8, idx: u8) -> bool {
    (num >> idx) & 1 == 1
}

pub struct Sprite<'a> {
    raw_sprite_data: &'a [u8],
}

impl<'a> Sprite<'a> {
    #[must_use]
    pub fn buffer(&self) -> &'a [u8] {
        self.raw_sprite_data
    }

    #[must_use]
    pub fn to_rgb(&self, colour_palette: ColourPalette) -> RgbSprite {
        let mut byte_chunks = self.raw_sprite_data.chunks_exact(SPRITE_WIDTH_HEIGHT);

        let mut rgb_iterator = byte_chunks
            .next()
            .unwrap()
            .iter()
            .zip(byte_chunks.next().unwrap())
            .map(|(first_byte, second_byte)| {
                let mut colour_data = [Colour::default(); SPRITE_WIDTH_HEIGHT];

                // Won't be truncated because 8 fits easily into a byte
                #[allow(clippy::cast_possible_truncation)]
                for i in 0..SPRITE_WIDTH_HEIGHT as u8 {
                    // None of the bits is set => Background colour
                    // The bit of the first byte is set => First colour
                    // The bit of the second byte is set => Second colour
                    // The bit if the first and second byte is set => Third colour

                    if bit_at(*first_byte, i) && bit_at(*second_byte, i) {
                        // Colour 3
                        colour_data[i as usize] = colour_palette.colours[2];
                    } else if bit_at(*second_byte, i) {
                        // Colour 2
                        colour_data[i as usize] = colour_palette.colours[1];
                    } else if bit_at(*first_byte, i) {
                        // Colour 1
                        colour_data[i as usize] = colour_palette.colours[0];
                    } else {
                        // Background
                        colour_data[i as usize] = colour_palette.background;
                    }
                }

                colour_data
            });

        // We have to do this to avoid having to use alloc
        let mut rgb_data = [[Colour::default(); SPRITE_WIDTH_HEIGHT]; SPRITE_WIDTH_HEIGHT];
        for data_ref in &mut rgb_data {
            *data_ref = rgb_iterator.next().unwrap();
        }

        rgb_data
    }
}

#[derive(Clone)]
pub struct Lemonade<'a> {
    sprites: ChunksExact<'a, u8>,
}

impl<'a> Lemonade<'a> {
    #[must_use]
    pub fn new(data: &'a [u8]) -> Self {
        let sprites = data.chunks_exact(SPRITE_SIZE);

        Self { sprites }
    }

    #[must_use]
    pub fn num_sprites(&self) -> usize {
        self.sprites.len()
    }
}

impl<'a> Iterator for Lemonade<'a> {
    type Item = Sprite<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.sprites
            .next()
            .map(|raw_sprite_data| Sprite { raw_sprite_data })
    }
}
