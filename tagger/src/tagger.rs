use crate::components::{Images, Quit, Render, TaggerFooter, Title};
use crate::event::{ComparePair, Event, CMPDISPATCH};
use crate::matrix::Matrix;
use crate::ordpaths::{CompareResult, OrdPaths};
use crate::state::{CurrentScreen, TAGGER_PROCESS};
use crate::terminal::AutoDropTerminal;
use crate::utils::{bincode_from, bincode_into, centered_rect, json_into};
use anyhow::Result;
use crossterm::event::{self, Event as TermEvent, KeyCode, KeyEventKind};
use mime_guess::MimeGuess;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::Frame;
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::atomic::Ordering as AtomicOrdering;
use std::thread;
use std::time::Duration;

#[derive(Debug)]
pub struct Tagger {
    current_screen: CurrentScreen,
    cmp: Option<ComparePair>,
    /// The matrix of comparison
    matrix: Matrix,
    cache_path: PathBuf,
    images: Vec<OrdPaths>,
}

impl Tagger {
    pub fn new(root: PathBuf, output: PathBuf, cache: PathBuf) -> Self {
        let matrix: Matrix = bincode_from(&cache).unwrap_or_default();
        let images = walkdir::WalkDir::new(root)
            .into_iter()
            .filter_map(|res| res.ok())
            .filter_map(|e| match MimeGuess::from_path(e.path()).first() {
                Some(mime) if mime.type_() == "image" => {
                    let path = e.into_path();
                    matrix.get_paths(&path).or(Some(OrdPaths::new([path])))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        TAGGER_PROCESS
            .total
            .store(images.len(), AtomicOrdering::Relaxed);
        let images_c = images.clone();
        thread::spawn(move || -> Result<()> {
            let mut btree: BTreeSet<OrdPaths> = BTreeSet::new();
            for paths in images_c.into_iter() {
                btree.insert(paths);
                TAGGER_PROCESS
                    .finished
                    .fetch_add(1, AtomicOrdering::Relaxed);
            }
            CMPDISPATCH.req_tx.send(Event::Finished)?;
            let res: Vec<(i64, OrdPaths)> = btree.into_iter().fold(vec![], |mut acc, node| {
                if acc.is_empty() {
                    vec![(0, node)]
                } else {
                    let (last_score, last) = *acc.last().unwrap();
                    CMPDISPATCH
                        .req_tx
                        .send(Event::Compare([node, last]))
                        .unwrap();
                    let delta = CMPDISPATCH.resp_rx.recv().unwrap() as i64;
                    acc.push((last_score + delta, node));
                    acc
                }
            });
            json_into(&output, &res)?;
            CMPDISPATCH.req_tx.send(Event::Finished)?;
            Ok(())
        });
        Self {
            current_screen: CurrentScreen::Main,
            cmp: None,
            matrix,
            cache_path: cache,
            images,
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
                if event::poll(Duration::from_millis(50))? {
                    continue;
                }
                match self.current_screen {
                    CurrentScreen::Main => match key.code {
                        KeyCode::Char('q') => {
                            self.current_screen = CurrentScreen::Exiting;
                        }
                        _ if self.cmp.is_some() => {
                            match key.code {
                                KeyCode::Up => self.resp_event(CompareResult::MuchBetter)?,
                                KeyCode::Left => self.resp_event(CompareResult::Better)?,
                                KeyCode::Right => self.resp_event(CompareResult::Worse)?,
                                KeyCode::Down => self.resp_event(CompareResult::MuchWorse)?,
                                KeyCode::Char('=') | KeyCode::Enter => {
                                    self.resp_event(CompareResult::Same)?
                                }
                                _ => continue,
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
                        KeyCode::Char('y') => {
                            bincode_into(&self.cache_path, &self.matrix)?;
                            break 'a Ok(());
                        }
                        KeyCode::Char('Y') => {
                            break 'a Ok(());
                        }
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
        loop {
            match CMPDISPATCH.req_rx.recv().unwrap() {
                Event::Compare([p1, p2]) => {
                    if let Some(ord) = self.matrix.get(&p1, &p2) {
                        CMPDISPATCH.resp_tx.send(ord)?;
                        continue;
                    }
                    self.cmp = Some([p1, p2]);
                    break;
                }
                Event::Finished => {
                    while let Event::Compare([p1, p2]) = CMPDISPATCH.req_rx.recv().unwrap() {
                        CMPDISPATCH
                            .resp_tx
                            .send(self.matrix.get(&p1, &p2).unwrap())?;
                    }
                    self.cmp = None;
                    self.current_screen = CurrentScreen::Finished;
                    break;
                }
            }
        }
        Ok(())
    }

    fn resp_event(&mut self, ord: CompareResult) -> Result<()> {
        let [p1, p2] = self.cmp.take().unwrap();
        self.matrix.insert(p1, p2, ord);
        if ord == CompareResult::Same {
            // p2 is the old key, if they are the same, p1 will be discarded in BTree.
            // Extending ensures data is not lost.
            // [BTreeSet src](https://github.com/rust-lang/rust/blob/6f7229c4da0471f1470bb1f86071848cba3a23d9/library/alloc/src/collections/btree/search.rs#L214-L230)
            p2.extend(p1);
        }
        CMPDISPATCH.resp_tx.send(ord)?;
        Ok(())
    }
}

impl Render for Tagger {
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
            Some([ref p1, ref p2]) => {
                // p2 is the old key in BTree, randomly choose from it for comparison.
                Images::new(&[&p1[0], p2.random_one()])?.render(f, chunks[1])?;
            }
            None => {
                Title {
                    title: "The comparation finished.".to_string(),
                }
                .render(f, chunks[1])?;
            }
        }
        TaggerFooter {
            current_screen: self.current_screen,
        }
        .render(f, chunks[2])?;
        Ok(())
    }
}

impl Drop for Tagger {
    fn drop(&mut self) {
        for paths in self.images.iter_mut() {
            unsafe {
                paths.drop();
            }
        }
    }
}
