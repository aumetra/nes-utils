#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]

//!
//! Parser for the INES file format  
//!
//! [File format documentation](http://wiki.nesdev.com/w/index.php/INES)
//!

use std::{
    array::TryFromSliceError,
    borrow::Cow,
    convert::TryInto,
    io::{self, Read},
};

// The word "NES" followed by the MS-DOS EOF delimiter
const MAGIC_BYTES: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A];

const HEADER_SIZE: usize = 16;
const TRAINER_SIZE: usize = 512;

const PRG_ROM_CHUNK_SIZE: usize = 16_384;
const CHR_ROM_CHUNK_SIZE: usize = 8192;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error")]
    Io(#[from] io::Error),

    #[error("Magic bytes didn't match; expected {:?}, got {:?}", MAGIC_BYTES, .0)]
    MagicBytesMismatch([u8; 4]),

    #[error("TryFromSliceError")]
    TryFromSlice(#[from] TryFromSliceError),
}

#[derive(Debug)]
pub enum VramLayout {
    HorizontalMirroring,
    VerticalMirroring,
    FourScreen,
}

#[derive(Debug)]
pub struct Header {
    pub prg_rom_size: usize,
    pub chr_rom_size: usize,
    pub vram_layout: VramLayout,
    pub has_persistent_memory: bool,

    has_trainer: bool,

    pub mapper_number: u8,
}

// We use the `Cow` type here to avoid unnecessary allocations
// When read from a stream, we have no other choice than to allocate memory and copy the contents to it
// But when we get the data from a byte slice reference, we have a choice
//
// Thus, when reading from a file, we wrap the allocated memory into `Cow::Owned`
// And when creating references to a sub-section of a byte slice, we wrap the created reference into `Cow::Borrowed`
//
// (I just really like `Cow` <3)
pub struct Ines<'a> {
    /// Header
    pub header: Header,
    /// Trainer
    pub trainer: Option<Cow<'a, [u8]>>,
    /// PRG ROM
    pub prg_rom: Cow<'a, [u8]>,
    /// CHR ROM
    pub chr_rom: Option<Cow<'a, [u8]>>,
}

fn bit_at(num: u8, idx: u8) -> bool {
    (num >> idx) & 1 == 1
}

fn parse_header(header_data: &[u8]) -> Result<Header> {
    let magic_bytes = header_data[0..4].try_into()?;
    if magic_bytes != MAGIC_BYTES {
        return Err(Error::MagicBytesMismatch(magic_bytes));
    }

    // Get the required bytes from the byte slice
    let num_prg_rom_chunk = header_data[4];
    let num_chr_rom_chunk = header_data[5];

    // Calculate the actual size in bytes
    let prg_rom_size = (num_prg_rom_chunk as usize) * PRG_ROM_CHUNK_SIZE;
    let chr_rom_size = (num_chr_rom_chunk as usize) * CHR_ROM_CHUNK_SIZE;

    // Check if the appropriate bits are set
    let four_screen_vram = bit_at(header_data[6], 3);

    let vram_layout = if four_screen_vram {
        VramLayout::FourScreen
    } else if bit_at(header_data[6], 0) {
        VramLayout::VerticalMirroring
    } else {
        VramLayout::HorizontalMirroring
    };
    let has_persistent_memory = bit_at(header_data[6], 1);
    let has_trainer = bit_at(header_data[6], 2);

    // Combine the upper bits of each byte to one mapper number
    let mapper_number = (header_data[7] & 0x0F) | (header_data[6] >> 4);

    Ok(Header {
        prg_rom_size,
        chr_rom_size,
        vram_layout,
        has_persistent_memory,
        has_trainer,
        mapper_number,
    })
}

impl<'a> Ines<'a> {
    /// Parse a INES ROM from a byte slice
    pub fn from_bytes(data: &'a [u8]) -> Result<Self> {
        // It doesn't matter whether we use the first 16 bytes or the whole thing
        // The function will ignore any data after the first 7 bytes or so anyway
        let header = parse_header(data)?;

        // Get a reference to the trainer (if the ROM even has one)
        let (after_position, trainer) = if header.has_trainer {
            let trainer = &data[HEADER_SIZE..HEADER_SIZE + TRAINER_SIZE];

            (HEADER_SIZE + TRAINER_SIZE, Some(Cow::Borrowed(trainer)))
        } else {
            (HEADER_SIZE, None)
        };

        // Get a reference to the PRG ROM
        let prg_rom = Cow::Borrowed(&data[after_position..after_position + header.prg_rom_size]);

        // Get a reference to the CHR ROM
        let chr_rom = if header.chr_rom_size > 0 {
            let after_prg_rom = after_position + header.prg_rom_size;

            Some(Cow::Borrowed(
                &data[after_prg_rom..after_prg_rom + header.chr_rom_size],
            ))
        } else {
            None
        };

        Ok(Ines {
            header,
            trainer,
            prg_rom,
            chr_rom,
        })
    }

    /// Parse a INES ROM from a file stream
    pub fn from_reader<T: Read>(input_stream: &mut T) -> Result<Self> {
        let mut header = [0; HEADER_SIZE];
        input_stream.read_exact(&mut header)?;

        let header = parse_header(&header)?;

        // Read the trainer (if the ROM even has one)
        let trainer = if header.has_trainer {
            let mut trainer: [u8; TRAINER_SIZE] = [0; TRAINER_SIZE];
            input_stream.read_exact(&mut trainer)?;

            Some(Cow::Owned(trainer.to_vec()))
        } else {
            None
        };

        // Read the PRG ROM
        let mut prg_rom = vec![0; header.prg_rom_size as usize];
        input_stream.read_exact(&mut prg_rom)?;
        let prg_rom = Cow::Owned(prg_rom);

        // Read the CHR ROM
        let chr_rom = if header.chr_rom_size > 0 {
            let mut chr_rom = vec![0; header.chr_rom_size as usize];
            input_stream.read_exact(&mut chr_rom)?;

            Some(Cow::Owned(chr_rom))
        } else {
            None
        };

        Ok(Ines {
            header,
            trainer,
            prg_rom,
            chr_rom,
        })
    }
}
