use burn::{
    backend::libtorch::{LibTorch, LibTorchDevice},
    backend::Autodiff,
    optim::AdamConfig,
};
use model::ModelConfig;
use train::TrainingConfig;

mod data;
mod model;
mod tag;
mod train;

fn main() {
    type MyBackend = LibTorch<f32, i8>;
    type MyAutodiffBackend = Autodiff<MyBackend>;
    #[cfg(target_os = "macos")]
    let device = LibTorchDevice::Metal(0);
    #[cfg(not(target_os = "macos"))]
    let device = LibTorchDevice::Cuda(0);

    train::train::<MyAutodiffBackend>(
        "/tmp/guide",
        TrainingConfig::new(ModelConfig::new(10, 512), AdamConfig::new()),
        device,
    );
}
