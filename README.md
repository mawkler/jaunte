# Jaunt

The most efficient way to move between windows. For Hyprland.

![Screenshot of Jaunt in action](docs/switcher.png)

> *Cogito ergo jaunteo. I think, therefore I jaunte.*
>
> — Alfred Bester, *The Stars My Destination*

## Usage

Jaunt works similarly to classic `Alt + Tab` switchers, but it is scoped to only one program's windows.

1. To switch to a window, press `Super + <key>`, where `<key>` is configured per program by you, for example:

    - `Super + t`: Terminal
    - `Super + w`: Web browser
    - `Super + s`: Spotify
    - etc...

2. Keep holding down the `Super` to see all windows of `<key>`'s program (if it's not running it gets launched)
3. Keep tapping `<key>` to cycle the focused window
4. Or press `Super + <number>` to go straight to a window, using the numbers shown beside them (windows 1-9, and only when the program has more than one)
5. Release `Super` to go to the currently selected window

Clicking anywhere outside the switcher closes it, which is the way out if a keybind ever isn't.

## Installation

### NixOS

```nix
# flake.nix
{
  inputs.jaunt.url = "github:mawkler/jaunt";
}
```

```nix
# ...then in your NixOS configuration:
{
  imports = [ inputs.jaunt.nixosModules.default ];

  services.jaunt = {
    enable = true;
    # If you prefer configuring with Nix (doesn't hot-reload)
    # settings = {
    #   keys.apps = {
    #     t = "ghostty";
    #     w = { cmd = "brave"; app_id = "brave-browser"; };
    #   }
    #   # ...
    # };
  };
}
```

This runs the daemon as a user service, which needs the session in systemd, e.g. `programs.hyprland.withUWSM = true`. Without `settings` it reads `~/.config/jaunt/config.toml` (which supports hot-reloading). See [Configuration](#configuration) for mor info.

### Other Linux distros

<details>
<summary><b>Click to expand</b></summary>

1. Install Jaunt:

    ```sh
    git clone https://github.com/mawkler/jaunt
    cargo install --path jaunt
    ```

2. Add this to your `~/.config/hypr/hyprland.lua`:

```lua
hl.on("hyprland.start", function() hl.exec_cmd("jaunt daemon") end)
```

</details>

## Configuration

Configuration is in `~/.config/jaunt/config.toml`, and it hot-reloads Jaunt on save.

`jaunt default-config` prints [the default config](src/config/default-config.toml).

Example configuration:

<details>
<summary><b>Click to expand</b></summary>

```toml
[keys]
next = "Tab"             # only bound while the switcher is open; Super is implied
previous = "Shift + Tab"
cancel = "Escape"
close = "q"              # close the highlighted window; unset by default

[keys.apps] # Super + key
t = "ghostty"
s = "spotify"
w = { cmd = "brave", app_id = "brave-browser" } # when the window class isn't the command
"Shift + a" = "teams"                           # a key may carry modifiers, but then has no Shift to go back with

[switcher]
delay = 90                      # milliseconds Super is held before the switcher appears
width = "50%"                   # of the screen, or pixels
max_height = "40%"              # previews shrink to fit
icons = true                    # icons rather than initials
background_opacity = 0.92       # the panel
foreground_opacity = 1          # the sheet holding the windows
theme = "default"
numbers = { action = "select" } # or "switch": a number key goes to its window at once

[previews]
enabled = true
height = 150   # pixels

[names] # by window class; otherwise the desktop entry's name
brave-browser = "Brave"
```

</details>

### Themes

The built-in themes live in [`themes/`](themes), and includes `default`, `light`, `catppuccin`, `nord`, `one-dark` and `tokyo-night`.

To create your own, put it in `~/.config/jaunt/themes/<name>.toml`, and set `theme = "<name>"` (or a path) in your config.

A theme only needs the colors it changes. The rest come from its `base`, or from `default` if it doesn't have one.

```toml
base = "nord"          # optional: the theme to start from
background = "#15171c" # the panel
surface = "#20232b"    # the sheet and its tab
accent = "#6b8cff"     # the selected window
text = "#c4cad8"       # window titles and keys
bright = "#eef1f7"     # the heading and the selected window's title
muted = "#78819a"      # secondary text and hints
```

A theme can also go straight into your config, written the same way:

```toml
[switcher.theme]
base = "nord"
accent = "#ff8800"
```

## Development

To set up the development environment (requires [Nix](https://nixos.org/download/)), run

```sh
nix develop
```

The switcher's QML in `shell/` is built into the binary. `JAUNT_SHELL=$PWD/shell jaunt daemon` runs it from the checkout instead, reloading on every save.

## AI disclosure

This project started out as hand-chiseled. I have since pivoted to relying heavily on agentic LLMs.
