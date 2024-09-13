use anyhow::Result;
use burn::{
    data::dataloader::{batcher::Batcher, Dataset},
    prelude::*,
};
use image::{imageops::FilterType, ImageReader};
use std::path::PathBuf;
use std::{fs::File, path::Path};

const SIZE: usize = 720;
const HALF: i64 = (SIZE / 2) as i64;

type Score = i64;

#[derive(Debug, Clone)]
pub(crate) struct ImageData {
    data: Vec<f32>,
    score: Score,
}

impl ImageData {
    pub(crate) fn data<B: Backend>(&self) -> Tensor<B, 1> {
        Tensor::from_data(&self.data[..], &B::Device::default())
    }

    pub(crate) fn score<B: Backend>(&self) -> Tensor<B, 1> {
        Tensor::from_data([self.score as i8], &B::Device::default())
    }
}

pub(crate) struct ImageDataSet {
    inner: Vec<(PathBuf, Score)>,
}

impl ImageDataSet {
    pub(crate) fn train(path: PathBuf) -> Result<Self> {
        Ok(Self {
            inner: serde_json::from_reader(File::open(path)?)?,
        })
    }

    pub(crate) fn test(path: PathBuf) -> Result<Self> {
        Ok(Self {
            inner: serde_json::from_reader(File::open(path)?)?,
        })
    }
}

impl Dataset<ImageData> for ImageDataSet {
    fn get(&self, index: usize) -> Option<ImageData> {
        self.inner.get(index).and_then(|(path, score)| {
            Some(ImageData {
                data: open_image(path)?
                    .into_iter()
                    .map(|p| p as f32 / 255.0)
                    .collect(),
                score: *score,
            })
        })
    }

    fn len(&self) -> usize {
        self.inner.len()
    }
}

#[derive(Clone)]
pub(crate) struct ImageBatcher<B: Backend> {
    device: B::Device,
}

#[derive(Debug, Clone)]
pub(crate) struct ImageBatch<B: Backend> {
    pub datas: Tensor<B, 4>,
    pub target_scores: Tensor<B, 2>,
}

impl<B: Backend> ImageBatcher<B> {
    pub(crate) fn new(device: B::Device) -> Self {
        Self { device }
    }
}

impl<B: Backend> Batcher<ImageData, ImageBatch<B>> for ImageBatcher<B> {
    fn batch(&self, items: Vec<ImageData>) -> ImageBatch<B> {
        let datas = items
            .iter()
            .map(|item| item.data().reshape([1, 3, SIZE, SIZE]))
            .collect::<Vec<_>>();
        let target_scores = items
            .iter()
            .map(|item| item.score().reshape([1, 1]))
            .collect::<Vec<_>>();

        let datas = Tensor::cat(datas, 0).to_device(&self.device);
        let target_scores = Tensor::cat(target_scores, 0).to_device(&self.device);

        ImageBatch {
            datas,
            target_scores,
        }
    }
}

fn open_image(path: impl AsRef<Path>) -> Option<Vec<u8>> {
    let size = SIZE as u32;
    let img = ImageReader::open(path.as_ref()).ok()?.decode().ok()?;
    let mut background = image::RgbImage::new(size, size);

    let factor = img.height().max(img.width()) / size;
    let nheight = (img.height() / factor).max(size);
    let nwidth = (img.width() / factor).max(size);

    let img = img.resize(nwidth, nheight, FilterType::Nearest);
    let img = img.to_rgb8();

    image::imageops::overlay(
        &mut background,
        &img,
        HALF - (nwidth / 2) as i64,
        HALF - (nheight / 2) as i64,
    );

    Some(background.into_raw().to_vec())
}
