use super::Render;
use anyhow::Result;
use image::ImageReader;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};
use ratatui_image::{picker::Picker, Resize, StatefulImage};
use std::path::PathBuf;

pub(crate) struct Images<'a> {
    picker: Picker,
    paths: &'a [&'a PathBuf; 2],
}

impl<'a> Images<'a> {
    pub(crate) fn new(paths: &'a [&'a PathBuf; 2]) -> Result<Self> {
        #[cfg(not(target_os = "windows"))]
        let mut picker =
            Picker::from_termios().map_err(|_| anyhow::anyhow!("Failed to get the picker"))?;
        #[cfg(target_os = "windows")]
        let mut picker = {
            let mut picker = Picker::new((12, 24));
            picker.protocol_type = ratatui_image::picker::ProtocolType::Iterm2;
            picker
        };
        picker.guess_protocol();
        Ok(Self { picker, paths })
    }
}

impl<'a> Render for Images<'a> {
    fn render(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);
        for i in 0..2 {
            Image {
                picker: &mut self.picker,
                path: self.paths[i],
            }
            .render(f, chunks[i])?
        }
        Ok(())
    }
}

pub struct Image<'a> {
    picker: &'a mut Picker,
    path: &'a PathBuf,
}

impl<'a> Image<'a> {
    pub(crate) fn new(picker: &'a mut Picker, path: &'a PathBuf) -> Result<Self> {
        Ok(Self { picker, path })
    }
}

impl Render for Image<'_> {
    fn render(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let dyn_img = match self.path.is_symlink() {
            true => ImageReader::open(self.path.read_link()?)?.decode()?,
            false => ImageReader::open(self.path)?.decode()?,
        };
        let mut image_fit_state = self.picker.new_resize_protocol(dyn_img);
        let image = StatefulImage::new(None).resize(Resize::Fit(None));
        f.render_stateful_widget(image, area, &mut image_fit_state);
        Ok(())
    }
}
