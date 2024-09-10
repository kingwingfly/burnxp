use crate::components::{Footer, Images, Quit, Render, Title};
use crate::event::{Compare, Event, CMPDISPATCHER};
use crate::sort::OrdPath;
use crate::state::{CurrentScreen, PROCESS};
use crate::terminal::AutoDropTerminal;
use crate::utils::{bincode_from, bincode_into, centered_rect, json_into};
use anyhow::Result;
use crossterm::event::{self, Event as TermEvent, KeyCode, KeyEventKind};
use mime_guess::MimeGuess;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::Frame;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::path::PathBuf;
use std::sync::atomic::Ordering as AtomicOrdering;
use std::thread;

pub struct App {
    current_screen: CurrentScreen,
    cmp: Option<Compare>,
}

impl App {
    pub fn new(root: PathBuf, output: PathBuf, cache: PathBuf) -> Self {
        let images = walkdir::WalkDir::new(root)
            .into_iter()
            .filter_map(|res| res.ok())
            .filter_map(|e| match MimeGuess::from_path(e.path()).first() {
                Some(mime) if mime.type_() == "image" => Some(e.path().to_path_buf()),
                _ => None,
            })
            .collect::<Vec<_>>();
        PROCESS.total.store(images.len(), AtomicOrdering::Relaxed);
        thread::spawn(move || -> Result<()> {
            let mut btree: BinaryHeap<OrdPath> = BinaryHeap::new();
            // SAFETY: This avoids rebuilding the binary heap.
            // It's safe because binary heap has the same memory layout as Vec.
            unsafe {
                let data_ptr = &mut btree as *mut BinaryHeap<OrdPath> as *mut Vec<OrdPath>;
                data_ptr.replace(bincode_from(&cache)?);
            }
            for path in images.into_iter() {
                if btree.iter().any(|x| x.path == path) {
                    PROCESS.finished.fetch_add(1, AtomicOrdering::Relaxed);
                    continue;
                }
                btree.push(OrdPath::new(path));
                if PROCESS.finished.fetch_add(1, AtomicOrdering::Relaxed) & 0b111 == 0 {
                    bincode_into(&cache, &btree)?;
                }
            }
            bincode_into(&cache, &btree)?;
            json_into(&output, &btree.into_sorted_vec())?;
            CMPDISPATCHER.req_tx.send(Event::Finished)?;
            Ok(())
        });
        Self {
            current_screen: CurrentScreen::Main,
            cmp: None,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        let mut terminal = AutoDropTerminal::new()?;
        // recv the first compare
        self.recv_event()?;
        'a: loop {
            terminal.draw(|f| {
                self.render(f, f.area()).ok();
            })?;
            while let TermEvent::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Release {
                    // Skip events that are not KeyEventKind::Press
                    continue;
                }
                match self.current_screen {
                    CurrentScreen::Main => match key.code {
                        KeyCode::Char('q') => {
                            self.current_screen = CurrentScreen::Exiting;
                        }
                        KeyCode::Left | KeyCode::Char('=') | KeyCode::Right
                            if self.cmp.is_some() =>
                        {
                            match key.code {
                                KeyCode::Left => self.resp_event(Ordering::Less)?,
                                KeyCode::Char('=') => self.resp_event(Ordering::Equal)?,
                                KeyCode::Right => self.resp_event(Ordering::Greater)?,
                                _ => {}
                            }
                            self.recv_event()?;
                        }
                        _ => continue,
                    },
                    CurrentScreen::Finished => match key.code {
                        KeyCode::Char('q') => self.current_screen = CurrentScreen::Exiting,
                        _ => continue,
                    },
                    CurrentScreen::Exiting => match key.code {
                        KeyCode::Char('y') => break 'a Ok(()),
                        _ => {
                            self.current_screen = match self.cmp {
                                Some(_) => CurrentScreen::Main,
                                None => CurrentScreen::Finished,
                            }
                        }
                    },
                }
                break;
            }
        }
    }

    fn recv_event(&mut self) -> Result<()> {
        match CMPDISPATCHER.req_rx.recv().unwrap() {
            Event::Compare(cmp) => self.cmp = Some(cmp),
            Event::Finished => {
                self.cmp = None;
                self.current_screen = CurrentScreen::Finished;
            }
        }
        Ok(())
    }

    fn resp_event(&self, ord: Ordering) -> Result<()> {
        CMPDISPATCHER.resp_tx.send(ord)?;
        Ok(())
    }
}

impl Render for App {
    fn render(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        if CurrentScreen::Exiting == self.current_screen {
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
            }
            None => {
                Title {
                    title: "The comparation finished.".to_string(),
                }
                .render(f, chunks[1])?;
            }
        }
        Footer {
            current_screen: self.current_screen,
        }
        .render(f, chunks[2])?;
        Ok(())
    }
}
