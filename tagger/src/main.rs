use anyhow::Result;
use clap::{CommandFactory as _, Parser, Subcommand};
use clap_complete::{generate, Shell};
use std::path::PathBuf;
#[cfg(feature = "cmper")]
use tagger::Cmper;
#[cfg(feature = "observer")]
use tagger::Observer;
use tagger::{Divider, Method, Picker, Tagger};

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Cli {
    #[clap(subcommand)]
    subcmd: SubCmd,
}

#[derive(Debug, Subcommand)]
enum SubCmd {
    /// Score images by tagging them
    Tag {
        /// The file to store and cache the tag results
        #[clap(short, long, default_value = "tags.json")]
        output: PathBuf,
        /// The root directory to scan for images
        root: PathBuf,
    },
    /// Pick images
    Pick {
        /// The file to cache the paths of the images which have been picked
        #[clap(short, long, default_value = "cache.json")]
        cache: PathBuf,
        /// The file ops method to use (note: hardlink is not allowed between different file systems)
        #[clap(short, long, default_value = "soft-link")]
        method: Method,
        /// The root directory to scan for images and mv/cp from
        from: PathBuf,
        /// The directory to mv/cp the images to
        to: PathBuf,
    },
    /// Divide scores.json into train set and validation set in certain ratio
    Divide {
        /// The ratio of the training set
        #[clap(short, long, default_value = "9")]
        train: u32,
        /// The ratio of the validation set
        #[clap(short, long, default_value = "1")]
        valid: u32,
        /// The output path of the training set
        #[clap(long, default_value = "train.json")]
        train_path: PathBuf,
        /// The output path of the validation set
        #[clap(long, default_value = "valid.json")]
        valid_path: PathBuf,
        /// The path to the scores produced by the tagger tag/cmp subcommand
        path: PathBuf,
    },
    /// generate auto completion script
    GenCompletion {
        /// shell name
        shell: Shell,
    },
    /// Observe the consistency of the scores produced by the scorer
    #[cfg(feature = "observer")]
    Observe {
        /// The path to the scores produced by the tagger score subcommand
        path: PathBuf,
    },
    /// Score image-groups by comparing them (deprecated)
    #[cfg(feature = "cmper")]
    Cmp {
        /// The file to cache the comparison results (relative paths are cached)
        #[clap(short, long, default_value = "cache.bin")]
        cache: PathBuf,
        /// The output file path
        #[clap(short, long, default_value = "scores.json")]
        output: PathBuf,
        /// The root directory to scan for images
        root: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.subcmd {
        SubCmd::Tag { output, root } => {
            let mut tagger = Tagger::new(root, output);
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
        SubCmd::Divide {
            train,
            valid,
            train_path,
            valid_path,
            path,
        } => {
            let divider = Divider::new(path, train, valid, train_path, valid_path)?;
            divider.divide()?;
        }
        SubCmd::GenCompletion { shell } => {
            generate(shell, &mut Cli::command(), "tagger", &mut std::io::stdout());
        }
        #[cfg(feature = "observer")]
        SubCmd::Observe { path } => {
            let mut observer = Observer::new(path)?;
            observer.run()?;
        }
        #[cfg(feature = "cmper")]
        SubCmd::Cmp {
            cache,
            output,
            root,
        } => {
            let mut cmper = Cmper::new(root, output, cache);
            cmper.run()?;
        }
    }

    Ok(())
}
