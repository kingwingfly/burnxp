use crate::{
    components::{Image, PickerFooter, Quit, Render, Title},
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
    image: Option<PathBuf>,
    method: Method,
    to: PathBuf,
    images: Vec<PathBuf>,
    /// cache those judged
    cache: HashSet<PathBuf>,
    cache_path: PathBuf,
}

#[derive(Debug, Clone, Default, ValueEnum)]
pub enum Method {
    #[default]
    Cp,
    Ln,
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
            cache: json_from(&cache).unwrap_or_default(),
            cache_path: cache,
            images,
            ..Default::default()
        }
    }

    pub fn run(&mut self) -> Result<()> {
        let mut terminal = AutoDropTerminal::new()?;
        loop {
            self.image = self.images.pop();
            if self.image.is_some() && self.cache.contains(self.image.as_ref().unwrap()) {
                continue;
            } else if self.image.is_none() {
                self.current_screen = CurrentScreen::Finished;
            }

            'a: loop {
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
                            _ if self.image.is_some() => {
                                match key.code {
                                    KeyCode::Enter => {
                                        let from = self.image.as_ref().unwrap();
                                        let mut to = self.to.join(from.file_name().unwrap());
                                        let mut i = 0;
                                        while to.exists() {
                                            let name = from.file_name().unwrap();
                                            let new_name =
                                                format!("{}_{}", i, name.to_string_lossy());
                                            to = self.to.join(new_name);
                                            i += 1;
                                        }
                                        match self.method {
                                            Method::Cp => {
                                                fs::copy(from, to)?;
                                            }
                                            Method::Ln => {
                                                #[cfg(target_family = "unix")]
                                                std::os::unix::fs::symlink(from, to)?;
                                                #[cfg(target_family = "windows")]
                                                std::os::windows::fs::symlink_file(from, to)?;
                                            }
                                        }
                                    }
                                    KeyCode::Delete | KeyCode::Backspace => {}
                                    _ => continue,
                                }
                                self.cache.insert(self.image.take().unwrap());
                                break 'a;
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
                                return Ok(());
                            }
                            KeyCode::Char('Y') => {
                                return Ok(());
                            }
                            _ => {
                                self.current_screen = match self.image {
                                    Some(_) => CurrentScreen::Main,
                                    None => CurrentScreen::Finished,
                                }
                            }
                        },
                    }
                    break;
                }
            }
            PICKER_PROCESS
                .finished
                .fetch_add(1, AtomicOrdering::Relaxed);
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
        match self.image {
            Some(ref path) => {
                #[cfg(not(target_os = "windows"))]
                let mut picker = ratatui_image::picker::Picker::from_termios()
                    .map_err(|_| anyhow::anyhow!("Failed to get the picker"))?;
                #[cfg(target_os = "windows")]
                let mut picker = {
                    let mut picker = ratatui_image::picker::Picker::new((12, 24));
                    picker.protocol_type = ratatui_image::picker::ProtocolType::Iterm2;
                    picker
                };
                picker.guess_protocol();
                Image::new(&mut picker, path)?.render(f, chunks[1])?;
            }
            None => {
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
