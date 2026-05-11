use crate::registry::Tool;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Bind {
    pub section: String,
    pub key: String,
    pub action: String,
}

pub fn parse(tool: &Tool) -> Vec<Bind> {
    let path = tool.expanded_path();
    let raw = std::fs::read_to_string(&path).unwrap_or_default();
    match tool.parser.as_str() {
        "hypr_bind" => parse_hypr(&raw),
        "ncmpcpp"   => parse_ncmpcpp(&raw),
        "imv_ini"   => parse_ini(&raw, "binds"),
        "aerc"      => parse_aerc(&raw),
        "kitty"     => parse_kitty(&raw),
        "mpv_input" => parse_mpv(&raw),
        "nvim_lua"  => parse_nvim(&raw),
        "yazi_toml" => parse_yazi(&raw),
        "static"    => parse_static(tool),
        _           => vec![],
    }
}

fn strip(s: &str) -> String { s.trim().to_string() }

fn parse_hypr(raw: &str) -> Vec<Bind> {
    let mut out = vec![];
    let mut section = String::from("binds");
    let mut desc: Option<String> = None;
    for line in raw.lines() {
        let t = line.trim();
        if t.starts_with("###") {
            let cleaned: String = t.trim_matches('#').trim().to_string();
            if !cleaned.is_empty() { section = cleaned.to_lowercase(); }
            desc = None;
            continue;
        }
        if t.starts_with('#') {
            desc = Some(t.trim_start_matches('#').trim().to_string());
            continue;
        }
        if t.is_empty() { desc = None; continue; }
        if !(t.starts_with("bind") && t.contains('=')) { continue; }
        let body = t.splitn(2, '=').nth(1).unwrap_or("").trim();
        let parts: Vec<&str> = body.splitn(4, ',').map(|s| s.trim()).collect();
        if parts.len() < 3 { continue; }
        let mods = parts[0].replace("$mainMod", "Super").replace(' ', "+");
        let mut key = parts[1].to_string();
        if key == "mouse:272" { key = "LMB".into(); }
        else if key == "mouse:273" { key = "RMB".into(); }
        let action = parts[2];
        let arg = parts.get(3).copied().unwrap_or("");
        let chord = if mods.is_empty() { key.clone() } else { format!("{mods}+{key}") };
        let label = desc.clone().unwrap_or_else(|| {
            match action {
                "workspace" => format!("workspace {arg}"),
                "movetoworkspace" | "movetoworkspacesilent" => format!("send→ws {arg}"),
                "togglespecialworkspace" => format!("scratch ({arg})"),
                "movefocus" => format!("focus {}", dir(arg)),
                "movewindow" if matches!(arg, "l"|"r"|"u"|"d") => format!("move win {}", dir(arg)),
                _ => if arg.is_empty() { action.into() } else { format!("{action} {arg}") },
            }
        });
        out.push(Bind { section: section.clone(), key: chord, action: label.replace("exec ", "") });
        desc = None;
    }
    out
}

fn dir(d: &str) -> &'static str {
    match d { "l" => "left", "r" => "right", "u" => "up", _ => "down" }
}

fn parse_ncmpcpp(raw: &str) -> Vec<Bind> {
    let mut out = vec![];
    let mut section = String::from("binds");
    let mut iter = raw.lines().peekable();
    while let Some(line) = iter.next() {
        let t = line.trim();
        if t.starts_with("#####") {
            section = t.trim_matches('#').trim().to_lowercase();
            continue;
        }
        if !t.starts_with("def_key") { continue; }
        let key = t.split('"').nth(1).unwrap_or("").to_string();
        let mut actions = vec![];
        while let Some(peek) = iter.peek() {
            let pt = peek.trim_start();
            if peek.starts_with(' ') || peek.starts_with('\t') {
                if !pt.is_empty() { actions.push(pt.to_string()); }
                iter.next();
            } else { break; }
        }
        if !actions.is_empty() {
            out.push(Bind { section: section.clone(), key, action: actions.join(", ") });
        }
    }
    out
}

fn parse_ini(raw: &str, want_section: &str) -> Vec<Bind> {
    let mut out = vec![];
    let mut in_sec = false;
    for line in raw.lines() {
        let t = line.trim();
        if t.starts_with('[') && t.ends_with(']') {
            in_sec = &t[1..t.len()-1] == want_section;
            continue;
        }
        if !in_sec || t.is_empty() || t.starts_with('#') { continue; }
        if let Some((k, v)) = t.split_once('=') {
            out.push(Bind {
                section: want_section.into(),
                key: strip(k),
                action: strip(v),
            });
        }
    }
    out
}

fn parse_aerc(raw: &str) -> Vec<Bind> {
    let mut out = vec![];
    let mut section = String::from("global");
    for line in raw.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') { continue; }
        if t.starts_with('[') && t.ends_with(']') {
            section = t[1..t.len()-1].to_string();
            continue;
        }
        if let Some(eq) = t.find('=') {
            let key = strip(&t[..eq]);
            let mut act = strip(&t[eq+1..]);
            act = act.replace("<Enter>", "");
            if let Some(s) = act.strip_prefix(':') { act = s.into(); }
            out.push(Bind { section: section.clone(), key, action: act });
        }
    }
    out
}

fn parse_kitty(raw: &str) -> Vec<Bind> {
    raw.lines()
        .filter_map(|l| {
            let t = l.trim();
            if !t.starts_with("map ") { return None; }
            let rest = &t[4..];
            let mut parts = rest.splitn(2, char::is_whitespace);
            let key = parts.next()?.to_string();
            let action = parts.next()?.trim().to_string();
            Some(Bind { section: "binds".into(), key, action })
        })
        .collect()
}

fn parse_mpv(raw: &str) -> Vec<Bind> {
    let mut out = vec![];
    for line in raw.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') { continue; }
        let (head, _) = match t.find('#') { Some(i) => t.split_at(i), None => (t, "") };
        let head = head.trim();
        let mut parts = head.splitn(2, char::is_whitespace);
        let key = match parts.next() { Some(k) if !k.is_empty() => k.to_string(), _ => continue };
        let action = parts.next().unwrap_or("").trim().to_string();
        if action.is_empty() { continue; }
        out.push(Bind { section: "binds".into(), key, action });
    }
    out
}

fn parse_nvim(raw: &str) -> Vec<Bind> {
    let mut out = vec![];
    for line in raw.lines() {
        let t = line.trim();
        if !t.starts_with("vim.keymap.set") { continue; }
        let open = match t.find('(') { Some(i) => i + 1, None => continue };
        let rest = &t[open..];
        let (mode, after_mode) = take_arg(rest);
        let (lhs, after_lhs) = take_arg(after_mode);
        let (rhs, _after_rhs) = take_arg(after_lhs);
        if lhs.is_empty() { continue; }
        let m = mode.trim_matches(|c: char| c == '"' || c == '{' || c == '}' || c == '\'' || c.is_whitespace())
            .replace('"', "").replace(' ', "");
        let m_short = m.replace(',', "/");
        let action = rhs.trim_matches('"').trim().to_string();
        let key = lhs.trim_matches('"').to_string();
        let section = if m_short.is_empty() { "binds".into() } else { m_short };
        out.push(Bind { section, key, action });
    }
    out
}

// Extract the first comma-separated argument from a Lua arg list, respecting
// quoted strings and nested {}/() brackets. Returns (arg, remainder).
fn take_arg(s: &str) -> (String, &str) {
    let bytes = s.as_bytes();
    let mut depth_brace = 0i32;
    let mut depth_paren = 0i32;
    let mut in_str: Option<u8> = None;
    let mut esc = false;
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if esc { esc = false; i += 1; continue; }
        match in_str {
            Some(q) => {
                if c == b'\\' { esc = true; }
                else if c == q { in_str = None; }
            }
            None => match c {
                b'"' | b'\'' => in_str = Some(c),
                b'{' => depth_brace += 1,
                b'}' => depth_brace -= 1,
                b'(' => depth_paren += 1,
                b')' => {
                    if depth_paren == 0 { return (s[..i].trim().to_string(), &s[i..]); }
                    depth_paren -= 1;
                }
                b',' if depth_brace == 0 && depth_paren == 0 => {
                    return (s[..i].trim().to_string(), &s[i+1..]);
                }
                _ => {}
            }
        }
        i += 1;
    }
    (s.trim().to_string(), "")
}

fn parse_yazi(raw: &str) -> Vec<Bind> {
    let parsed: toml::Value = match toml::from_str(raw) {
        Ok(v) => v,
        Err(_) => return vec![],
    };
    let mut out = vec![];
    // Collect arrays-of-tables under common sections: mgr.{,prepend_,append_}keymap
    let candidates = [
        ("mgr", "keymap"),
        ("mgr", "prepend_keymap"),
        ("mgr", "append_keymap"),
        ("manager", "keymap"),
        ("manager", "prepend_keymap"),
        ("manager", "append_keymap"),
        ("input", "keymap"),
        ("input", "prepend_keymap"),
    ];
    for (top, arr) in candidates {
        let Some(t) = parsed.get(top).and_then(|v| v.get(arr)).and_then(|v| v.as_array()) else { continue };
        for entry in t {
            let on = entry.get("on").and_then(|v| v.as_str())
                .or_else(|| entry.get("on").and_then(|v| v.as_array()).and_then(|a| a.first()).and_then(|v| v.as_str()))
                .unwrap_or("");
            if on.is_empty() { continue; }
            let desc = entry.get("desc").and_then(|v| v.as_str()).unwrap_or_else(|| {
                entry.get("run").and_then(|v| v.as_str()).unwrap_or("")
            });
            out.push(Bind { section: format!("{top}.{arr}"), key: on.into(), action: desc.into() });
        }
    }
    out
}

fn parse_static(tool: &Tool) -> Vec<Bind> {
    let section = tool.section.clone().unwrap_or_else(|| "binds".into());
    tool.static_binds
        .as_ref()
        .map(|rows| rows.iter().filter_map(|r| {
            if r.len() < 2 { return None; }
            Some(Bind { section: section.clone(), key: r[0].clone(), action: r[1].clone() })
        }).collect())
        .unwrap_or_default()
}

pub fn config_path(tool: &Tool) -> &Path { Box::leak(tool.expanded_path().into_boxed_path()) }
