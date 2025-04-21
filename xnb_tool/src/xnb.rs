use std::io::Read;

use byteorder::{LittleEndian, ReadBytesExt};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Platform {
    Windows,
    WindowsPhone,
    Xbox360,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Version {
    XNA31,
    XNA40,
}

#[derive(Debug)]
pub struct Header {
    platform: Platform,
    version: Version,
    hi_def: bool,
    compressed: bool,
    compressed_size: u32,
    uncompressed_size: u32,
}

pub struct Xnb {
    header: Header,
    data: Vec<u8>,
}

impl Xnb {
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
        let uncompressed_size = reader.read_u32::<LittleEndian>()?;

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
        dbg!(&data);

        let xnb = Xnb { header, data };

        Ok(xnb)
    }
}
