use std::{fs::File, io::BufReader};

use args::{Args, Subcommands};
use clap::Parser;
use xnb_tool::xnb::Xnb;

mod args;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    dbg!(&args);

    match args.subcommand {
        Subcommands::Extract { input, output } => {
            extract(&input, &output)?;
            todo!();
        }
    }

    Ok(())
}

fn extract(input: &str, output: &str) -> anyhow::Result<()> {
    let file = File::open(input)?;
    let reader = BufReader::new(file);
    let xnb = Xnb::parse(reader)?;

    todo!();
}
