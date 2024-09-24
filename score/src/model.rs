use crate::data::ImageBatch;
use burn::prelude::*;
use burn::serde::Serialize;
use burn::tensor::backend::AutodiffBackend;
use burn::{
    config::Config,
    module::Module,
    nn::{LeakyRelu, LeakyReluConfig, Linear, LinearConfig},
    train::{RegressionOutput, TrainOutput, TrainStep, ValidStep},
};
use clap::builder::OsStr;
use clap::ValueEnum;
use nn::loss::{HuberLossConfig, Reduction};
use resnet_burn::ResNet;
use serde::Deserialize;

#[derive(Module, Debug)]
pub(crate) struct ScoreModel<B: Backend> {
    resnet: ResNet<B>,
    leaky_relu: LeakyRelu,
    fc: Linear<B>,
}

impl<B: Backend> ScoreModel<B> {
    /// # Shapes
    ///   - Images [batch_size, 1024, 1024, 3]
    ///   - Output [batch_size, 1]
    pub fn forward(&self, datas: Tensor<B, 4>) -> Tensor<B, 2> {
        let x = self.resnet.forward(datas); // [batch_size, 1]
        let x = self.leaky_relu.forward(x);
        self.fc.forward(x)
    }

    fn forward_regression(
        &self,
        datas: Tensor<B, 4>,
        target_scores: Tensor<B, 2>,
    ) -> RegressionOutput<B> {
        let output = self.forward(datas);
        let loss = HuberLossConfig::new(10.0).init().forward(
            output.clone(),
            target_scores.clone(),
            Reduction::Auto,
        );
        RegressionOutput::new(loss, output, target_scores)
    }
}

impl<B: AutodiffBackend> TrainStep<ImageBatch<B>, RegressionOutput<B>> for ScoreModel<B> {
    fn step(&self, batch: ImageBatch<B>) -> TrainOutput<RegressionOutput<B>> {
        let reg = self.forward_regression(batch.datas, batch.target_scores);
        TrainOutput::new(self, reg.loss.backward(), reg)
    }
}

impl<B: Backend> ValidStep<ImageBatch<B>, RegressionOutput<B>> for ScoreModel<B> {
    fn step(&self, batch: ImageBatch<B>) -> RegressionOutput<B> {
        self.forward_regression(batch.datas, batch.target_scores)
    }
}

#[derive(Config, Debug)]
pub struct ScoreModelConfig {
    rnn_type: RnnType,
}

impl ScoreModelConfig {
    pub(crate) fn init<B: Backend>(&self, device: &B::Device) -> ScoreModel<B> {
        let resnet = match self.rnn_type {
            RnnType::Layer18 => ResNet::resnet18(512, device),
            RnnType::Layer34 => ResNet::resnet34(512, device),
            RnnType::Layer50 => ResNet::resnet50(512, device),
            RnnType::Layer101 => ResNet::resnet101(512, device),
            RnnType::Layer152 => ResNet::resnet152(512, device),
        };
        ScoreModel {
            resnet,
            leaky_relu: LeakyReluConfig::new().init(),
            fc: LinearConfig::new(512, 1).init(device),
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, ValueEnum)]
pub enum RnnType {
    Layer18 = 18,
    Layer34 = 34,
    Layer50 = 50,
    #[default]
    Layer101 = 101,
    Layer152 = 152,
}

impl From<RnnType> for OsStr {
    fn from(value: RnnType) -> Self {
        format!("layer{:?}", value as usize).into()
    }
}
