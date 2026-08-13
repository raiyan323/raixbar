# RaixBar

> ⚡ A fast, minimal, Waybar-style status bar for Hyprland and Wayland.

**RaixBar** is a lightweight and configurable status bar written in **Rust + GTK4**, designed for Linux users who want a fast, clean, and minimal desktop bar.

It uses **Wayland layer-shell** through `gtk4-layer-shell`, making it especially suitable for Wayland compositors such as **Hyprland**.

RaixBar is designed around a simple layout:

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│   Raix   [1] [2] [3] [4] [5]                  Wed 13 Aug • 13:42   CPU RAM   │
└──────────────────────────────────────────────────────────────────────────────┘
   LEFT                         CENTER                              RIGHT
```

---

## 🖼️ Screenshot

<p align="center">
  <img src="assets/screenshot.png" alt="RaixBar" width="1200">
</p>

---

# ✨ Features

- ⚡ Fast and lightweight Rust implementation
- 🎨 GTK4 interface
- 🐧 Native Wayland support
- 🎯 `wlr-layer-shell` integration
- 🖥️ Designed for Hyprland
- 📏 Full-width bar
- ⬅️ Dedicated left section
- 🎯 Dedicated centered section
- ➡️ Dedicated right section
- 🕐 Centered clock
- 🧩 Configurable modules
- 🛠️ TOML-based configuration
- 🖱️ Mouse click actions
- 🖱️ Middle-click actions
- 🖱️ Right-click actions
- 🖱️ Scroll actions
- 🖥️ Hyprland workspace support
- 📊 CPU usage
- 🧠 RAM usage
- 🌐 Network status
- 🔊 Audio volume
- 🔋 Battery support
- 🎨 Custom CSS
- 🔤 Nerd Font support
- 🚀 Low resource usage
- 📦 Arch Linux / PKGBUILD support

---

# 🧱 Layout

RaixBar separates the bar into three independent sections.

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│  LEFT                         CENTER                         RIGHT           │
│                                                                             │
│   Raix   1 2 3 4 5       Wed 13 Aug • 13:42       CPU  RAM            │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### LEFT

Used for:

- Logo
- Workspaces
- Custom left-side modules

### CENTER

Used for:

- Clock
- Custom centered modules

The center section remains centered independently of the contents of the left and right sections.

### RIGHT

Used for:

- CPU
- RAM
- Network
- Volume
- Battery
- Other system modules

This structure is inspired by the layout behavior of **Waybar**.

---

# 🧰 Requirements

- Linux
- Wayland
- Wayland compositor
- `wlr-layer-shell` support
- Rust
- Cargo
- GTK4
- `gtk4-layer-shell`

For the default modules, the following optional applications may also be useful:

- `wpctl`
- `pactl`
- `pavucontrol`
- `networkmanager`
- `kitty`
- `btop`

---

# 📦 Arch Linux

RaixBar includes a `PKGBUILD`, so Arch Linux users can build and install it directly.

## Install dependencies

```bash
sudo pacman -S --needed \
    base-devel \
    rust \
    cargo \
    gtk4 \
    gtk4-layer-shell \
    wayland \
    wayland-protocols \
    pkgconf
```

Depending on your system, you may also need:

```bash
sudo pacman -S --needed \
    libxkbcommon
```

---

# 🚀 Build From Source

Clone the repository:

```bash
git clone https://github.com/raiyan323/raixbar.git
```

Enter the project directory:

```bash
cd raixbar
```

Build:

```bash
cargo build --release
```

The compiled binary will be available at:

```text
target/release/raixbar
```

Run it:

```bash
./target/release/raixbar
```

---

# 📦 Arch Package

If the repository contains a `PKGBUILD`, you can build and install the package with:

```bash
makepkg -si
```

After installation:

```bash
raixbar
```

---

# 🛠️ Development Build

Clone the repository:

```bash
git clone https://github.com/raiyan323/raixbar.git
cd raixbar
```

Build:

```bash
cargo build
```

Run:

```bash
cargo run
```

For an optimized build:

```bash
cargo build --release
```

---

# ⚙️ Configuration

RaixBar automatically creates its configuration file on first launch:

```text
~/.config/raixbar/config.toml
```

Edit it with your preferred editor:

```bash
nano ~/.config/raixbar/config.toml
```

or:

```bash
vim ~/.config/raixbar/config.toml
```

---

# 📝 Example Configuration

```toml
[bar]

width = 0
height = 44

position = "top"
alignment = "center"

margin_top = 0
margin_bottom = 0
margin_left = 0
margin_right = 0

exclusive_zone = 44

layer = "top"
keyboard_mode = "none"


[style]

background = "rgba(17, 17, 27, 0.94)"
background_hover = "rgba(255, 255, 255, 0.07)"

opacity = 0.95

text_color = "#cdd6f4"
muted_color = "#6c7086"
accent_color = "#89b4fa"

border_color = "rgba(255,255,255,0.08)"
border_width = 0
border_radius = 0

module_background = "transparent"
module_hover_background = "rgba(255,255,255,0.065)"
module_radius = 9

workspace_background = "rgba(255,255,255,0.035)"
workspace_active_background = "rgba(137,180,250,0.20)"
workspace_hover_background = "rgba(255,255,255,0.08)"
workspace_urgent_background = "rgba(243,139,168,0.22)"

workspace_active_color = "#ffffff"
workspace_inactive_color = "#6c7086"
workspace_urgent_color = "#f38ba8"


[font]

family = "JetBrainsMono Nerd Font"

size = 12
weight = 650

logo_size = 16
workspace_size = 12


[spacing]

module = 3

horizontal = 10
vertical = 3

bar_padding = 5


[layout]

left = [
    "logo",
    "workspaces"
]

center = [
    "clock"
]

right = [
    "cpu",
    "ram",
    "network",
    "volume"
]


[logo]

enabled = true

icon = ""
text = "Raix"

icon_color = "#89b4fa"
text_color = "#cdd6f4"

icon_size = 16


[workspaces]

enabled = true

show_empty = true
show_special = false

numbers = [
    1,
    2,
    3,
    4,
    5
]

format = "{id}"

active_color = "#ffffff"
inactive_color = "#6c7086"
urgent_color = "#f38ba8"

active_background = "rgba(137,180,250,0.20)"
inactive_background = "transparent"
hover_background = "rgba(255,255,255,0.08)"
urgent_background = "rgba(243,139,168,0.22)"

padding = 9
radius = 8

scroll_switch = true

left_click = "hyprctl dispatch workspace {id}"

right_click = ""
middle_click = ""

interval = 500


[cpu]

enabled = true

format = "{icon}  {value}%"
icon = "󰍛"

color = "#cba6f7"
background = "transparent"

font_size = 12
font_weight = 650

padding = 10
radius = 9

interval = 1000

tooltip = "CPU Usage"

left_click = "kitty -e btop"
right_click = ""
middle_click = ""

scroll_up = ""
scroll_down = ""


[ram]

enabled = true

format = "{icon}  {value}%"
icon = "󰘚"

color = "#a6e3a1"
background = "transparent"

font_size = 12
font_weight = 650

padding = 10
radius = 9

interval = 1000

tooltip = "Memory Usage"

left_click = "kitty -e btop"
right_click = ""
middle_click = ""

scroll_up = ""
scroll_down = ""


[network]

enabled = true

format = "{icon}  {value}"
icon = "󰖩"

color = "#89dceb"
background = "transparent"

font_size = 12
font_weight = 650

padding = 10
radius = 9

interval = 1000

tooltip = "Network"

left_click = "nm-connection-editor"
right_click = "kitty -e nmtui"
middle_click = ""

scroll_up = ""
scroll_down = ""


[volume]

enabled = true

format = "{icon}  {value}%"
icon = "󰕾"

color = "#f5c2e7"
background = "transparent"

font_size = 12
font_weight = 650

padding = 10
radius = 9

interval = 500

tooltip = "Audio Volume"

left_click = "pavucontrol"

right_click = "wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle"

middle_click = ""

scroll_up = "wpctl set-volume -l 1.5 @DEFAULT_AUDIO_SINK@ 5%+"
scroll_down = "wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%-"


[battery]

enabled = false

format = "{icon}  {value}%"
icon = "󰁹"

color = "#f9e2af"

interval = 5000

tooltip = "Battery"


[clock]

enabled = true

format = "%a  %d %b  •  %H:%M"

color = "#89b4fa"

font_size = 12
font_weight = 800

padding = 14
radius = 9

interval = 1000


[css]

custom = """
.raix-bar {
    box-shadow: 0 8px 30px rgba(0,0,0,0.28);
}

.workspace-container {
    margin-left: 3px;
    margin-right: 3px;
}

.module:hover {
    box-shadow: 0 0 12px rgba(137,180,250,0.06);
}

.workspace.active {
    font-weight: 800;
}

.workspace.urgent {
    font-weight: 800;
}
"""
```

> Restart RaixBar after changing the configuration.

---

# 🖥️ Hyprland

Add RaixBar to your Hyprland configuration:

```ini
exec-once = raixbar
```

You can also run it manually:

```bash
raixbar
```

---

# 🔄 Restart RaixBar

After changing the configuration:

```bash
pkill raixbar
raixbar &
```

---

# 📊 Available Modules

| Module | Description |
|---|---|
| `logo` | Custom logo and text |
| `workspaces` | Hyprland workspaces |
| `clock` | Date and time |
| `cpu` | CPU utilization |
| `ram` | Memory utilization |
| `network` | Network interface |
| `volume` | Audio volume |
| `battery` | Battery percentage |
| `separator` | Visual separator |

Modules can be arranged through:

```toml
[layout]

left = [
    "logo",
    "workspaces"
]

center = [
    "clock"
]

right = [
    "cpu",
    "ram",
    "network",
    "volume"
]
```

---

# 🖱️ Module Actions

Most modules support:

```toml
left_click = ""
right_click = ""
middle_click = ""

scroll_up = ""
scroll_down = ""
```

For example:

```toml
left_click = "pavucontrol"
right_click = "wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle"

scroll_up = "wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%+"
scroll_down = "wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%-"
```

---

# 🧩 Hyprland Workspaces

Workspace buttons can execute Hyprland commands directly:

```toml
left_click = "hyprctl dispatch workspace {id}"
```

The `{id}` placeholder is automatically replaced with the workspace number.

---

# 🎨 Custom CSS

RaixBar supports additional GTK CSS:

```toml
[css]

custom = """
.module:hover {
    background: rgba(255,255,255,0.10);
}

.workspace.active {
    font-weight: 800;
}
"""
```

---

# 🏗️ Project Structure

```text
raixbar/
├── Cargo.toml
├── Cargo.lock
├── PKGBUILD
├── README.md
├── assets/
│   └── screenshot.png
├── src/
│   └── main.rs
└── LICENSE
```

---

# 🔧 Technology

RaixBar is built with:

- **Rust**
- **GTK4**
- **gtk4-layer-shell**
- **Wayland**
- **Hyprland**
- **TOML**

The project intentionally keeps the implementation lightweight and avoids heavy status-bar frameworks.

---

# 📜 License

RaixBar is open-source software licensed under the **MIT License**.

See [`LICENSE`](LICENSE) for the full license text.

---

# ⚡ Philosophy

RaixBar is built around a simple idea:

> **Your desktop information should be visible without getting in your way.**

No unnecessary animations.

No bloated framework.

No complicated configuration.

Just:

```text
See → Understand → Act
```

Fast.

Minimal.

Wayland-native.

---

# ⭐ Support

If you like RaixBar, consider giving the project a ⭐ on GitHub.

Issues, improvements, feature requests, and pull requests are welcome.

---

# ⚡ RaixBar

**A minimal status bar for a minimal desktop.**
