use crate::components::Title;
use anyhow::{anyhow, Result};
use image::DynamicImage;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, StatefulWidget, Widget},
};
use ratatui_image::{picker::Picker, FilterType, Resize, StatefulImage};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::thread;

pub(crate) struct Images<'a> {
    picker: Arc<RwLock<Picker>>,
    paths: &'a [&'a PathBuf; 2],
}

impl<'a> Images<'a> {
    pub(crate) fn new(paths: &'a [&'a PathBuf; 2]) -> Self {
        #[cfg(not(target_os = "windows"))]
        let mut picker = Picker::from_termios()
            .map_err(|_| anyhow::anyhow!("Failed to get the picker"))
            .unwrap();
        #[cfg(target_os = "windows")]
        let mut picker = {
            let mut picker = Picker::new((12, 24));
            picker.protocol_type = ratatui_image::picker::ProtocolType::Iterm2;
            picker
        };
        picker.guess_protocol();
        Self {
            picker: Arc::new(RwLock::new(picker)),
            paths,
        }
    }
}

impl<'a> Widget for Images<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);
        let mut jhs = Vec::with_capacity(2);
        for i in 0..2 {
            let path = self.paths[i].clone();
            let picker = self.picker.clone();
            jhs.push(thread::spawn(move || -> Result<Image> {
                let image = image::open(&path)?;
                Ok(Image::new(picker, image, path))
            }))
        }
        for (jh, &chunk) in jhs.into_iter().zip(chunks.iter()) {
            if let Ok(Ok(w)) = jh.join() {
                w.render(chunk, buf);
            }
        }
    }
}

pub struct Grid<'a> {
    picker: Arc<RwLock<Picker>>,
    paths: &'a [PathBuf],
    heighlight: [bool; 9],
}

impl<'a> Grid<'a> {
    pub(crate) fn new(paths: &'a [PathBuf], heighlight: [bool; 9]) -> Self {
        #[cfg(not(target_os = "windows"))]
        let mut picker = ratatui_image::picker::Picker::from_termios()
            .map_err(|_| anyhow::anyhow!("Failed to get the picker"))
            .unwrap();
        #[cfg(target_os = "windows")]
        let mut picker = {
            let mut picker = ratatui_image::picker::Picker::new((12, 24));
            picker.protocol_type = ratatui_image::picker::ProtocolType::Iterm2;
            picker
        };
        picker.guess_protocol();
        Self {
            picker: Arc::new(RwLock::new(picker)),
            paths,
            heighlight,
        }
    }
}

impl<'a> Widget for Grid<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let grid = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ])
            .split(area)
            .iter()
            .flat_map(|&line| {
                Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(33),
                        Constraint::Percentage(33),
                        Constraint::Percentage(33),
                    ])
                    .split(line)
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let mut jhs = Vec::with_capacity(9);
        for path in self.paths {
            let path = path.clone();
            let picker = self.picker.clone();
            jhs.push(thread::spawn(move || -> Result<Image> {
                let image = image::open(&path)?;
                Ok(Image::new(picker, image, path.clone()))
            }))
        }
        for (i, jh) in jhs.into_iter().enumerate() {
            let block = Block::default()
                .title(format!("{}", i + 1))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(match self.heighlight[i] {
                    true => Color::Green,
                    false => Color::White,
                }));
            let inner = block.inner(grid[i]);
            block.render(grid[i], buf);
            if let Ok(Ok(w)) = jh.join() {
                w.render(inner, buf);
            }
        }
    }
}

struct Image {
    picker: Arc<RwLock<Picker>>,
    image: DynamicImage,
    path: PathBuf,
}

impl Image {
    pub(crate) fn new(picker: Arc<RwLock<Picker>>, image: DynamicImage, path: PathBuf) -> Self {
        Self {
            picker,
            image,
            path,
        }
    }
}

impl Widget for Image {
    fn render(self, area: Rect, buf: &mut Buffer) {
        fn inner(this: Image, area: Rect, buf: &mut Buffer) -> Result<()> {
            let mut picker = this
                .picker
                .write()
                .map_err(|_| anyhow!("Race condition of Picker RwLock"))?;
            let mut image_fit_state = picker.new_resize_protocol(this.image);
            let image = StatefulImage::new(None).resize(Resize::Fit(Some(FilterType::Nearest)));
            image.render(area, buf, &mut image_fit_state);
            Ok(())
        }
        let path = self.path.clone();
        inner(self, area, buf)
            .or_else(|e| -> Result<()> {
                Title {
                    title: format!("{}: {}", path.display(), e),
                }
                .render(area, buf);
                Ok(())
            })
            .ok();
    }
}
