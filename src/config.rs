use ratatui::style::Color;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    pub colors: Colors,
    pub keybinds: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Colors {
    pub pink:  String,
    pub lilac: String,
    pub mauve: String,
    pub fg:    String,
    pub gray:  String,
    pub highlight_bg: String,
}

impl Default for Colors {
    fn default() -> Self {
        Self {
            pink:  "#FF6EC7".into(),
            lilac: "#9A6DD7".into(),
            mauve: "#5A2A82".into(),
            fg:    "#E5E5E5".into(),
            gray:  "#6A6A6A".into(),
            highlight_bg: "#3C1932".into(),
        }
    }
}

impl Colors {
    pub fn pink(&self)  -> Color { hex(&self.pink)  }
    pub fn lilac(&self) -> Color { hex(&self.lilac) }
    pub fn mauve(&self) -> Color { hex(&self.mauve) }
    pub fn fg(&self)    -> Color { hex(&self.fg)    }
    pub fn gray(&self)  -> Color { hex(&self.gray)  }
    pub fn highlight_bg(&self) -> Color { hex(&self.highlight_bg) }
}

fn hex(s: &str) -> Color {
    let s = s.trim_start_matches('#');
    if s.len() != 6 { return Color::White; }
    let r = u8::from_str_radix(&s[0..2], 16).unwrap_or(255);
    let g = u8::from_str_radix(&s[2..4], 16).unwrap_or(255);
    let b = u8::from_str_radix(&s[4..6], 16).unwrap_or(255);
    Color::Rgb(r, g, b)
}

pub fn defaults() -> HashMap<String, String> {
    let pairs: &[(&str, &str)] = &[
        ("quit",          "q"),
        ("quit_alt",      "Esc"),
        ("focus_tools",   "h"),
        ("focus_binds",   "l"),
        ("focus_toggle",  "Tab"),
        ("down",          "j"),
        ("up",            "k"),
        ("first",         "gg"),
        ("last",          "G"),
        ("search",        "/"),
        ("yank_key",      "y"),
        ("yank_action",   "Y"),
        ("open_config",   "gd"),
        ("reload",        "r"),
    ];
    pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
}

pub fn load(path: &str) -> Config {
    let mut cfg: Config = std::fs::read_to_string(path)
        .ok()
        .and_then(|s| toml::from_str(&s).ok())
        .unwrap_or_default();
    let defaults = defaults();
    for (k, v) in defaults {
        cfg.keybinds.entry(k).or_insert(v);
    }
    cfg
}

pub fn lookup<'a>(cfg: &'a Config, action: &str) -> Option<&'a str> {
    cfg.keybinds.get(action).map(|s| s.as_str())
}
