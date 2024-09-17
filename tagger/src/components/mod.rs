mod image;
mod picker_footer;
mod quit;
mod tagger_footer;
mod title;

use anyhow::Result;
use ratatui::{layout::Rect, Frame};

pub(crate) use image::{Image, Images};
pub(crate) use picker_footer::PickerFooter;
pub(crate) use quit::Quit;
pub(crate) use tagger_footer::TaggerFooter;
pub(crate) use title::Title;

pub(crate) trait Render {
    fn render(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()>;
}
