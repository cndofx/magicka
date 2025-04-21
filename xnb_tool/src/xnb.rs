use std::{
    fs::File,
    io::{Cursor, Read, Seek, Write},
    path::Path,
};

use anyhow::Context;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use lzxd::Lzxd;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Windows,
    WindowsPhone,
    Xbox360,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Version {
    XNA31,
    XNA40,
}

#[derive(Debug)]
pub struct Header {
    pub platform: Platform,
    pub version: Version,
    pub hi_def: bool,
    pub compressed: bool,
    pub compressed_size: u32,
    pub uncompressed_size: u32,
}

pub struct Xnb {
    header: Header,
    data: Vec<u8>,
}

impl Xnb {
    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn parse(mut reader: impl Read) -> anyhow::Result<Self> {
        let mut magic = [0u8; 3];
        reader.read_exact(&mut magic)?;
        if &magic != b"XNB" {
            anyhow::bail!("not an XNB file");
        }

        let platform = match reader.read_u8()? {
            b'w' => Platform::Windows,
            b'm' => Platform::WindowsPhone,
            b'x' => Platform::Xbox360,
            v => anyhow::bail!("unknown platform: {v}"),
        };

        let version = match reader.read_u8()? {
            4 => Version::XNA31,
            5 => Version::XNA40,
            v => anyhow::bail!("unknown version: {v}"),
        };

        if version != Version::XNA31 {
            anyhow::bail!("unsupported XNA version: {version:?}, only 3.1 is supported");
        }

        let flags = reader.read_u8()?;
        let hi_def = flags & 0x01 != 0;
        let compressed = flags & 0x80 != 0;

        let compressed_size = reader.read_u32::<LittleEndian>()?;
        let uncompressed_size = if compressed {
            reader.read_u32::<LittleEndian>()?
        } else {
            0
        };

        let header = Header {
            platform,
            version,
            hi_def,
            compressed,
            compressed_size,
            uncompressed_size,
        };
        dbg!(&header);

        let mut data = Vec::with_capacity(header.compressed_size as usize);
        reader.read_to_end(&mut data)?;

        let xnb = Xnb { header, data };

        Ok(xnb)
    }

    pub fn extract(&self, file_path: impl AsRef<Path>, overwrite: bool) -> anyhow::Result<()> {
        let file_path = file_path.as_ref();
        let exists = file_path.try_exists()?;
        if exists && !overwrite {
            anyhow::bail!("{} already exists", file_path.display());
        }
        let mut file = File::create(file_path)?;

        if self.header.compressed {
            let decompressed = self.decompress().context("decompression failed")?;
            file.write_all(&decompressed)?;
        } else {
            // this is probably just writing self.data to a file,
            // but i don't have an uncompressed xnb to test with
            todo!();
        }

        Ok(())
    }

    pub fn decompress(&self) -> anyhow::Result<Vec<u8>> {
        let mut data = Cursor::new(&self.data);
        let mut decompressed = Vec::with_capacity(self.header.uncompressed_size as usize);

        let mut lzxd = Lzxd::new(lzxd::WindowSize::KB64);

        while (data.position() as usize) < data.get_ref().len() {
            let frame_size;
            let block_size;
            if data.read_u8()? == 0xFF {
                frame_size = data.read_u16::<BigEndian>()?;
                block_size = data.read_u16::<BigEndian>()?;
            } else {
                data.seek_relative(-1)?;
                block_size = data.read_u16::<BigEndian>()?;
                frame_size = 0x8000;
            }
            dbg!(frame_size, block_size);

            if block_size == 0 || frame_size == 0 {
                break;
            }

            let mut block = vec![0; block_size as usize];
            data.read_exact(&mut block)?;
            let frame = lzxd.decompress_next(&block, frame_size as usize)?;
            decompressed.extend_from_slice(&frame);
        }

        Ok(decompressed)
    }
}
