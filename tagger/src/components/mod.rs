mod histogram;
mod image;
mod input;
mod picker_footer;
mod quit;
mod scorer_footer;
mod title;

use anyhow::Result;
use ratatui::{layout::Rect, Frame};

pub(crate) use histogram::Histogram;
pub(crate) use image::{Grid, Images};
pub(crate) use input::NumInput;
pub(crate) use picker_footer::PickerFooter;
pub(crate) use quit::Quit;
pub(crate) use scorer_footer::ScorerFooter;
pub(crate) use title::Title;

pub(crate) trait Render {
    fn render(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()>;
}
