use std::{ffi::OsStr, fs::File, io::BufReader, path::Path};

use anyhow::Context;
use args::{Args, Subcommands};
use clap::Parser;
use walkdir::WalkDir;
use xnb_tool::xnb::{ExtractOptions, Xnb};

mod args;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.subcommand {
        Subcommands::Extract {
            input,
            output,
            overwrite,
            dump_raw,
            msgpack,
            compression_level,
        } => {
            let options = ExtractOptions {
                overwrite,
                dump_raw,
                msgpack,
                compression_level,
            };
            extract(&input, &output, &options)
                .with_context(|| format!("failed to extract {input}"))?;
        }
    }

    Ok(())
}

fn extract(
    input_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
    options: &ExtractOptions,
) -> anyhow::Result<()> {
    let input_path = input_path.as_ref();
    let output_path = output_path.as_ref();

    if !input_path.try_exists()? {
        anyhow::bail!("input path {} does not exist", input_path.display());
    }

    if input_path.is_file() {
        extract_file(
            input_path,
            output_path.join(input_path.file_name().unwrap()),
            options,
        )?;
    } else if input_path.is_dir() {
        extract_directory(input_path, output_path, options)?;
    } else {
        todo!();
    }

    Ok(())
}

fn extract_directory(
    input_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
    options: &ExtractOptions,
) -> anyhow::Result<()> {
    let input_path = input_path.as_ref();
    let output_path = output_path.as_ref();

    let mut successes = 0;
    let mut failures = Vec::new();

    for entry in WalkDir::new(input_path) {
        let entry = match entry {
            Ok(v) => v,
            Err(e) => {
                eprintln!("failed to read entry: {e}");
                continue;
            }
        };

        if entry.path().is_dir() {
            continue;
        }

        if entry.path().extension() != Some(OsStr::new("xnb")) {
            eprintln!("\nskipping non xnb file: {}", entry.path().display());
            continue;
        }

        let relative_path = entry.path().strip_prefix(input_path)?;
        eprintln!("\nextracting entry: {}", relative_path.display());

        if let Err(e) = extract_file(entry.path(), output_path.join(relative_path), options) {
            failures.push(relative_path.display().to_string());
            eprintln!("failed to extract entry: {e}");
            for (i, cause) in e.chain().enumerate() {
                eprintln!("  {i}: {cause}");
            }
        } else {
            successes += 1;
        }
    }

    println!("\nextracted {successes} files");
    if !failures.is_empty() {
        println!("failed to extract {} files:", failures.len());
        for f in &failures {
            println!("  {f}");
        }
    }

    Ok(())
}

fn extract_file(
    input_file_path: impl AsRef<Path>,
    output_file_path: impl AsRef<Path>,
    options: &ExtractOptions,
) -> anyhow::Result<()> {
    let file = File::open(input_file_path).context("failed to open file")?;
    let mut reader = BufReader::new(file);
    let xnb = Xnb::parse(&mut reader).context("failed to parse xnb header")?;
    xnb.extract(output_file_path, options)
        .context("failed to extract xnb")?;
    Ok(())
}
