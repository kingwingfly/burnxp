use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tagger::{Method, Picker, Tagger};

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Cli {
    #[clap(subcommand)]
    subcmd: SubCmd,
}

#[derive(Debug, Subcommand)]
enum SubCmd {
    /// Tag images
    Tag {
        /// The output file path
        #[clap(short, long, default_value = "tags.json")]
        output: PathBuf,
        /// The file to cache the comparison results (relative paths are cached)
        #[clap(short, long, default_value = "cache.bin")]
        cache: PathBuf,
        /// The root directory to scan for images
        root: PathBuf,
    },
    /// Pick images
    Pick {
        /// The file to cache the decided results
        #[clap(short, long, default_value = "cache.json")]
        cache: PathBuf,
        /// The method to use
        #[clap(short, long, default_value = "ln")]
        method: Method,
        /// The root directory to scan for images and move from
        from: PathBuf,
        /// The directory to move the images to
        to: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    // create app and run it
    match cli.subcmd {
        SubCmd::Tag {
            output,
            cache,
            root,
        } => {
            let mut tagger = Tagger::new(root, output, cache);
            tagger.run()?;
        }
        SubCmd::Pick {
            cache,
            method,
            from,
            to,
        } => {
            let mut picker = Picker::new(method, cache, from, to);
            picker.run()?;
        }
    }

    Ok(())
}
