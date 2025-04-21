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
        #[arg(short, long)]
        output: String,
    },
}
