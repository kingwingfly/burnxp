use crate::state::{CurrentScreen, PROCESS, PROCESS_WITH_COMPLEXITY};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};
use std::sync::atomic::Ordering;

pub(crate) struct PickerFooter {
    pub current_screen: CurrentScreen,
}

impl Widget for PickerFooter {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(33), Constraint::Fill(1)])
            .split(area);

        PickerNavigation {
            current_screen: self.current_screen,
        }
        .render(chunks[0], buf);
        PickerHint {
            current_screen: self.current_screen,
        }
        .render(chunks[1], buf);
    }
}

struct PickerNavigation {
    current_screen: CurrentScreen,
}

struct PickerHint {
    current_screen: CurrentScreen,
}

impl Widget for PickerNavigation {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let current_navigation_text = vec![
            match self.current_screen {
                CurrentScreen::Main => Span::styled("Picke", Style::default().fg(Color::Cyan)),
                CurrentScreen::Finished => {
                    Span::styled("Finished", Style::default().fg(Color::Green))
                }
                CurrentScreen::Exiting => Span::styled("Exiting", Style::default().fg(Color::Red)),
                _ => unreachable!(),
            },
            Span::styled(" | ", Style::default().fg(Color::White)),
            match self.current_screen {
                CurrentScreen::Main => Span::styled(
                    format!(
                        "{} page: {}/{}",
                        *PROCESS,
                        PROCESS.finished.load(Ordering::Relaxed) / 9,
                        PROCESS.total.load(Ordering::Relaxed) / 9,
                    ),
                    Style::default().fg(Color::LightCyan),
                ),
                CurrentScreen::Finished => Span::styled(
                    "Quit to save the result",
                    Style::default().fg(Color::LightGreen),
                ),
                CurrentScreen::Exiting => {
                    Span::styled("Exiting", Style::default().fg(Color::LightRed))
                }
                _ => unreachable!(),
            },
        ];
        Paragraph::new(Line::from(current_navigation_text))
            .wrap(Wrap { trim: true })
            .block(Block::default().borders(Borders::ALL))
            .render(area, buf);
    }
}

impl Widget for PickerHint {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let hint = {
            match self.current_screen {
                CurrentScreen::Main => Span::styled(
                    "Choose(1-9) Jump(j) Pre(<-) Next(->/↵) Quit(q)",
                    Style::default().fg(Color::Green),
                ),
                CurrentScreen::Finished => {
                    Span::styled("Press (q) to exit", Style::default().fg(Color::LightGreen))
                }
                CurrentScreen::Exiting => Span::styled(
                    "Are you sure you want to exit? (y/n)",
                    Style::default().fg(Color::LightRed),
                ),
                _ => unreachable!(),
            }
        };
        Paragraph::new(hint)
            .wrap(Wrap { trim: true })
            .block(Block::default().borders(Borders::ALL))
            .render(area, buf);
    }
}

pub(crate) struct CmperFooter {
    pub current_screen: CurrentScreen,
}

impl Widget for CmperFooter {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);
        CmperNavigation {
            current_screen: self.current_screen,
        }
        .render(chunks[0], buf);
        CmperHint {
            current_screen: self.current_screen,
        }
        .render(chunks[1], buf);
    }
}

struct CmperNavigation {
    current_screen: CurrentScreen,
}

struct CmperHint {
    current_screen: CurrentScreen,
}

impl Widget for CmperNavigation {
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
                _ => unreachable!(),
            },
            Span::styled(" | ", Style::default().fg(Color::White)),
            match self.current_screen {
                CurrentScreen::Main => Span::styled(
                    format!("{}", *PROCESS_WITH_COMPLEXITY),
                    Style::default().fg(Color::LightCyan),
                ),
                CurrentScreen::Finished => Span::styled(
                    "Quit to save the result",
                    Style::default().fg(Color::LightGreen),
                ),
                CurrentScreen::Exiting => {
                    Span::styled("Exiting", Style::default().fg(Color::LightRed))
                }
                _ => unreachable!(),
            },
        ];
        Paragraph::new(Line::from(current_navigation_text))
            .wrap(Wrap { trim: true })
            .block(Block::default().borders(Borders::ALL))
            .render(area, buf);
    }
}

impl Widget for CmperHint {
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
                _ => unreachable!(),
            }
        };
        Paragraph::new(hint)
            .block(Block::default().borders(Borders::ALL))
            .wrap(Wrap { trim: true })
            .render(area, buf);
    }
}

pub(crate) struct TaggerFooter {
    pub current_screen: CurrentScreen,
}

impl Widget for TaggerFooter {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(33), Constraint::Fill(1)])
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
                CurrentScreen::Main => Span::styled("Tag", Style::default().fg(Color::Cyan)),
                CurrentScreen::Finished => {
                    Span::styled("Finished", Style::default().fg(Color::Green))
                }
                CurrentScreen::Exiting => Span::styled("Exiting", Style::default().fg(Color::Red)),
                _ => unreachable!(),
            },
            Span::styled(" | ", Style::default().fg(Color::White)),
            match self.current_screen {
                CurrentScreen::Main => Span::styled(
                    format!("{}", *PROCESS),
                    Style::default().fg(Color::LightCyan),
                ),
                CurrentScreen::Finished => Span::styled(
                    "Quit to save the result",
                    Style::default().fg(Color::LightGreen),
                ),
                CurrentScreen::Exiting => {
                    Span::styled("Exiting", Style::default().fg(Color::LightRed))
                }
                _ => unreachable!(),
            },
        ];
        Paragraph::new(Line::from(current_navigation_text))
            .block(Block::default().borders(Borders::ALL))
            .wrap(Wrap { trim: true })
            .render(area, buf);
    }
}

impl Widget for TaggerHint {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let hint = {
            match self.current_screen {
                CurrentScreen::Main => Span::styled(
                    "(Un)Tag(1-9) Pre(<-) Next(->/↵) ModifyTag(n) Quit(q)",
                    Style::default().fg(Color::Green),
                ),
                CurrentScreen::Finished => {
                    Span::styled("Press (q) to exit", Style::default().fg(Color::LightGreen))
                }
                CurrentScreen::Exiting => Span::styled(
                    "Are you sure you want to exit? (y/n)",
                    Style::default().fg(Color::LightRed),
                ),
                _ => unreachable!(),
            }
        };
        Paragraph::new(hint)
            .block(Block::default().borders(Borders::ALL))
            .wrap(Wrap { trim: true })
            .render(area, buf);
    }
}
