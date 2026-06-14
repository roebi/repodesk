use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    style::Print,
    terminal::{self, ClearType},
};
use repodesk_commands::{Command, EditorCommand, GitCommand, LayoutCommand, PaletteCommand};
use repodesk_config::Config;
use repodesk_core::{reduce, AppState};
use repodesk_fs::{read_file, write_file};
use repodesk_git::{GitBackend, GitCli};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::palette::render_palette_overlay;
use crate::snapshot::render_to_lines_with_layout_and_root;

pub fn run(repo_path: Option<PathBuf>) -> io::Result<()> {
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;
    let result = event_loop(&mut stdout, repo_path);
    let _ = execute!(stdout, terminal::LeaveAlternateScreen, cursor::Show);
    terminal::disable_raw_mode()?;
    result
}

struct Runtime {
    app: AppState,
    config: Config,
    repo_root: Option<PathBuf>,
    active_path: Option<PathBuf>,
    open_input: Option<String>,
}

impl Runtime {
    fn new(repo_root: Option<PathBuf>) -> Self {
        let config = repo_root.as_deref().map(Config::load_or_default).unwrap_or_default();
        Self { app: AppState::default(), config, repo_root, active_path: None, open_input: None }
    }

    fn refresh_git(&mut self) {
        if let Some(root) = &self.repo_root {
            let git = GitCli::new(root);
            if let Ok(branch) = git.current_branch() {
                self.app = reduce(self.app.clone(), Command::Git(GitCommand::RefreshBranch(branch)));
            }
            if let Ok(entries) = git.status() {
                self.app = reduce(self.app.clone(), Command::Git(GitCommand::RefreshStatus(entries)));
            }
            if let Ok(diff) = git.diff() {
                self.app = reduce(self.app.clone(), Command::Git(GitCommand::RefreshDiff(diff)));
            }
        }
    }

    fn open_file(&mut self, path: &Path) {
        match read_file(path) {
            Ok(buffer) => {
                self.app = reduce(self.app.clone(), Command::OpenFile(path.display().to_string()));
                if let Some(ref mut editor) = self.app.active_editor { *editor = buffer; }
                self.active_path = Some(path.to_path_buf());
                self.app = reduce(self.app.clone(), Command::SetStatus(format!("Opened {}", path.display())));
            }
            Err(e) => {
                self.app = reduce(self.app.clone(), Command::SetStatus(format!("Error: {}", e)));
            }
        }
    }

    fn save_active(&mut self) {
        match &self.active_path {
            None => { self.app = reduce(self.app.clone(), Command::SetStatus("No file open to save".to_string())); }
            Some(path) => {
                let path = path.clone();
                if let Some(editor) = &self.app.active_editor {
                    match write_file(&path, editor) {
                        Ok(()) => { self.app = reduce(self.app.clone(), Command::SetStatus(format!("Saved {}", path.display()))); }
                        Err(e) => { self.app = reduce(self.app.clone(), Command::SetStatus(format!("Save error: {}", e))); }
                    }
                }
            }
        }
    }
}

fn event_loop(stdout: &mut impl Write, repo_root: Option<PathBuf>) -> io::Result<()> {
    let mut rt = Runtime::new(repo_root);
    rt.refresh_git();

    loop {
        let (w, h) = terminal::size()?;
        draw(stdout, &rt.app, w, h, rt.repo_root.as_deref())?;

        if let Event::Key(key) = event::read()? {

            // Palette open mode
            if rt.app.palette_open {
                match (key.modifiers, key.code) {
                    (_, KeyCode::Esc) => { rt.app = reduce(rt.app.clone(), Command::Palette(PaletteCommand::Close)); }
                    (_, KeyCode::Up)   => { rt.app = reduce(rt.app.clone(), Command::Palette(PaletteCommand::MoveSelection(-1))); }
                    (_, KeyCode::Down) => { rt.app = reduce(rt.app.clone(), Command::Palette(PaletteCommand::MoveSelection(1))); }
                    (_, KeyCode::Enter) => {
                        let idx = rt.app.palette_selected;
                        let cmd = rt.app.palette_results.get(idx).map(|r| r.command.clone());
                        rt.app = reduce(rt.app.clone(), Command::Palette(PaletteCommand::SelectResult(idx)));
                        if let Some(c) = cmd {
                            rt.app = reduce(rt.app.clone(), c);
                        }
                    }
                    (_, KeyCode::Backspace) => {
                        let mut q = rt.app.palette_query.clone();
                        q.pop();
                        rt.app = reduce(rt.app.clone(), Command::Palette(PaletteCommand::SetQuery(q)));
                    }
                    (_, KeyCode::Char(ch)) => {
                        let mut q = rt.app.palette_query.clone();
                        q.push(ch);
                        rt.app = reduce(rt.app.clone(), Command::Palette(PaletteCommand::SetQuery(q)));
                    }
                    _ => {}
                }
                continue;
            }

            // :open input mode
            if let Some(ref mut input) = rt.open_input {
                match key.code {
                    KeyCode::Enter => {
                        let path = PathBuf::from(input.clone());
                        rt.open_input = None;
                        rt.open_file(&path);
                        rt.refresh_git();
                    }
                    KeyCode::Esc => { rt.open_input = None; rt.app = reduce(rt.app.clone(), Command::SetStatus(String::new())); }
                    KeyCode::Backspace => {
                        input.pop();
                        let p = format!(":open {}", input);
                        rt.app = reduce(rt.app.clone(), Command::SetStatus(p));
                    }
                    KeyCode::Char(ch) => {
                        input.push(ch);
                        let p = format!(":open {}", input);
                        rt.app = reduce(rt.app.clone(), Command::SetStatus(p));
                    }
                    _ => {}
                }
                continue;
            }

            // Normal mode
            match (key.modifiers, key.code) {
                (KeyModifiers::CONTROL, KeyCode::Char('q')) => break,
                (KeyModifiers::CONTROL, KeyCode::Char('s')) => rt.save_active(),
                (KeyModifiers::CONTROL, KeyCode::Char('o')) => {
                    rt.open_input = Some(String::new());
                    rt.app = reduce(rt.app.clone(), Command::SetStatus(":open ".to_string()));
                }
                (KeyModifiers::CONTROL, KeyCode::Char('g')) => rt.refresh_git(),
                (KeyModifiers::CONTROL, KeyCode::Char('p')) => {
                    rt.app = reduce(rt.app.clone(), Command::Palette(PaletteCommand::Open));
                }
                (KeyModifiers::CONTROL, KeyCode::Char('d')) => {
                    rt.app = reduce(rt.app.clone(), Command::Layout(LayoutCommand::ToggleDiff));
                }
                (KeyModifiers::CONTROL, KeyCode::Char('e')) => {
                    rt.app = reduce(rt.app.clone(), Command::Layout(LayoutCommand::ToggleEditorFull));
                }
                (_, KeyCode::Left)      => { rt.app = reduce(rt.app.clone(), Command::Editor(EditorCommand::MoveLeft)); }
                (_, KeyCode::Right)     => { rt.app = reduce(rt.app.clone(), Command::Editor(EditorCommand::MoveRight)); }
                (_, KeyCode::Up)        => { rt.app = reduce(rt.app.clone(), Command::Editor(EditorCommand::MoveUp)); }
                (_, KeyCode::Down)      => { rt.app = reduce(rt.app.clone(), Command::Editor(EditorCommand::MoveDown)); }
                (_, KeyCode::Char(ch))  => { rt.app = reduce(rt.app.clone(), Command::Editor(EditorCommand::InsertChar(ch))); }
                (_, KeyCode::Backspace) => { rt.app = reduce(rt.app.clone(), Command::Editor(EditorCommand::DeleteChar)); }
                (_, KeyCode::Enter)     => { rt.app = reduce(rt.app.clone(), Command::Editor(EditorCommand::SplitLine)); }
                _ => { rt.app = reduce(rt.app.clone(), Command::SetStatus(String::new())); }
            }
        }
    }
    Ok(())
}

fn draw(stdout: &mut impl Write, state: &AppState, w: u16, h: u16, repo_root: Option<&Path>) -> io::Result<()> {
    let mut lines = render_to_lines_with_layout_and_root(state, w, h, repo_root);
    render_palette_overlay(&mut lines, state, w as usize);
    execute!(stdout, cursor::MoveTo(0, 0), terminal::Clear(ClearType::All))?;
    for (row, line) in lines.iter().enumerate() {
        execute!(stdout, cursor::MoveTo(0, row as u16), Print(line))?;
    }
    stdout.flush()
}
