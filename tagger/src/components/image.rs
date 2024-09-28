use crate::components::Title;

use super::Render;
use anyhow::Result;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders},
    Frame,
};
use ratatui_image::{picker::Picker, FilterType, Resize, StatefulImage};
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

pub struct Grid<'a> {
    picker: Picker,
    paths: &'a [PathBuf],
    heighlight: [bool; 9],
}

impl<'a> Grid<'a> {
    pub(crate) fn new(paths: &'a [PathBuf], heighlight: [bool; 9]) -> Result<Self> {
        #[cfg(not(target_os = "windows"))]
        let mut picker = ratatui_image::picker::Picker::from_termios()
            .map_err(|_| anyhow::anyhow!("Failed to get the picker"))?;
        #[cfg(target_os = "windows")]
        let mut picker = {
            let mut picker = ratatui_image::picker::Picker::new((12, 24));
            picker.protocol_type = ratatui_image::picker::ProtocolType::Iterm2;
            picker
        };
        picker.guess_protocol();
        Ok(Self {
            picker,
            paths,
            heighlight,
        })
    }
}

impl<'a> Render for Grid<'a> {
    fn render(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
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
        for (i, path) in self.paths.iter().enumerate() {
            let block = Block::default()
                .title(format!("{}", i + 1))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(if self.heighlight[i] {
                    Color::Green
                } else {
                    Color::White
                }));
            let inner = block.inner(grid[i]);
            f.render_widget(block, grid[i]);
            Image::new(&mut self.picker, path)?.render(f, inner)?;
        }
        Ok(())
    }
}

struct Image<'a> {
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
        fn inner(this: &mut Image<'_>, f: &mut Frame<'_>, area: Rect) -> Result<()> {
            let dyn_img = image::open(this.path.canonicalize()?)?;
            let mut image_fit_state = this.picker.new_resize_protocol(dyn_img);
            let image = StatefulImage::new(None).resize(Resize::Fit(Some(FilterType::Gaussian)));
            f.render_stateful_widget(image, area, &mut image_fit_state);
            Ok(())
        }
        inner(self, f, area).or_else(|e| {
            Title {
                title: format!("{}: {}", self.path.display(), e),
            }
            .render(f, area)
        })
    }
}
