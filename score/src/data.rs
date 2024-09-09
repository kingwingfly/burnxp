use burn::{
    data::dataloader::{batcher::Batcher, Dataset},
    prelude::*,
};
use std::fs::File;
use std::path::PathBuf;

type Score = f32;

#[derive(Debug, Clone)]
pub(crate) struct ImageData {
    data: [[f32; 1024 * 1024]; 3],
    score: f32,
}

impl ImageData {
    pub(crate) fn data<B: Backend>(&self) -> Tensor<B, 3> {
        Tensor::from_data(self.data, &B::Device::default())
    }

    pub(crate) fn score<B: Backend>(&self) -> Tensor<B, 1> {
        Tensor::from_data([self.score], &B::Device::default())
    }
}

pub(crate) struct ImageDataSet {
    inner: Vec<(PathBuf, Score)>,
}

impl ImageDataSet {
    pub(crate) fn train(path: PathBuf) -> Self {
        Self {
            inner: serde_json::from_reader(File::open(path).unwrap()).unwrap(),
        }
    }

    pub(crate) fn test(path: PathBuf) -> Self {
        Self {
            inner: serde_json::from_reader(File::open(path).unwrap()).unwrap(),
        }
    }
}

impl Dataset<ImageData> for ImageDataSet {
    fn get(&self, index: usize) -> Option<ImageData> {
        self.inner.get(index).map(|(_path, score)| ImageData {
            // todo read image
            data: [[0.0; 1024 * 1024]; 3],
            score: *score,
        })
    }

    fn len(&self) -> usize {
        self.inner.len()
    }
}

#[derive(Clone)]
pub(crate) struct PicBatcher<B: Backend> {
    device: B::Device,
}

#[derive(Debug, Clone)]
pub(crate) struct PicBatch<B: Backend> {
    pub(crate) datas: Tensor<B, 4>,
    pub(crate) target_scores: Tensor<B, 2>,
}

impl<B: Backend> PicBatcher<B> {
    pub(crate) fn new(device: B::Device) -> Self {
        Self { device }
    }
}

impl<B: Backend> Batcher<ImageData, PicBatch<B>> for PicBatcher<B> {
    fn batch(&self, items: Vec<ImageData>) -> PicBatch<B> {
        let datas = items
            .iter()
            .map(|item| item.data().reshape([1, 1024, 1024, 3]))
            .collect();
        let target_scores = items
            .iter()
            .map(|item| item.score().reshape([1, 1]))
            .collect();

        let datas = Tensor::cat(datas, 0).to_device(&self.device);
        let target_scores = Tensor::cat(target_scores, 0).to_device(&self.device);

        PicBatch {
            datas,
            target_scores,
        }
    }
}
