use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use anyhow::Context;
use args::{Args, Subcommands};
use clap::Parser;
use walkdir::WalkDir;
use xnb_tool::xnb::Xnb;

mod args;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    dbg!(&args);

    match args.subcommand {
        Subcommands::Extract {
            input,
            output,
            overwrite,
        } => {
            extract(&input, &output, overwrite)
                .with_context(|| format!("failed to extract {input}"))?;
        }
    }

    Ok(())
}

fn extract(
    input_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
    overwrite: bool,
) -> anyhow::Result<()> {
    let input_path = input_path.as_ref();
    let output_path = output_path.as_ref();

    if !input_path.try_exists()? {
        anyhow::bail!("input path {} does not exist", input_path.display());
    }

    std::fs::create_dir_all(output_path).context("failed to create output directory")?;

    if input_path.is_file() {
        extract_file(
            input_path,
            output_path.join(input_path.file_name().unwrap()),
            overwrite,
        )?;
    } else if input_path.is_dir() {
        extract_directory(input_path, output_path, overwrite)?;
    } else {
        todo!();
    }

    Ok(())
}

fn extract_directory(
    input_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
    overwrite: bool,
) -> anyhow::Result<()> {
    let input_path = input_path.as_ref();
    let output_path = output_path.as_ref();

    for entry in WalkDir::new(input_path) {
        let entry = match entry {
            Ok(v) => v,
            Err(e) => {
                eprintln!("failed to read entry: {e}");
                continue;
            }
        };

        let relative_path = entry.path().strip_prefix(input_path)?;
        dbg!(entry.path(), relative_path);

        eprintln!("extracting entry: {}", relative_path.display());

        extract_file(entry.path(), output_path.join(relative_path), overwrite)?;
        // if let Err(e) = extract_file(entry.path(), output_path, overwrite) {
        //     eprintln!("failed to extract entry: {e}");
        //     for cause in e.chain() {
        //         eprintln!("because: {cause}");
        //     }
        // }
    }

    Ok(())
}

fn extract_file(
    input_file_path: impl AsRef<Path>,
    output_file_path: impl AsRef<Path>,
    overwrite: bool,
) -> anyhow::Result<()> {
    let file = File::open(input_file_path).context("failed to open file")?;
    let reader = BufReader::new(file);
    let xnb = Xnb::parse(reader).context("failed to parse xnb header")?;
    xnb.extract(output_file_path, overwrite)
        .context("failed to extract xnb")?;
    Ok(())
}
