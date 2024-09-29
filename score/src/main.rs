use burn::{
    backend::{libtorch::LibTorchDevice, Autodiff, LibTorch},
    optim::AdamConfig,
};
use clap::{CommandFactory as _, Parser, Subcommand};
use clap_complete::{generate, Shell};
use score::{predict, train, Output, PredictConfig, ResNetType, ScoreModelConfig, TrainingConfig};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Cli {
    #[clap(subcommand)]
    subcmd: SubCmd,
}

#[derive(Debug, Subcommand)]
enum SubCmd {
    /// Train a ResNet model with scores produced by the tagger.
    Train {
        /// Model type
        #[arg(short, long, default_value=ResNetType::default())]
        model: ResNetType,
        /// Path to the training set json file produced by the tagger divide
        #[arg(short, long, default_value = "train.json")]
        train_set: PathBuf,
        /// Path to the validation set json file produced by the tagger divide
        #[arg(short, long, default_value = "valid.json")]
        valid_set: PathBuf,
        /// Directory to save artifacts (The directory will be recreated if it exists)
        #[arg(short, long, default_value = "burnxp_artifact")]
        artifact_dir: PathBuf,
        #[arg(short, long, default_value = "128")]
        num_epochs: usize,
        #[arg(short, long, default_value = "1")]
        batch_size: usize,
        /// Number of workers for data loading
        #[arg(short = 'w', long, default_value = "1")]
        num_workers: usize,
        #[arg(short, long, default_value = "1.0e-4")]
        learning_rate: f64,
        /// Number of epochs before allowing early stopping
        #[arg(short, long, default_value = "20")]
        early_stopping: usize,
        /// Path to the pretrained model checkpoint
        #[arg(short, long)]
        pretrained: Option<PathBuf>,
    },
    /// Predict using a ResNet model checkpoint
    Predict {
        /// Model type
        #[arg(short, long, default_value=ResNetType::Layer101)]
        model: ResNetType,
        /// Path to the model checkpoint
        #[arg(short, long)]
        checkpoint: PathBuf,
        /// Root of test images directory
        #[arg(short, long)]
        input: PathBuf,
        /// Method to output the scores
        #[arg(short, long, default_value = "tty")]
        output: Output,
        #[arg(short, long, default_value = "32")]
        batch_size: usize,
        /// Number of workers for data loading
        #[arg(short = 'w', long, default_value = "8")]
        num_workers: usize,
    },
    /// generate auto completion script
    GenCompletion {
        /// shell name
        shell: Shell,
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
            model,
            train_set,
            valid_set,
            artifact_dir,
            num_epochs,
            batch_size,
            num_workers,
            learning_rate,
            early_stopping,
            pretrained,
        } => {
            train::<MyAutodiffBackend>(
                artifact_dir,
                TrainingConfig::new(
                    ScoreModelConfig::new(model),
                    AdamConfig::new(),
                    train_set,
                    valid_set,
                )
                .with_num_epochs(num_epochs)
                .with_batch_size(batch_size)
                .with_num_workers(num_workers)
                .with_learning_rate(learning_rate)
                .with_pretrained(pretrained)
                .with_early_stopping(early_stopping),
                device,
            );
        }
        SubCmd::Predict {
            model,
            checkpoint,
            input,
            output,
            batch_size,
            num_workers,
        } => predict::<MyBackend>(
            PredictConfig::new(model, checkpoint, input, output)
                .with_batch_size(batch_size)
                .with_num_workers(num_workers),
            device,
        ),
        SubCmd::GenCompletion { shell } => {
            generate(shell, &mut Cli::command(), "score", &mut std::io::stdout());
        }
    }
}
