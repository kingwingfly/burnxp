use anyhow::Result;
use clap::{CommandFactory as _, Parser, Subcommand};
use clap_complete::{generate, Shell};
use std::path::PathBuf;
use tagger::{Divider, Method, Observer, Picker, Tagger};

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
        /// The file to cache the paths of the images which have been picked or rejected
        #[clap(short, long, default_value = "cache.json")]
        cache: PathBuf,
        /// The file ops method to use (note: hardlink is not allowed between different file systems)
        #[clap(short, long, default_value = "softlink")]
        method: Method,
        /// The root directory to scan for images and move from
        from: PathBuf,
        /// The directory to move the images to
        to: PathBuf,
    },
    /// Divide tags.json into train set and validation set in certain ratio
    Divide {
        /// The ratio of the training set
        #[clap(short, long, default_value = "4")]
        train: usize,
        /// The ratio of the validation set
        #[clap(short, long, default_value = "1")]
        valid: usize,
        /// The path to save the training set
        #[clap(short, long, default_value = "train.json")]
        train_path: PathBuf,
        /// The path to save the validation set
        #[clap(short, long, default_value = "valid.json")]
        valid_path: PathBuf,
        /// The path to the tags produced by the tagger tag subcommand
        path: PathBuf,
    },
    /// Observe the consistency of the tags produced by the tagger
    Observe {
        /// The path to the tags produced by the tagger tag subcommand
        path: PathBuf,
    },
    /// generate auto completion script
    GenCompletion {
        /// shell name
        shell: Shell,
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
        SubCmd::Divide {
            train,
            valid,
            train_path,
            valid_path,
            path,
        } => {
            let divider = Divider::new(path, train, valid, train_path, valid_path)?;
            divider.devide()?;
        }
        SubCmd::Observe { path } => {
            let mut observer = Observer::new(path)?;
            observer.run()?;
        }
        SubCmd::GenCompletion { shell } => {
            generate(shell, &mut Cli::command(), "tagger", &mut std::io::stdout());
        }
    }

    Ok(())
}
