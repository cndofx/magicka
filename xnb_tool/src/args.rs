use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
pub struct Args {
    #[command(subcommand)]
    pub subcommand: Subcommands,
}

#[derive(Subcommand, Debug)]
pub enum Subcommands {
    /// Extract an XNB file or a directory containing XNB files
    Extract {
        /// File or directory to extract from
        input: String,

        /// Directory to extract to
        output: String,

        /// Overwrite existing files
        #[arg(short, long)]
        overwrite: bool,

        /// Dump raw decompressed data in addition to the regular output
        #[arg(short, long)]
        dump_raw: bool,

        /// Save output as MessagePack instead of JSON
        #[arg(short, long)]
        msgpack: bool,

        /// Compression level applied to the output [0 - 9]
        #[arg(short, long, default_value_t = 0, value_parser = compression_level_range)]
        compression_level: u8,
    },
}

fn compression_level_range(s: &str) -> Result<u8, String> {
    let level = s
        .parse()
        .map_err(|_| format!("invalid compression level: {s}"))?;
    if level > 9 {
        Err("level must be between 0 and 9".to_string())
    } else {
        Ok(level)
    }
}
