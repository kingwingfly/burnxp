use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Padding, Paragraph, Widget, Wrap},
};

pub(crate) struct Quit;

impl Widget for Quit {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let popup_block = Block::default()
            .title("Y/N")
            .padding(Padding::horizontal(3))
            .borders(Borders::NONE)
            .style(Style::default().bg(Color::DarkGray));
        let exit_text = Text::styled(
            "Quit? (y/n) without cache (Y)",
            Style::default().fg(Color::Red),
        );
        let exit_paragraph = Paragraph::new(exit_text)
            .block(popup_block)
            .wrap(Wrap { trim: false });
        exit_paragraph.render(area, buf);
    }
}
