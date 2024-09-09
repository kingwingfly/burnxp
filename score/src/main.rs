use std::path::PathBuf;

use burn::{
    backend::{libtorch::LibTorchDevice, Autodiff, LibTorch},
    optim::AdamConfig,
};
use clap::Parser;
use score::{train, ScoreModelConfig, TrainingConfig};

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Cli {
    #[arg(short, long)]
    train_set: PathBuf,
    #[arg(short, long)]
    valid_set: PathBuf,
}

type MyBackend = LibTorch<f32, i8>;
type MyAutodiffBackend = Autodiff<MyBackend>;

fn main() {
    #[cfg(target_os = "macos")]
    let device = LibTorchDevice::Mps;
    #[cfg(not(target_os = "macos"))]
    let device = LibTorchDevice::Cuda(0);

    let args = Cli::parse();

    train::<MyAutodiffBackend>(
        "/tmp/score",
        TrainingConfig::new(
            ScoreModelConfig::new(),
            AdamConfig::new(),
            args.train_set,
            args.valid_set,
        ),
        device,
    );
}
