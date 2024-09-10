use crate::data::PicBatch;
use crate::resnet::ResNet;
use burn::prelude::*;
use burn::tensor::backend::AutodiffBackend;
use burn::{
    config::Config,
    module::Module,
    train::{RegressionOutput, TrainOutput, TrainStep, ValidStep},
};
use nn::loss::{HuberLossConfig, Reduction};

#[derive(Module, Debug)]
pub(crate) struct ScoreModel<B: Backend> {
    resnet: ResNet<B>,
}

impl<B: Backend> ScoreModel<B> {
    /// # Shapes
    ///   - Images [batch_size, 1024, 1024, 3]
    ///   - Output [batch_size, 1]
    fn forward(&self, datas: Tensor<B, 4>) -> Tensor<B, 2> {
        self.resnet.forward(datas) // [batch_size, 1]
    }

    fn forward_regression(
        &self,
        datas: Tensor<B, 4>,
        target_scores: Tensor<B, 2>,
    ) -> RegressionOutput<B> {
        let output = self.forward(datas);
        let loss = HuberLossConfig::new(0.5).init().forward(
            output.clone(),
            target_scores.clone(),
            Reduction::Auto,
        );
        RegressionOutput::new(loss, output, target_scores)
    }
}

impl<B: AutodiffBackend> TrainStep<PicBatch<B>, RegressionOutput<B>> for ScoreModel<B> {
    fn step(&self, batch: PicBatch<B>) -> TrainOutput<RegressionOutput<B>> {
        let item = self.forward_regression(batch.datas, batch.target_scores);
        TrainOutput::new(self, item.loss.backward(), item)
    }
}

impl<B: Backend> ValidStep<PicBatch<B>, RegressionOutput<B>> for ScoreModel<B> {
    fn step(&self, batch: PicBatch<B>) -> RegressionOutput<B> {
        self.forward_regression(batch.datas, batch.target_scores)
    }
}

#[derive(Config, Debug)]
pub struct ScoreModelConfig {}

impl ScoreModelConfig {
    pub(crate) fn init<B: Backend>(&self, device: &B::Device) -> ScoreModel<B> {
        ScoreModel {
            resnet: ResNet::resnet101(1, device),
        }
    }
}
