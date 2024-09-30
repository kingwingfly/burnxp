use crate::state::{CurrentScreen, TAGGER_PROCESS};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

pub(crate) struct ScorerFooter {
    pub current_screen: CurrentScreen,
}

impl Widget for ScorerFooter {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);
        TaggerNavigation {
            current_screen: self.current_screen,
        }
        .render(chunks[0], buf);
        TaggerHint {
            current_screen: self.current_screen,
        }
        .render(chunks[1], buf);
    }
}

struct TaggerNavigation {
    current_screen: CurrentScreen,
}

struct TaggerHint {
    current_screen: CurrentScreen,
}

impl Widget for TaggerNavigation {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let current_navigation_text = vec![
            match self.current_screen {
                CurrentScreen::Main => {
                    Span::styled("Which is better?", Style::default().fg(Color::Cyan))
                }
                CurrentScreen::Finished => {
                    Span::styled("Finished", Style::default().fg(Color::Green))
                }
                CurrentScreen::Exiting => Span::styled("Exiting", Style::default().fg(Color::Red)),
            },
            Span::styled(" | ", Style::default().fg(Color::White)),
            match self.current_screen {
                CurrentScreen::Main => Span::styled(
                    format!("{}", *TAGGER_PROCESS),
                    Style::default().fg(Color::LightCyan),
                ),
                CurrentScreen::Finished => Span::styled(
                    "Quit to save the result",
                    Style::default().fg(Color::LightGreen),
                ),
                CurrentScreen::Exiting => {
                    Span::styled("Exiting", Style::default().fg(Color::LightRed))
                }
            },
        ];
        let mode_footer = Paragraph::new(Line::from(current_navigation_text))
            .block(Block::default().borders(Borders::ALL));
        mode_footer.render(area, buf);
    }
}

impl Widget for TaggerHint {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let hint = {
            match self.current_screen {
                CurrentScreen::Main => Span::styled(
                    "left(↑/<-) equal(=/↵) right(->/↓) quit(q)",
                    Style::default().fg(Color::Green),
                ),
                CurrentScreen::Finished => {
                    Span::styled("Press (q) to exit", Style::default().fg(Color::LightGreen))
                }
                CurrentScreen::Exiting => Span::styled(
                    "Are you sure you want to exit? (y/n)",
                    Style::default().fg(Color::LightRed),
                ),
            }
        };
        let key_notes_footer = Paragraph::new(hint).block(Block::default().borders(Borders::ALL));
        key_notes_footer.render(area, buf);
    }
}
