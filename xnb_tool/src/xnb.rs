use std::{
    fs::File,
    io::{Cursor, Read, Seek, Write},
    path::Path,
};

use anyhow::Context;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use flate2::{Compression, write::ZlibEncoder};
use image::{ExtendedColorType, ImageEncoder, codecs::png::PngEncoder};
use lzxd::Lzxd;
use serde::{Deserialize, Serialize};

use crate::{content::Content, ext::MyReadBytesExt};

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

    pub fn parse(reader: &mut impl Read) -> anyhow::Result<Self> {
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

        let mut data = Vec::with_capacity(header.compressed_size as usize);
        reader.read_to_end(&mut data)?;

        let xnb = Xnb { header, data };

        Ok(xnb)
    }

    pub fn extract(
        &self,
        file_path: impl AsRef<Path>,
        options: &ExtractOptions,
    ) -> anyhow::Result<()> {
        let directory = file_path.as_ref().parent().unwrap();
        if !directory.try_exists()? {
            std::fs::create_dir_all(directory)
                .with_context(|| format!("failed to create directory {}", directory.display()))?;
        }

        let raw = if self.header.compressed {
            self.decompress()
                .context("failed to decompress xnb content")?
        } else {
            // TODO: find a way to avoid the clone, maybe COW?
            self.data.clone()
        };

        if options.dump_raw {
            let file_path = file_path.as_ref().with_extension("raw");
            let exists = file_path.try_exists()?;
            if exists && !options.overwrite {
                anyhow::bail!("{} already exists", file_path.display());
            }
            let mut file = File::create(&file_path)?;
            file.write_all(&raw)?;
            eprintln!("saved to {}", file_path.display());
        }

        let mut reader = Cursor::new(&raw);
        let content = XnbContent::parse(&mut reader)?;

        let extension = match content.primary_content {
            Content::Null => {
                eprintln!("WARNING: null content");
                return Ok(());
            }
            Content::String(..) => "string",
            Content::Item(..) => "item",
            Content::Character(..) => "character",
            Content::Texture2D(..) => "texture2d",
            Content::Model(..) => "model",
            Content::VertexDeclaration(..) => "vertexdecl",
            Content::VertexBuffer(..) => "vertexbuffer",
            Content::IndexBuffer(..) => "indexbuffer",
            Content::RenderDeferredEffect(..) => "renderdeferredeffect",
        };

        let extension = if options.msgpack {
            format!("{}.msgpack", extension)
        } else {
            format!("{}.json", extension)
        };

        let file_path = file_path.as_ref().with_extension(extension);
        let exists = file_path.try_exists()?;
        if exists && !options.overwrite {
            anyhow::bail!("{} already exists", file_path.display());
        }

        let mut file = File::create(&file_path)?;

        let serialized = if options.msgpack {
            rmp_serde::to_vec(&content)?
        } else {
            serde_json::to_string_pretty(&content)
                .context("failed to serialize content")?
                .into_bytes()
        };

        if options.compression_level > 0 {
            let mut encoder = ZlibEncoder::new(
                &mut file,
                Compression::new(options.compression_level as u32),
            );
            encoder.write_all(&serialized)?;
            encoder.finish()?;
        } else {
            file.write_all(&serialized)?;
        };

        eprintln!("saved to {}", file_path.display());

        if let Content::Texture2D(texture) = &content.primary_content {
            let file_path = file_path.with_extension("png");
            let exists = file_path.try_exists()?;
            if exists && !options.overwrite {
                anyhow::bail!("{} already exists", file_path.display());
            }
            let mut file = File::create(&file_path).context("failed to create image file")?;

            let pixels = texture.decompress()?;
            let encoder = PngEncoder::new(&mut file);
            encoder.write_image(
                &pixels,
                texture.width,
                texture.height,
                ExtendedColorType::Rgba8,
            )?;

            eprintln!("saved to {}", file_path.display());
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

#[derive(Serialize, Deserialize, Debug)]
pub struct XnbContent {
    pub readers: Vec<TypeReader>,
    pub primary_content: Content,
    pub shared_content: Vec<Content>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TypeReader {
    pub name: String,
    pub version: i32,
}

impl XnbContent {
    pub fn parse(reader: &mut impl Read) -> anyhow::Result<Self> {
        let reader_count = reader.read_7bit_encoded_i32()?;
        let mut readers = Vec::with_capacity(reader_count as usize);
        for _ in 0..reader_count {
            let name = reader.read_7bit_length_string()?;
            let version = reader.read_i32::<LittleEndian>()?;
            let reader = TypeReader { name, version };
            readers.push(reader);
        }

        let shared_content_count = reader.read_7bit_encoded_i32()?;

        let primary_content = Content::read(reader, &readers)?;

        let mut shared_content = Vec::with_capacity(shared_content_count as usize);
        for _ in 0..shared_content_count {
            let content = Content::read(reader, &readers)?;
            shared_content.push(content);
        }

        let mut rem = Vec::new();
        reader.read_to_end(&mut rem)?;
        if rem.len() != 0 {
            eprintln!("WARNING: {} bytes left in XNB", rem.len());
            // dbg!(&rem);
        }

        let content = XnbContent {
            readers,
            shared_content,
            primary_content,
        };
        Ok(content)
    }
}

pub struct ExtractOptions {
    pub overwrite: bool,
    pub dump_raw: bool,
    pub msgpack: bool,
    pub compression_level: u8,
}
