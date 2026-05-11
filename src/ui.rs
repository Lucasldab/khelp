use crate::config::Config;
use crate::parsers::Bind;
use crate::registry::Tool;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

#[derive(Clone, Copy, PartialEq)]
pub enum Pane { Tools, Binds }

pub struct App {
    pub tools: Vec<Tool>,
    pub binds_per_tool: Vec<Vec<Bind>>,
    pub tool_state: ListState,
    pub bind_state: ListState,
    pub focus: Pane,
    pub filter: String,
    pub filter_mode: bool,
    pub status: String,
    pub quit: bool,
    pub pending: Option<char>,
    pub config: Config,
}

impl App {
    pub fn new(tools: Vec<Tool>, binds: Vec<Vec<Bind>>, config: Config) -> Self {
        let mut s = ListState::default();
        s.select(Some(0));
        let mut b = ListState::default();
        b.select(Some(0));
        Self {
            tools, binds_per_tool: binds,
            tool_state: s, bind_state: b,
            focus: Pane::Tools, filter: String::new(),
            filter_mode: false, status: String::new(),
            quit: false, pending: None, config,
        }
    }

    pub fn current_tool(&self) -> Option<&Tool> {
        self.tool_state.selected().and_then(|i| self.tools.get(i))
    }

    pub fn filtered_binds(&self) -> Vec<&Bind> {
        let i = self.tool_state.selected().unwrap_or(0);
        let all = self.binds_per_tool.get(i).map(|v| &v[..]).unwrap_or(&[]);
        if self.filter.is_empty() { return all.iter().collect(); }
        let f = self.filter.to_lowercase();
        all.iter().filter(|b| {
            b.key.to_lowercase().contains(&f)
                || b.action.to_lowercase().contains(&f)
                || b.section.to_lowercase().contains(&f)
        }).collect()
    }
}

pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(f.area());

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(20), Constraint::Min(1)])
        .split(chunks[0]);

    draw_tools(f, app, cols[0]);
    draw_binds(f, app, cols[1]);
    draw_status(f, app, chunks[1]);
}

fn pane_block<'a>(cfg: &Config, title: &'a str, focused: bool) -> Block<'a> {
    let c = &cfg.colors;
    Block::default()
        .borders(Borders::ALL)
        .title(format!(" {title} "))
        .border_style(if focused {
            Style::default().fg(c.lilac()).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(c.mauve())
        })
        .title_style(Style::default().fg(c.pink()).add_modifier(Modifier::BOLD))
}

fn draw_tools(f: &mut Frame, app: &mut App, area: Rect) {
    let c = &app.config.colors;
    let items: Vec<ListItem> = app.tools.iter()
        .map(|t| ListItem::new(t.name.clone()))
        .collect();
    let list = List::new(items)
        .block(pane_block(&app.config, "tools", app.focus == Pane::Tools))
        .highlight_style(Style::default().fg(c.pink()).add_modifier(Modifier::BOLD))
        .highlight_symbol("▌");
    f.render_stateful_widget(list, area, &mut app.tool_state);
}

fn draw_binds(f: &mut Frame, app: &mut App, area: Rect) {
    let c = &app.config.colors;
    let binds = app.filtered_binds();
    let mut last_section = String::new();
    let mut items: Vec<ListItem> = vec![];
    for b in &binds {
        if b.section != last_section {
            last_section = b.section.clone();
            items.push(ListItem::new(Line::from(vec![
                Span::styled(format!(" {} ", last_section),
                    Style::default().fg(c.lilac()).add_modifier(Modifier::BOLD)),
            ])));
        }
        items.push(ListItem::new(Line::from(vec![
            Span::styled(format!("  {:<24}", b.key), Style::default().fg(c.pink())),
            Span::styled(b.action.clone(), Style::default().fg(c.fg())),
        ])));
    }
    let title = match app.current_tool() {
        Some(t) => format!("{} — {} binds", t.name, binds.len()),
        None => "binds".into(),
    };
    let list = List::new(items)
        .block(pane_block(&app.config, &title, app.focus == Pane::Binds))
        .highlight_style(Style::default().bg(c.highlight_bg()).add_modifier(Modifier::BOLD));
    f.render_stateful_widget(list, area, &mut app.bind_state);
}

fn draw_status(f: &mut Frame, app: &App, area: Rect) {
    let c = &app.config.colors;
    let text = if app.filter_mode {
        format!("/{}", app.filter)
    } else if !app.status.is_empty() {
        app.status.clone()
    } else {
        let kb = &app.config.keybinds;
        let g = |k: &str| kb.get(k).cloned().unwrap_or_default();
        format!("{}/{} pane · {}/{} move · {} search · {} key · {} act · {} config · {} quit",
            g("focus_tools"), g("focus_binds"), g("down"), g("up"),
            g("search"), g("yank_key"), g("yank_action"), g("open_config"), g("quit"))
    };
    let p = Paragraph::new(text).style(Style::default().fg(c.gray()));
    f.render_widget(p, area);
}
