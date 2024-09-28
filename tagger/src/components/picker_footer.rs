use super::Render;
use crate::state::{CurrentScreen, PICKER_PROCESS};
use anyhow::Result;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::sync::atomic::Ordering;

pub(crate) struct PickerFooter {
    pub current_screen: CurrentScreen,
}

impl Render for PickerFooter {
    fn render(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(33), Constraint::Percentage(67)])
            .split(area);
        PickerNavigation {
            current_screen: self.current_screen,
        }
        .render(f, chunks[0])?;
        PickerHint {
            current_screen: self.current_screen,
        }
        .render(f, chunks[1])?;
        Ok(())
    }
}

struct PickerNavigation {
    current_screen: CurrentScreen,
}

struct PickerHint {
    current_screen: CurrentScreen,
}

impl Render for PickerNavigation {
    fn render(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let current_navigation_text = vec![
            match self.current_screen {
                CurrentScreen::Main => Span::styled("Picke", Style::default().fg(Color::Cyan)),
                CurrentScreen::Finished => {
                    Span::styled("Finished", Style::default().fg(Color::Green))
                }
                CurrentScreen::Exiting => Span::styled("Exiting", Style::default().fg(Color::Red)),
            },
            Span::styled(" | ", Style::default().fg(Color::White)),
            match self.current_screen {
                CurrentScreen::Main => Span::styled(
                    format!(
                        "{} page: {}",
                        *PICKER_PROCESS,
                        PICKER_PROCESS.finished.load(Ordering::Relaxed) / 9,
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
            },
        ];
        let mode_footer = Paragraph::new(Line::from(current_navigation_text))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(mode_footer, area);
        Ok(())
    }
}

impl Render for PickerHint {
    fn render(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let hint = {
            match self.current_screen {
                CurrentScreen::Main => Span::styled(
                    "Pick(Enter) Choose(1-9) Jump(j) Last(<-) Next(->) quit(q)",
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
        f.render_widget(key_notes_footer, area);
        Ok(())
    }
}
