use burn::{
    backend::{libtorch::LibTorchDevice, Autodiff, LibTorch},
    optim::AdamConfig,
};
use clap::{Parser, Subcommand};
use score::{train, ScoreModelConfig, TrainingConfig};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Cli {
    #[clap(subcommand)]
    subcmd: SubCmd,
}

#[derive(Debug, Subcommand)]
enum SubCmd {
    /// Train a model
    Train {
        /// Path to the training set json file produced by the tagger
        #[arg(short, long)]
        train_set: PathBuf,
        /// Path to the validation set json file produced by the tagger
        #[arg(short, long)]
        valid_set: PathBuf,
        /// Directory to save artifacts
        #[arg(short, long)]
        artifact_dir: PathBuf,
    },
    /// Predict using a model
    Predict {
        /// Path to the model checkpoint
        #[arg(short, long)]
        checkpoint: PathBuf,
        /// Path to the input image
        #[arg(short, long)]
        input: PathBuf,
        /// Path to the output image
        #[arg(short, long)]
        output: PathBuf,
    },
}

type MyBackend = LibTorch<f32, i8>;
type MyAutodiffBackend = Autodiff<MyBackend>;

fn main() {
    #[cfg(target_os = "macos")]
    let device = LibTorchDevice::Mps;
    #[cfg(not(target_os = "macos"))]
    let device = LibTorchDevice::Cuda(0);

    let args = Cli::parse();
    match args.subcmd {
        SubCmd::Train {
            train_set,
            valid_set,
            artifact_dir,
        } => {
            train::<MyAutodiffBackend>(
                artifact_dir,
                TrainingConfig::new(
                    ScoreModelConfig::new(),
                    AdamConfig::new(),
                    train_set,
                    valid_set,
                ),
                device,
            );
        }
        SubCmd::Predict {
            checkpoint,
            input,
            output,
        } => {
            println!("Predicting with checkpoint: {:?}", checkpoint);
            println!("Input image: {:?}", input);
            println!("Output image: {:?}", output);
        }
    }
}
