use std::io::Read;

use bcndecode::{BcnDecoderFormat, BcnEncoding};
use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Texture2D {
    pub unk: u32,
    pub width: u32,
    pub height: u32,
    pub mips: Vec<Vec<u8>>,
}

impl Texture2D {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let unk = reader.read_u32::<LittleEndian>()?;
        let width = reader.read_u32::<LittleEndian>()?;
        let height = reader.read_u32::<LittleEndian>()?;
        let mip_count = reader.read_u32::<LittleEndian>()?;
        let mut mips = Vec::with_capacity(mip_count as usize);
        for _ in 0..mip_count {
            let size = reader.read_u32::<LittleEndian>()?;
            let mut mip = vec![0u8; size as usize];
            reader.read_exact(&mut mip)?;
            mips.push(mip);
        }
        Ok(Texture2D {
            unk,
            width,
            height,
            mips,
        })
    }

    pub fn decompress(&self) -> anyhow::Result<Vec<u8>> {
        let pixels = bcndecode::decode(
            &self.mips[0],
            self.width as usize,
            self.height as usize,
            BcnEncoding::Bc1,
            BcnDecoderFormat::RGBA,
        )?;
        Ok(pixels)
    }
}
