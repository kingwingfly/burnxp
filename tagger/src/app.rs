use crate::components::{Footer, Images, Quit, Render, Title};
use crate::state::CurrentScreen;
use crate::terminal::AutoDropTerminal;
use crate::utils::centered_rect;
use anyhow::Result;
use crossbeam::channel::{bounded, Receiver, Sender};
use crossterm::event::{self, Event, KeyCode};
use mime_guess::MimeGuess;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::Frame;
use std::cmp::Ordering;
use std::fs::File;
use std::path::PathBuf;
use std::thread;

type Compare = [PathBuf; 2];

pub struct App {
    current_screen: CurrentScreen,
    req_rx: Receiver<Compare>,
    resp_tx: Sender<Ordering>,
    cmp: Option<Compare>,
}

impl App {
    pub fn new(root: String) -> Self {
        let (req_tx, req_rx) = bounded::<Compare>(0);
        let (resp_tx, resp_rx) = bounded::<Ordering>(0);
        thread::spawn(move || {
            let mut images = walkdir::WalkDir::new(root)
                .into_iter()
                .filter_map(|res| res.ok())
                .filter_map(|e| match MimeGuess::from_path(e.path()).first() {
                    Some(mime) if mime.type_() == "image" => Some(e.path().to_path_buf()),
                    _ => None,
                })
                .collect::<Vec<_>>();
            images.sort_by(|p1, p2| {
                req_tx.send([p1.clone(), p2.clone()]).unwrap();
                resp_rx.recv().unwrap()
            });
            serde_json::to_writer_pretty(File::create("tag.json").unwrap(), &images).unwrap();
        });
        Self {
            current_screen: CurrentScreen::Main,
            req_rx,
            resp_tx,
            cmp: None,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        let mut terminal = AutoDropTerminal::new()?;
        // recv the first compare
        self.cmp = self.req_rx.recv().ok();
        let mut render = true;
        loop {
            if render {
                terminal.draw(|f| {
                    self.render(f, f.area()).ok();
                })?;
            }
            render = true;
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Release {
                    render = false;
                    // Skip events that are not KeyEventKind::Press
                    continue;
                }
                match self.current_screen {
                    CurrentScreen::Main => match key.code {
                        KeyCode::Char('q') => {
                            self.current_screen = CurrentScreen::Finished;
                        }
                        KeyCode::Left if self.cmp.is_some() => {
                            self.resp_tx.send(Ordering::Less).unwrap();
                            self.cmp = self.req_rx.recv().ok();
                        }
                        KeyCode::Char('=') if self.cmp.is_some() => {
                            self.resp_tx.send(Ordering::Equal).unwrap();
                            self.cmp = self.req_rx.recv().ok();
                        }
                        KeyCode::Right if self.cmp.is_some() => {
                            self.resp_tx.send(Ordering::Greater).unwrap();
                            self.cmp = self.req_rx.recv().ok();
                        }
                        _ => render = false,
                    },
                    CurrentScreen::Finished => match key.code {
                        KeyCode::Char('y') => break Ok(()),
                        _ => self.current_screen = CurrentScreen::Main,
                    },
                }
            }
        }
    }
}

impl Render for App {
    fn render(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        if CurrentScreen::Finished == self.current_screen {
            let area = centered_rect(60, 25, f.area());
            Quit.render(f, area)?;
            return Ok(());
        }
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Length(3),
            ])
            .split(area);
        Title {
            title: "Tagger".to_string(),
        }
        .render(f, chunks[0])?;
        match self.cmp {
            Some(ref paths) => {
                Images::new(paths)?.render(f, chunks[1])?;
                Footer {
                    current_screen: self.current_screen,
                }
                .render(f, chunks[2])?;
            }
            None => {
                Title {
                    title: "The comparation finished.".to_string(),
                }
                .render(f, chunks[1])?;
                Footer {
                    current_screen: CurrentScreen::Finished,
                }
                .render(f, chunks[2])?;
            }
        }
        Ok(())
    }
}
