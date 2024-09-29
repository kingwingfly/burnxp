use crate::{
    components::{Grid, NumInput, PickerFooter, Quit, Render, Title},
    state::{CurrentScreen, PICKER_PROCESS},
    terminal::AutoDropTerminal,
    utils::{centered_rect, json_from, json_into},
};
use anyhow::Result;
use clap::ValueEnum;
use crossterm::event::{self, Event as TermEvent, KeyCode, KeyEventKind};
use mime_guess::MimeGuess;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};
use std::{
    collections::HashSet, fs, path::PathBuf, sync::atomic::Ordering as AtomicOrdering,
    time::Duration,
};

#[derive(Debug, Default)]
pub struct Picker {
    current_screen: CurrentScreen,
    buffer: Vec<PathBuf>,
    chosen: [bool; 9],
    page: usize,
    method: Method,
    to: PathBuf,
    images: Vec<PathBuf>,
    /// cache those picked
    cache: HashSet<PathBuf>,
    cache_path: PathBuf,
}

#[derive(Debug, Clone, Default, ValueEnum)]
pub enum Method {
    #[default]
    Cp,
    SoftLink,
    HardLink,
    Move,
}

impl Picker {
    pub fn new(method: Method, cache: PathBuf, from: PathBuf, to: PathBuf) -> Self {
        fs::create_dir_all(&to).unwrap();
        let images = walkdir::WalkDir::new(from)
            .into_iter()
            .filter_map(|res| res.ok())
            .filter_map(|e| match MimeGuess::from_path(e.path()).first() {
                Some(mime) if mime.type_() == "image" => Some(e.into_path()),
                _ => None,
            })
            .collect::<Vec<_>>();
        PICKER_PROCESS
            .total
            .store(images.len(), AtomicOrdering::Relaxed);

        Self {
            method,
            to,
            buffer: Vec::with_capacity(9),
            cache: json_from(&cache).unwrap_or_default(),
            cache_path: cache,
            images,
            ..Default::default()
        }
    }

    pub fn run(&mut self) -> Result<()> {
        let mut terminal = AutoDropTerminal::new()?;
        loop {
            PICKER_PROCESS
                .finished
                .store(9 * self.page, AtomicOrdering::Relaxed);
            if 9 * self.page + 1 > self.images.len() {
                self.current_screen = CurrentScreen::Finished;
            } else {
                self.buffer = self.images.chunks(9).nth(self.page).unwrap().to_vec();
                for (i, p) in self.buffer.iter().enumerate() {
                    if self.cache.contains(p) {
                        self.chosen[i] = true;
                    }
                }
            }

            'l: loop {
                terminal.draw(|f| {
                    self.render(f, f.area()).ok();
                })?;
                while let TermEvent::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Release {
                        // Skip events that are not KeyEventKind::Press
                        continue;
                    }
                    if event::poll(Duration::from_millis(50))? {
                        continue;
                    }
                    match self.current_screen {
                        CurrentScreen::Main => match key.code {
                            KeyCode::Char('q') => {
                                self.current_screen = CurrentScreen::Exiting;
                            }
                            _ if !self.buffer.is_empty() => match key.code {
                                KeyCode::Char(c) if c.is_numeric() => {
                                    let i = c.to_digit(10).unwrap() as usize;
                                    if i > 0 && i <= self.buffer.len() {
                                        self.chosen[i - 1] = !self.chosen[i - 1];
                                        if self.chosen[i - 1] {
                                            self.cache.insert(self.buffer[i - 1].clone());
                                        } else {
                                            self.cache.remove(&self.buffer[i - 1]);
                                        }
                                    }
                                }
                                KeyCode::Char('j') => self.buffer.clear(), // empty means jump page
                                _ => {
                                    match key.code {
                                        KeyCode::Enter | KeyCode::Right => self.page += 1,
                                        KeyCode::Left => self.page = self.page.saturating_sub(1),
                                        _ => continue,
                                    }
                                    self.chosen = [false; 9];
                                    self.buffer.clear();
                                    break 'l;
                                }
                            },
                            KeyCode::Char(c) if c.is_numeric() => {
                                self.page = self.page * 10 + c.to_digit(10).unwrap() as usize;
                            }
                            KeyCode::Backspace => self.page /= 10,
                            KeyCode::Enter => {
                                self.page =
                                    self.page.clamp(0, self.images.len().saturating_sub(1) / 9);
                                self.current_screen = CurrentScreen::Main;
                                self.chosen = [false; 9];
                                break 'l;
                            }
                            _ => continue,
                        },
                        CurrentScreen::Finished => match key.code {
                            KeyCode::Char('q') => self.current_screen = CurrentScreen::Exiting,
                            _ => continue,
                        },
                        CurrentScreen::Exiting => match key.code {
                            KeyCode::Char('y') => {
                                json_into(&self.cache_path, &self.cache)?;
                                for from in self.cache.iter() {
                                    let mut to = self.to.join(from.file_name().unwrap());
                                    if to.exists()
                                        && to.canonicalize().unwrap()
                                            == from.canonicalize().unwrap()
                                    {
                                        continue;
                                    }
                                    let mut i = 0;
                                    while to.exists() {
                                        let name = from.file_name().unwrap();
                                        let new_name = format!("{}_{}", i, name.to_string_lossy());
                                        to = self.to.join(new_name);
                                        i += 1;
                                    }
                                    match self.method {
                                        Method::Cp => fs::copy(from, to).map(|_| {})?,
                                        Method::SoftLink => {
                                            #[cfg(target_family = "unix")]
                                            std::os::unix::fs::symlink(from.canonicalize()?, to)?;
                                            #[cfg(target_family = "windows")]
                                            std::os::windows::fs::symlink_file(
                                                from.canonicalize()?,
                                                to,
                                            )?;
                                        }
                                        Method::HardLink => fs::hard_link(from, to)?,
                                        Method::Move => fs::rename(from, to)?,
                                    }
                                }
                                return Ok(());
                            }
                            KeyCode::Char('Y') => {
                                return Ok(());
                            }
                            _ => {
                                self.current_screen = match self.buffer.is_empty() {
                                    false => CurrentScreen::Main,
                                    true => CurrentScreen::Finished,
                                }
                            }
                        },
                    }
                    break;
                }
            }
        }
    }
}

impl Render for Picker {
    fn render(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        if CurrentScreen::Exiting == self.current_screen {
            let area = centered_rect(60, 25, f.area());
            Quit.render(f, area)?;
            return Ok(());
        }
        if CurrentScreen::Main == self.current_screen && self.buffer.is_empty() {
            let area = centered_rect(60, 25, f.area());
            NumInput { num: self.page }.render(f, area)?;
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
            title: "Picker".to_string(),
        }
        .render(f, chunks[0])?;
        match self.buffer.is_empty() {
            false => Grid::new(self.buffer.as_slice(), self.chosen)?.render(f, chunks[1])?,
            true => {
                Title {
                    title: "The picking finished.".to_string(),
                }
                .render(f, chunks[1])?;
            }
        }
        PickerFooter {
            current_screen: self.current_screen,
        }
        .render(f, chunks[2])?;
        Ok(())
    }
}
