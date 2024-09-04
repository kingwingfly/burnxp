use super::Component;
use anyhow::Result;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Padding, Paragraph, Wrap},
    Frame,
};

pub(crate) struct Quit;

impl Component for Quit {
    fn render(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let popup_block = Block::default()
            .title("Y/N")
            .padding(Padding::horizontal(3))
            .borders(Borders::NONE)
            .style(Style::default().bg(Color::DarkGray));
        let exit_text = Text::styled("Quit? (y/n)", Style::default().fg(Color::Red));
        let exit_paragraph = Paragraph::new(exit_text)
            .block(popup_block)
            .wrap(Wrap { trim: false });
        f.render_widget(exit_paragraph, area);
        Ok(())
    }
}
