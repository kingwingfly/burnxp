mod footer;
mod image;
mod quit;
mod title;

use anyhow::Result;
use ratatui::{layout::Rect, Frame};

pub(crate) use footer::Footer;
pub(crate) use image::Images;
pub(crate) use quit::Quit;
pub(crate) use title::Title;

pub(crate) trait Render {
    fn render(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()>;
}
