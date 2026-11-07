# khelp

A keybind cheatsheet for the terminal. khelp reads your tools' config files and
lists the keybinds they define, so the cheatsheet stays in sync with your setup
instead of going out of date like a hand-written one.

Two panes: your tools on the left, and the selected tool's binds on the right,
grouped by section. Search, copy a key or action to the clipboard, or jump
straight to the config file that defines it.

## Build

```sh
cargo build --release
install -Dm755 target/release/khelp ~/.local/bin/khelp
```

It works well as a floating window on a global shortcut. In a Hyprland
`keybinds.conf`, for example:

```
bind = $mainMod, slash, exec, kitty --class kitty-mini -e ~/.local/bin/khelp
```

## Registry

`~/.config/khelp/registry.toml` lists the tools to show, one `[[tool]]` block
each. khelp reads that file, or `$KHELP_REGISTRY` if set, or `registry.toml` in
the current directory as a fallback. The [example registry](registry.toml) in
this repo covers 16 tools.

```toml
[[tool]]
name   = "kitty"
path   = "~/.config/kitty/kitty.conf"
parser = "kitty"

[[tool]]
name   = "zathura"
path   = "~/.config/zathura/zathurarc"
parser = "static"
section = "core"
static_binds = [
  ["j / k", "scroll down / up"],
  ["/",     "search"],
]
```

`path` is the config file the parser reads, and the file `gd` opens. A tool
whose file is missing or unreadable shows no binds.

| Parser | Reads |
|--------|-------|
| `hypr_bind` | Hyprland `keybinds.conf`: `bind`, `binde`, `bindl`, `bindle`, `bindm` |
| `kitty` | `map <key> <action>` lines |
| `nvim_lua` | `vim.keymap.set(mode, lhs, rhs)` calls in Lua |
| `yazi_toml` | yazi `keymap.toml` (`on`, `run`, `desc`) |
| `mpv_input` | mpv `input.conf` |
| `aerc` | aerc `binds.conf` sections |
| `ncmpcpp` | ncmpcpp `bindings` (`def_key`) |
| `imv_ini` | the `[binds]` section of an imv config |
| `static` | the `static_binds` list in the registry entry itself |

`static` covers tools whose config is not parseable, or that use built-in keys,
including a Hyprland setup configured in Lua.

## Keys

| Key | Action |
|-----|--------|
| `j` / `k` | down / up |
| `h` / `l`, `Tab` | focus the tools / binds pane |
| `gg` / `G` | first / last |
| `/` | search keys, actions and sections in the selected tool |
| `y` / `Y` | copy the selected key / action |
| `gd` | open the tool's config in `$EDITOR` (default `nvim`) |
| `r` | re-read every config |
| `q`, `Esc` | quit |

## Config

`~/.config/khelp/config.toml` (or `$KHELP_CONFIG`) can change the colours and
rebind any of the keys above. Every field is optional, and anything left out
keeps its default. See [config.toml](config.toml) for the full list and the
supported key forms (single keys, two-key chords like `gg`, special keys and
`ctrl-` combinations).

## License

MIT, see [LICENSE](LICENSE).
