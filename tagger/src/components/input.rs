use anyhow::Result;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::Paragraph,
    Frame,
};

use super::Render;

#[derive(Debug)]
pub(crate) struct NumInput {
    pub num: usize,
}

impl Render for NumInput {
    fn render(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);
        let title = Paragraph::new("Page to go:");
        f.render_widget(title, chunks[0]);
        let input = Paragraph::new(format!("{}", self.num));
        f.render_widget(input, chunks[1]);
        Ok(())
    }
}
