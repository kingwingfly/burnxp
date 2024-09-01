use crate::indicator::IndicatorSet;
use burn::{
    data::dataloader::{batcher::Batcher, Dataset},
    prelude::*,
};

#[derive(Debug, Clone)]
pub(crate) struct PicData {
    indicators: IndicatorSet,
}

pub(crate) struct PicDataSet {
    inner: Vec<IndicatorSet>,
}

impl Dataset<PicData> for PicDataSet {
    fn get(&self, index: usize) -> Option<PicData> {
        self.inner.get(index).map(|indicators| PicData {
            indicators: *indicators,
        })
    }

    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl PicDataSet {
    pub(crate) fn train() -> Self {
        Self {
            inner: [IndicatorSet {
                age: 1,
                body: 2,
                emotion: 3,
                face: 4,
                hair: 5,
                outfit: 6,
                pose: 7,
                xp: 8,
                score: 9,
            }]
            .repeat(64),
        }
    }

    pub(crate) fn test() -> Self {
        Self {
            inner: vec![IndicatorSet {
                age: 1,
                body: 2,
                emotion: 3,
                face: 4,
                hair: 5,
                outfit: 6,
                pose: 7,
                xp: 8,
                score: 9,
            }],
        }
    }
}

#[derive(Clone)]
pub(crate) struct PicBatcher<B: Backend> {
    device: B::Device,
}

#[derive(Debug, Clone)]
pub(crate) struct PicBatch<B: Backend> {
    pub(crate) datas: Tensor<B, 2>,
    pub(crate) target_scores: Tensor<B, 2>,
}

impl<B: Backend> PicBatcher<B> {
    pub(crate) fn new(device: B::Device) -> Self {
        Self { device }
    }
}

impl<B: Backend> Batcher<PicData, PicBatch<B>> for PicBatcher<B> {
    fn batch(&self, items: Vec<PicData>) -> PicBatch<B> {
        let datas = items
            .iter()
            .map(|item| item.indicators.to_tensor().reshape([1, 8]))
            .collect();
        let target_scores = items
            .iter()
            .map(|item| item.indicators.score().reshape([1, 1]))
            .collect();

        let datas = Tensor::cat(datas, 0).to_device(&self.device);
        let target_scores = Tensor::cat(target_scores, 0).to_device(&self.device);

        PicBatch {
            datas,
            target_scores,
        }
    }
}
