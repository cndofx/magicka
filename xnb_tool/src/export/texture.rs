use bcndecode::{BcnDecoderFormat, BcnEncoding};
use image::{ExtendedColorType, ImageEncoder, codecs::png::PngEncoder};

use crate::content::texture::Texture2D;

impl Texture2D {
    pub fn to_png(&self) -> anyhow::Result<Vec<u8>> {
        let decompressed = if self.format == 0x1C {
            bcndecode::decode(
                &self.mips[0],
                self.width as usize,
                self.height as usize,
                BcnEncoding::Bc1,
                BcnDecoderFormat::RGBA,
            )?
        } else if self.format == 0x20 {
            bcndecode::decode(
                &self.mips[0],
                self.width as usize,
                self.height as usize,
                BcnEncoding::Bc3,
                BcnDecoderFormat::RGBA,
            )?
        } else {
            anyhow::bail!("unknown texture format (self.unk): {}", self.format);
        };

        let mut png = Vec::new();
        let encoder = PngEncoder::new(&mut png);
        encoder.write_image(
            &decompressed,
            self.width,
            self.height,
            ExtendedColorType::Rgba8,
        )?;

        Ok(png)
    }
}
