use burn::{
    backend::{libtorch::LibTorchDevice, Autodiff, LibTorch},
    optim::AdamConfig,
};
use score::{train, ScoreModelConfig, TrainingConfig};

fn main() {
    type MyBackend = LibTorch<f32, i8>;
    type MyAutodiffBackend = Autodiff<MyBackend>;
    #[cfg(target_os = "macos")]
    let device = LibTorchDevice::Mps;
    #[cfg(not(target_os = "macos"))]
    let device = LibTorchDevice::Cuda(0);

    train::<MyAutodiffBackend>(
        "/tmp/score",
        TrainingConfig::new(ScoreModelConfig::new(), AdamConfig::new()),
        device,
    );
}
