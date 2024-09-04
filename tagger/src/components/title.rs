use super::Component;
use anyhow::Result;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub(crate) struct Title {
    pub(crate) title: String,
}

impl Component for Title {
    fn render(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let title_block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default());
        let title = Paragraph::new(Text::styled(&self.title, Style::default().fg(Color::Green)))
            .block(title_block);
        f.render_widget(title, area);
        Ok(())
    }
}
