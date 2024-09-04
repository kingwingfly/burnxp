use super::Component;
use anyhow::{anyhow, Result};
use image::ImageReader;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};
use ratatui_image::{picker::Picker, Resize, StatefulImage};
use std::path::PathBuf;

pub(crate) struct Images {
    picker: Picker,
    path: [PathBuf; 2],
}

impl Images {
    pub(crate) fn new(path: &[&str; 2]) -> Result<Self> {
        let path = [PathBuf::from(path[0]), PathBuf::from(path[1])];
        #[cfg(not(target_os = "windows"))]
        let mut picker = Picker::from_termios().map_err(|_| anyhow!("Failed to get the picker"))?;
        #[cfg(target_os = "windows")]
        let mut picker = Picker::new((7, 14));
        picker.guess_protocol();
        Ok(Self { picker, path })
    }
}

impl Component for Images {
    fn render(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);
        for i in 0..2 {
            Image {
                picker: &mut self.picker,
                path: self.path[i].clone(),
            }
            .render(f, chunks[i])?
        }
        Ok(())
    }
}

struct Image<'a> {
    picker: &'a mut Picker,
    path: PathBuf,
}

impl Component for Image<'_> {
    fn render(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let dyn_img = ImageReader::open(&self.path)?.decode()?;
        let mut image_fit_state = self.picker.new_resize_protocol(dyn_img);
        let image = StatefulImage::new(None).resize(Resize::Fit(None));
        f.render_stateful_widget(image, area, &mut image_fit_state);
        Ok(())
    }
}
