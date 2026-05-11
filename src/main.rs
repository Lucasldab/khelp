mod config;
mod parsers;
mod registry;
mod ui;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, process::Command, time::Duration};
use ui::{App, Pane};

fn xdg_config_dir() -> String {
    let xdg = std::env::var("XDG_CONFIG_HOME")
        .unwrap_or_else(|_| format!("{}/.config", std::env::var("HOME").unwrap_or_default()));
    format!("{xdg}/khelp")
}

fn registry_path() -> String {
    if let Ok(p) = std::env::var("KHELP_REGISTRY") { return p; }
    let cfg = format!("{}/registry.toml", xdg_config_dir());
    if std::path::Path::new(&cfg).exists() { return cfg; }
    "registry.toml".into()
}

fn config_path() -> String {
    if let Ok(p) = std::env::var("KHELP_CONFIG") { return p; }
    format!("{}/config.toml", xdg_config_dir())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let reg = registry::load(&registry_path())?;
    let cfg = config::load(&config_path());
    let binds: Vec<_> = reg.tools.iter().map(parsers::parse).collect();
    let mut app = App::new(reg.tools, binds, cfg);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let mut term = Terminal::new(CrosstermBackend::new(stdout))?;

    let res = run(&mut term, &mut app);

    disable_raw_mode()?;
    execute!(term.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    term.show_cursor()?;
    res
}

fn run<B: ratatui::backend::Backend>(term: &mut Terminal<B>, app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    while !app.quit {
        term.draw(|f| ui::draw(f, app))?;
        if !event::poll(Duration::from_millis(200))? { continue; }
        if let Event::Key(k) = event::read()? {
            if k.kind != event::KeyEventKind::Press { continue; }
            if app.filter_mode { handle_filter(app, k.code); continue; }
            handle_normal(app, k);
        }
    }
    Ok(())
}

fn handle_filter(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Esc => { app.filter.clear(); app.filter_mode = false; }
        KeyCode::Enter => { app.filter_mode = false; }
        KeyCode::Backspace => { app.filter.pop(); }
        KeyCode::Char(c) => { app.filter.push(c); }
        _ => {}
    }
    app.bind_state.select(Some(0));
}

fn key_event_str(k: &KeyEvent) -> String {
    let base = match k.code {
        KeyCode::Char(c) => c.to_string(),
        KeyCode::Esc => "Esc".into(),
        KeyCode::Enter => "Enter".into(),
        KeyCode::Tab => "Tab".into(),
        KeyCode::BackTab => "S-Tab".into(),
        KeyCode::Left => "Left".into(),
        KeyCode::Right => "Right".into(),
        KeyCode::Up => "Up".into(),
        KeyCode::Down => "Down".into(),
        KeyCode::Backspace => "BS".into(),
        KeyCode::F(n) => format!("F{n}"),
        _ => return String::new(),
    };
    if k.modifiers.contains(KeyModifiers::CONTROL) && matches!(k.code, KeyCode::Char(_)) {
        return format!("ctrl-{}", base.to_lowercase());
    }
    base
}

fn handle_normal(app: &mut App, k: KeyEvent) {
    let pressed = key_event_str(&k);
    if pressed.is_empty() { return; }

    if let Some(prev) = app.pending.take() {
        let chord = format!("{prev}{pressed}");
        if let Some(action) = match_action(app, &chord) {
            dispatch(app, &action);
            return;
        }
    }

    if let KeyCode::Char(c) = k.code {
        let chord_starts: Vec<String> = app.config.keybinds.values()
            .filter(|v| v.len() > 1 && v.starts_with(c) && !v.contains('-'))
            .cloned().collect();
        if !chord_starts.is_empty() && match_action(app, &pressed).is_none() {
            app.pending = Some(c);
            return;
        }
    }

    if let Some(action) = match_action(app, &pressed) {
        dispatch(app, &action);
    } else if k.modifiers.contains(KeyModifiers::CONTROL) && matches!(k.code, KeyCode::Char('c')) {
        app.quit = true;
    }
}

fn match_action(app: &App, key: &str) -> Option<String> {
    app.config.keybinds.iter()
        .find(|(_, v)| v.as_str() == key)
        .map(|(k, _)| k.clone())
}

fn dispatch(app: &mut App, action: &str) {
    match action {
        "quit" | "quit_alt" => app.quit = true,
        "focus_tools" => app.focus = Pane::Tools,
        "focus_binds" => app.focus = Pane::Binds,
        "focus_toggle" => app.focus = if app.focus == Pane::Tools { Pane::Binds } else { Pane::Tools },
        "down" => move_sel(app, 1),
        "up"   => move_sel(app, -1),
        "first" => select_first(app),
        "last"  => select_last(app),
        "search" => { app.filter_mode = true; app.filter.clear(); }
        "yank_key"    => yank_key(app),
        "yank_action" => yank_action(app),
        "open_config" => open_config(app),
        "reload"      => reload(app),
        _ => {}
    }
}

fn move_sel(app: &mut App, delta: i32) {
    match app.focus {
        Pane::Tools => {
            let len = app.tools.len();
            let cur = app.tool_state.selected().unwrap_or(0) as i32;
            let new = ((cur + delta).rem_euclid(len as i32)) as usize;
            app.tool_state.select(Some(new));
            app.bind_state.select(Some(0));
        }
        Pane::Binds => {
            let len = app.filtered_binds().len();
            if len == 0 { return; }
            let cur = app.bind_state.selected().unwrap_or(0) as i32;
            let new = ((cur + delta).rem_euclid(len as i32)) as usize;
            app.bind_state.select(Some(new));
        }
    }
}

fn select_first(app: &mut App) {
    match app.focus {
        Pane::Tools => app.tool_state.select(Some(0)),
        Pane::Binds => app.bind_state.select(Some(0)),
    }
}

fn select_last(app: &mut App) {
    match app.focus {
        Pane::Tools => app.tool_state.select(Some(app.tools.len().saturating_sub(1))),
        Pane::Binds => {
            let n = app.filtered_binds().len().saturating_sub(1);
            app.bind_state.select(Some(n));
        }
    }
}

fn current_bind(app: &App) -> Option<parsers::Bind> {
    let i = app.bind_state.selected()?;
    app.filtered_binds().into_iter().nth(i).cloned()
}

fn yank_key(app: &mut App) {
    if let Some(b) = current_bind(app) {
        copy(&b.key);
        app.status = format!("yanked key: {}", b.key);
    }
}

fn yank_action(app: &mut App) {
    if let Some(b) = current_bind(app) {
        copy(&b.action);
        app.status = format!("yanked action: {}", b.action);
    }
}

fn copy(s: &str) {
    if let Ok(mut cb) = arboard::Clipboard::new() {
        let _ = cb.set_text(s.to_string());
    }
}

fn open_config(app: &mut App) {
    let Some(t) = app.current_tool() else { return };
    let path = t.expanded_path();
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nvim".into());
    app.status = format!("opening {} in {editor}", path.display());
    let _ = Command::new(editor).arg(path).status();
}

fn reload(app: &mut App) {
    app.binds_per_tool = app.tools.iter().map(parsers::parse).collect();
    app.status = "reloaded".into();
    app.bind_state.select(Some(0));
}
