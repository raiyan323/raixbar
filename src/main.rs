use gtk4::gdk;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{
    Align,
    Application,
    ApplicationWindow,
    Box as GtkBox,
    CenterBox,
    CssProvider,
    EventControllerScroll,
    EventControllerScrollFlags,
    GestureClick,
    Label,
    Orientation,
};

use gtk4_layer_shell::{
    Edge,
    KeyboardMode,
    Layer,
    LayerShell,
};

use serde::{Deserialize, Serialize};

use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::rc::Rc;
use std::time::Duration;


// ============================================================
// CONFIG
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct Config {
    bar: BarConfig,
    style: StyleConfig,
    font: FontConfig,
    spacing: SpacingConfig,
    layout: LayoutConfig,

    logo: LogoConfig,
    logo_text: LogoTextConfig,

    workspaces: WorkspaceConfig,

    cpu: ModuleConfig,
    ram: ModuleConfig,
    network: ModuleConfig,
    volume: ModuleConfig,
    battery: ModuleConfig,

    clock: ClockConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bar: BarConfig::default(),
            style: StyleConfig::default(),
            font: FontConfig::default(),
            spacing: SpacingConfig::default(),
            layout: LayoutConfig::default(),

            logo: LogoConfig::default(),
            logo_text: LogoTextConfig::default(),

            workspaces: WorkspaceConfig::default(),

            cpu: ModuleConfig::cpu(),
            ram: ModuleConfig::ram(),
            network: ModuleConfig::network(),
            volume: ModuleConfig::volume(),
            battery: ModuleConfig::battery(),

            clock: ClockConfig::default(),
        }
    }
}


// ============================================================
// BAR
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct BarConfig {
    width: i32,
    height: i32,

    position: String,

    margin_top: i32,
    margin_bottom: i32,
    margin_left: i32,
    margin_right: i32,

    exclusive_zone: i32,

    layer: String,
    keyboard_mode: String,
}

impl Default for BarConfig {
    fn default() -> Self {
        Self {
            // 0 = full display width
            width: 0,

            height: 44,

            position: "top".into(),

            margin_top: 0,
            margin_bottom: 0,
            margin_left: 0,
            margin_right: 0,

            exclusive_zone: 44,

            layer: "top".into(),
            keyboard_mode: "none".into(),
        }
    }
}


// ============================================================
// STYLE
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct StyleConfig {
    bar_background: String,
    bar_opacity: f32,

    bar_border_color: String,
    bar_border_width: i32,
    bar_radius: i32,

    module_background: String,
    module_opacity: f32,
    module_hover_background: String,

    module_border_color: String,
    module_border_width: i32,
    module_radius: i32,

    text_color: String,
    muted_color: String,
    accent_color: String,

    workspace_background: String,
    workspace_active_background: String,
    workspace_hover_background: String,
    workspace_urgent_background: String,

    workspace_active_color: String,
    workspace_inactive_color: String,
    workspace_urgent_color: String,

    workspace_border_color: String,
    workspace_border_width: i32,
    workspace_radius: i32,

    clock_background: String,
    clock_opacity: f32,
    clock_hover_background: String,

    clock_border_color: String,
    clock_border_width: i32,
    clock_radius: i32,
}

impl Default for StyleConfig {
    fn default() -> Self {
        Self {
            bar_background: "transparent".into(),
            bar_opacity: 1.0,

            bar_border_color: "transparent".into(),
            bar_border_width: 0,
            bar_radius: 0,

            module_background:
                "rgba(255,255,255,0.055)".into(),

            module_opacity: 1.0,

            module_hover_background:
                "rgba(255,255,255,0.09)".into(),

            module_border_color:
                "transparent".into(),

            module_border_width: 0,
            module_radius: 9,

            text_color: "#cdd6f4".into(),
            muted_color: "#6c7086".into(),
            accent_color: "#89b4fa".into(),

            workspace_background:
                "rgba(255,255,255,0.035)".into(),

            workspace_active_background:
                "rgba(137,180,250,0.20)".into(),

            workspace_hover_background:
                "rgba(255,255,255,0.08)".into(),

            workspace_urgent_background:
                "rgba(243,139,168,0.22)".into(),

            workspace_active_color:
                "#ffffff".into(),

            workspace_inactive_color:
                "#6c7086".into(),

            workspace_urgent_color:
                "#f38ba8".into(),

            workspace_border_color:
                "transparent".into(),

            workspace_border_width: 0,
            workspace_radius: 8,

            clock_background:
                "rgba(255,255,255,0.055)".into(),

            clock_opacity: 1.0,

            clock_hover_background:
                "rgba(255,255,255,0.09)".into(),

            clock_border_color:
                "transparent".into(),

            clock_border_width: 0,
            clock_radius: 9,
        }
    }
}


// ============================================================
// FONT
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct FontConfig {
    family: String,
    size: i32,
    weight: i32,

    logo_size: i32,
    workspace_size: i32,
}

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            family:
                "JetBrainsMono Nerd Font".into(),

            size: 12,
            weight: 650,

            logo_size: 17,
            workspace_size: 12,
        }
    }
}


// ============================================================
// SPACING
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct SpacingConfig {
    zone: i32,
    module: i32,

    horizontal: i32,
    vertical: i32,

    bar_horizontal: i32,
    bar_vertical: i32,
}

impl Default for SpacingConfig {
    fn default() -> Self {
        Self {
            zone: 0,
            module: 4,

            horizontal: 10,
            vertical: 3,

            bar_horizontal: 5,
            bar_vertical: 4,
        }
    }
}


// ============================================================
// LAYOUT
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct LayoutConfig {
    left: Vec<String>,
    center: Vec<String>,
    right: Vec<String>,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        // IMPORTANT:
        // Nothing appears unless user adds it to config.
        Self {
            left: Vec::new(),
            center: Vec::new(),
            right: Vec::new(),
        }
    }
}


// ============================================================
// LOGO
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct LogoConfig {
    enabled: bool,

    // Controls ONLY the icon.
    icon_enabled: bool,

    icon: String,

    icon_color: String,

    background: String,
    hover_background: String,

    padding: i32,
    radius: i32,
}

impl Default for LogoConfig {
    fn default() -> Self {
        Self {
            enabled: true,

            icon_enabled: true,

            icon: "".into(),

            icon_color:
                "#89b4fa".into(),

            background:
                "transparent".into(),

            hover_background:
                "rgba(255,255,255,0.08)"
                    .into(),

            padding: 9,
            radius: 9,
        }
    }
}


// ============================================================
// LOGO TEXT
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct LogoTextConfig {
    enabled: bool,

    text: String,

    color: String,

    background: String,
    hover_background: String,

    padding: i32,
    radius: i32,
}

impl Default for LogoTextConfig {
    fn default() -> Self {
        Self {
            // IMPORTANT:
            // false means NO text widget is created.
            enabled: false,

            text: "Raix".into(),

            color:
                "#cdd6f4".into(),

            background:
                "transparent".into(),

            hover_background:
                "rgba(255,255,255,0.08)"
                    .into(),

            padding: 9,
            radius: 9,
        }
    }
}


// ============================================================
// WORKSPACES
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct WorkspaceConfig {
    enabled: bool,

    numbers: Vec<i32>,

    show_empty: bool,

    format: String,

    padding: i32,
    gap: i32,

    scroll_switch: bool,

    left_click: String,
    right_click: String,
    middle_click: String,

    interval: u64,
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        Self {
            enabled: true,

            numbers:
                vec![1, 2, 3, 4, 5],

            show_empty: true,

            format: "{id}".into(),

            padding: 9,
            gap: 2,

            scroll_switch: true,

            left_click:
                "hyprctl dispatch workspace {id}"
                    .into(),

            right_click: String::new(),
            middle_click: String::new(),

            interval: 500,
        }
    }
}


// ============================================================
// MODULE
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct ModuleConfig {
    enabled: bool,

    format: String,
    icon: String,

    color: String,

    background: String,
    hover_background: String,

    opacity: f32,

    border_color: String,
    border_width: i32,

    padding: i32,
    radius: i32,

    interval: u64,

    tooltip: String,

    left_click: String,
    right_click: String,
    middle_click: String,

    scroll_up: String,
    scroll_down: String,
}

impl Default for ModuleConfig {
    fn default() -> Self {
        Self {
            enabled: true,

            format:
                "{icon}  {value}".into(),

            icon: String::new(),

            color:
                "#cdd6f4".into(),

            background:
                "rgba(255,255,255,0.055)"
                    .into(),

            hover_background:
                "rgba(255,255,255,0.09)"
                    .into(),

            opacity: 1.0,

            border_color:
                "transparent".into(),

            border_width: 0,

            padding: 10,
            radius: 9,

            interval: 1000,

            tooltip: String::new(),

            left_click: String::new(),
            right_click: String::new(),
            middle_click: String::new(),

            scroll_up: String::new(),
            scroll_down: String::new(),
        }
    }
}

impl ModuleConfig {
    fn cpu() -> Self {
        Self {
            format:
                "{icon}  {value}%".into(),

            icon: "󰍛".into(),

            color:
                "#cba6f7".into(),

            ..Default::default()
        }
    }

    fn ram() -> Self {
        Self {
            format:
                "{icon}  {value}%".into(),

            icon: "󰘚".into(),

            color:
                "#a6e3a1".into(),

            ..Default::default()
        }
    }

    fn network() -> Self {
        Self {
            format:
                "{icon}  {value}".into(),

            icon: "󰖩".into(),

            color:
                "#89dceb".into(),

            ..Default::default()
        }
    }

    fn volume() -> Self {
        Self {
            format:
                "{icon}  {value}%".into(),

            icon: "󰕾".into(),

            color:
                "#f5c2e7".into(),

            ..Default::default()
        }
    }

    fn battery() -> Self {
        Self {
            format:
                "{icon}  {value}%".into(),

            icon: "󰁹".into(),

            color:
                "#f9e2af".into(),

            ..Default::default()
        }
    }
}


// ============================================================
// CLOCK
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct ClockConfig {
    enabled: bool,

    format: String,

    color: String,

    background: String,
    hover_background: String,

    opacity: f32,

    border_color: String,
    border_width: i32,

    font_size: i32,
    font_weight: i32,

    padding: i32,
    radius: i32,

    interval: u64,

    left_click: String,
    right_click: String,
    middle_click: String,
}

impl Default for ClockConfig {
    fn default() -> Self {
        Self {
            enabled: true,

            format:
                "%a  %d %b  •  %H:%M".into(),

            color:
                "#89b4fa".into(),

            background:
                "rgba(137,180,250,0.10)"
                    .into(),

            hover_background:
                "rgba(137,180,250,0.18)"
                    .into(),

            opacity: 1.0,

            border_color:
                "transparent".into(),

            border_width: 0,

            font_size: 12,
            font_weight: 800,

            padding: 14,
            radius: 9,

            interval: 1000,

            left_click: String::new(),
            right_click: String::new(),
            middle_click: String::new(),
        }
    }
}


// ============================================================
// CONFIG PATH
// ============================================================

fn config_path() -> PathBuf {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));

    home.join(".config")
        .join("raixbar")
        .join("config.toml")
}


// ============================================================
// LOAD CONFIG
// ============================================================

fn load_config() -> Config {
    let path = config_path();

    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    if !path.exists() {
        let config =
            Config::default();

        if let Ok(data) =
            toml::to_string_pretty(&config)
        {
            let _ =
                fs::write(&path, data);
        }

        return config;
    }

    match fs::read_to_string(&path) {
        Ok(data) => {
            match toml::from_str::<Config>(&data) {
                Ok(config) => config,

                Err(error) => {
                    eprintln!(
                        "RaixBar config error: {error}"
                    );

                    Config::default()
                }
            }
        }

        Err(error) => {
            eprintln!(
                "RaixBar cannot read config: {error}"
            );

            Config::default()
        }
    }
}


// ============================================================
// STATE
// ============================================================

struct AppState {
    labels: HashMap<String, Label>,
    workspace_box: GtkBox,
}

impl AppState {
    fn new() -> Self {
        Self {
            labels:
                HashMap::new(),

            workspace_box:
                GtkBox::new(
                    Orientation::Horizontal,
                    0,
                ),
        }
    }
}


// ============================================================
// MAIN
// ============================================================

fn main() {
    let app =
        Application::builder()
            .application_id(
                "com.raiyan.raixbar",
            )
            .build();

    app.connect_activate(build_ui);

    app.run();
}


// ============================================================
// BUILD UI
// ============================================================

fn build_ui(app: &Application) {
    let config =
        load_config();

    let window =
        ApplicationWindow::builder()
            .application(app)
            .title("RaixBar")
            .decorated(false)
            .resizable(false)
            .build();


    // ========================================================
    // LAYER SHELL
    // ========================================================

    window.init_layer_shell();

    let layer =
        match config
            .bar
            .layer
            .to_lowercase()
            .as_str()
        {
            "bottom" =>
                Layer::Bottom,

            "overlay" =>
                Layer::Overlay,

            _ =>
                Layer::Top,
        };

    window.set_layer(layer);


    let keyboard =
        match config
            .bar
            .keyboard_mode
            .to_lowercase()
            .as_str()
        {
            "exclusive" =>
                KeyboardMode::Exclusive,

            "on_demand" =>
                KeyboardMode::OnDemand,

            _ =>
                KeyboardMode::None,
        };

    window.set_keyboard_mode(
        keyboard,
    );


    // ========================================================
    // ANCHORS
    // ========================================================

    let bottom =
        config
            .bar
            .position
            .eq_ignore_ascii_case(
                "bottom",
            );

    if config.bar.width == 0 {
        window.set_anchor(
            Edge::Left,
            true,
        );

        window.set_anchor(
            Edge::Right,
            true,
        );
    } else {
        window.set_anchor(
            Edge::Left,
            false,
        );

        window.set_anchor(
            Edge::Right,
            false,
        );
    }


    window.set_anchor(
        Edge::Top,
        !bottom,
    );

    window.set_anchor(
        Edge::Bottom,
        bottom,
    );


    window.set_margin(
        Edge::Top,
        config.bar.margin_top,
    );

    window.set_margin(
        Edge::Bottom,
        config.bar.margin_bottom,
    );

    window.set_margin(
        Edge::Left,
        config.bar.margin_left,
    );

    window.set_margin(
        Edge::Right,
        config.bar.margin_right,
    );


    window.set_exclusive_zone(
        config.bar.exclusive_zone,
    );

    window.set_namespace(
        Some("raixbar"),
    );


    // ========================================================
    // CSS
    // ========================================================

    let css =
        CssProvider::new();

    let css_data = format!(
r#"
* {{
    font-family:
        "{font}",
        "Noto Sans",
        sans-serif;
}}

window {{
    background: transparent;
}}

.raix-bar {{
    background: {bar_background};
    opacity: {bar_opacity};

    border:
        {bar_border_width}px solid
        {bar_border_color};

    border-radius:
        {bar_radius}px;

    padding-left:
        {bar_horizontal}px;

    padding-right:
        {bar_horizontal}px;

    padding-top:
        {bar_vertical}px;

    padding-bottom:
        {bar_vertical}px;

    min-height:
        {height}px;
}}

.bar-center {{
    min-height: {height}px;
}}

.left-zone,
.center-zone,
.right-zone {{
    padding: 0;
}}

.left-zone {{
    padding-left: {zone_padding}px;
}}

.right-zone {{
    padding-right: {zone_padding}px;
}}

.module {{
    background:
        {module_background};

    color:
        {text_color};

    opacity:
        {module_opacity};

    border:
        {module_border_width}px solid
        {module_border_color};

    border-radius:
        {module_radius}px;

    padding-left:
        {module_padding}px;

    padding-right:
        {module_padding}px;

    padding-top:
        {module_vertical}px;

    padding-bottom:
        {module_vertical}px;

    font-size:
        {font_size}px;

    font-weight:
        {font_weight};
}}

.module:hover {{
    background:
        {module_hover_background};
}}


/* ============================================================
   CPU
   ============================================================ */

.module-cpu {{
    color: {cpu_color};
    background: {cpu_background};
    opacity: {cpu_opacity};

    border:
        {cpu_border_width}px solid
        {cpu_border_color};

    border-radius:
        {cpu_radius}px;
}}

.module-cpu:hover {{
    background:
        {cpu_hover};
}}


/* ============================================================
   RAM
   ============================================================ */

.module-ram {{
    color: {ram_color};
    background: {ram_background};
    opacity: {ram_opacity};

    border:
        {ram_border_width}px solid
        {ram_border_color};

    border-radius:
        {ram_radius}px;
}}

.module-ram:hover {{
    background:
        {ram_hover};
}}


/* ============================================================
   NETWORK
   ============================================================ */

.module-network {{
    color: {network_color};
    background: {network_background};
    opacity: {network_opacity};

    border:
        {network_border_width}px solid
        {network_border_color};

    border-radius:
        {network_radius}px;
}}

.module-network:hover {{
    background:
        {network_hover};
}}


/* ============================================================
   VOLUME
   ============================================================ */

.module-volume {{
    color: {volume_color};
    background: {volume_background};
    opacity: {volume_opacity};

    border:
        {volume_border_width}px solid
        {volume_border_color};

    border-radius:
        {volume_radius}px;
}}

.module-volume:hover {{
    background:
        {volume_hover};
}}


/* ============================================================
   BATTERY
   ============================================================ */

.module-battery {{
    color: {battery_color};
    background: {battery_background};
    opacity: {battery_opacity};

    border:
        {battery_border_width}px solid
        {battery_border_color};

    border-radius:
        {battery_radius}px;
}}

.module-battery:hover {{
    background:
        {battery_hover};
}}


/* ============================================================
   CLOCK
   ============================================================ */

.module-clock {{
    color: {clock_color};

    background:
        {clock_background};

    opacity:
        {clock_opacity};

    border:
        {clock_border_width}px solid
        {clock_border_color};

    border-radius:
        {clock_radius}px;

    padding-left:
        {clock_padding}px;

    padding-right:
        {clock_padding}px;

    font-size:
        {clock_font_size}px;

    font-weight:
        {clock_font_weight};
}}

.module-clock:hover {{
    background:
        {clock_hover};
}}


/* ============================================================
   LOGO ICON
   ============================================================ */

.logo-icon {{
    color:
        {logo_icon_color};

    background:
        {logo_background};

    font-size:
        {logo_size}px;

    padding-left:
        {logo_padding}px;

    padding-right:
        {logo_padding}px;

    border-radius:
        {logo_radius}px;
}}

.logo-icon:hover {{
    background:
        {logo_hover};
}}


/* ============================================================
   LOGO TEXT
   ============================================================ */

.logo-text {{
    color:
        {logo_text_color};

    background:
        {logo_text_background};

    font-size:
        {font_size}px;

    font-weight:
        800;

    padding-left:
        {logo_text_padding}px;

    padding-right:
        {logo_text_padding}px;

    border-radius:
        {logo_text_radius}px;
}}

.logo-text:hover {{
    background:
        {logo_text_hover};
}}


/* ============================================================
   WORKSPACES
   ============================================================ */

.workspace-container {{
    background:
        {workspace_background};

    padding:
        2px;

    border-radius:
        {workspace_radius}px;
}}

.workspace {{
    color:
        {workspace_inactive_color};

    background:
        transparent;

    padding-left:
        {workspace_padding}px;

    padding-right:
        {workspace_padding}px;

    min-width:
        14px;

    min-height:
        28px;

    border-radius:
        {workspace_radius}px;

    font-size:
        {workspace_size}px;

    font-weight:
        700;
}}

.workspace:hover {{
    color:
        {workspace_active_color};

    background:
        {workspace_hover_background};
}}

.workspace.active {{
    color:
        {workspace_active_color};

    background:
        {workspace_active_background};
}}

.workspace.urgent {{
    color:
        {workspace_urgent_color};

    background:
        {workspace_urgent_background};
}}
"#,

        font =
            config.font.family,

        height =
            config.bar.height,

        bar_background =
            config.style.bar_background,

        bar_opacity =
            config.style.bar_opacity,

        bar_border_color =
            config.style.bar_border_color,

        bar_border_width =
            config.style.bar_border_width,

        bar_radius =
            config.style.bar_radius,

        bar_horizontal =
            config.spacing.bar_horizontal,

        bar_vertical =
            config.spacing.bar_vertical,

        zone_padding =
            config.spacing.zone,

        module_background =
            config.style.module_background,

        module_opacity =
            config.style.module_opacity,

        module_hover_background =
            config.style.module_hover_background,

        module_border_color =
            config.style.module_border_color,

        module_border_width =
            config.style.module_border_width,

        module_radius =
            config.style.module_radius,

        module_padding =
            config.spacing.horizontal,

        module_vertical =
            config.spacing.vertical,

        text_color =
            config.style.text_color,

        font_size =
            config.font.size,

        font_weight =
            config.font.weight,


        cpu_color =
            config.cpu.color,

        cpu_background =
            config.cpu.background,

        cpu_hover =
            config.cpu.hover_background,

        cpu_opacity =
            config.cpu.opacity,

        cpu_border_color =
            config.cpu.border_color,

        cpu_border_width =
            config.cpu.border_width,

        cpu_radius =
            config.cpu.radius,


        ram_color =
            config.ram.color,

        ram_background =
            config.ram.background,

        ram_hover =
            config.ram.hover_background,

        ram_opacity =
            config.ram.opacity,

        ram_border_color =
            config.ram.border_color,

        ram_border_width =
            config.ram.border_width,

        ram_radius =
            config.ram.radius,


        network_color =
            config.network.color,

        network_background =
            config.network.background,

        network_hover =
            config.network.hover_background,

        network_opacity =
            config.network.opacity,

        network_border_color =
            config.network.border_color,

        network_border_width =
            config.network.border_width,

        network_radius =
            config.network.radius,


        volume_color =
            config.volume.color,

        volume_background =
            config.volume.background,

        volume_hover =
            config.volume.hover_background,

        volume_opacity =
            config.volume.opacity,

        volume_border_color =
            config.volume.border_color,

        volume_border_width =
            config.volume.border_width,

        volume_radius =
            config.volume.radius,


        battery_color =
            config.battery.color,

        battery_background =
            config.battery.background,

        battery_hover =
            config.battery.hover_background,

        battery_opacity =
            config.battery.opacity,

        battery_border_color =
            config.battery.border_color,

        battery_border_width =
            config.battery.border_width,

        battery_radius =
            config.battery.radius,


        clock_color =
            config.clock.color,

        clock_background =
            config.clock.background,

        clock_hover =
            config.clock.hover_background,

        clock_opacity =
            config.clock.opacity,

        clock_border_color =
            config.clock.border_color,

        clock_border_width =
            config.clock.border_width,

        clock_radius =
            config.clock.radius,

        clock_padding =
            config.clock.padding,

        clock_font_size =
            config.clock.font_size,

        clock_font_weight =
            config.clock.font_weight,


        logo_icon_color =
            config.logo.icon_color,

        logo_background =
            config.logo.background,

        logo_hover =
            config.logo.hover_background,

        logo_size =
            config.font.logo_size,

        logo_padding =
            config.logo.padding,

        logo_radius =
            config.logo.radius,


        logo_text_color =
            config.logo_text.color,

        logo_text_background =
            config.logo_text.background,

        logo_text_hover =
            config.logo_text.hover_background,

        logo_text_padding =
            config.logo_text.padding,

        logo_text_radius =
            config.logo_text.radius,


        workspace_background =
            config.style.workspace_background,

        workspace_active_background =
            config.style.workspace_active_background,

        workspace_hover_background =
            config.style.workspace_hover_background,

        workspace_urgent_background =
            config.style.workspace_urgent_background,

        workspace_active_color =
            config.style.workspace_active_color,

        workspace_inactive_color =
            config.style.workspace_inactive_color,

        workspace_urgent_color =
            config.style.workspace_urgent_color,

        workspace_padding =
            config.workspaces.padding,

        workspace_radius =
            config.style.workspace_radius,

        workspace_size =
            config.font.workspace_size,
    );

    css.load_from_data(&css_data);

    if let Some(display) =
        gdk::Display::default()
    {
        gtk4::style_context_add_provider_for_display(
            &display,
            &css,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }


    // ========================================================
    // ROOT
    // ========================================================

    let root =
        GtkBox::new(
            Orientation::Vertical,
            0,
        );

    root.set_hexpand(true);
    root.set_vexpand(true);

    root.set_halign(Align::Fill);
    root.set_valign(Align::Fill);


    // ========================================================
    // BAR
    // ========================================================

    let bar =
        GtkBox::new(
            Orientation::Horizontal,
            0,
        );

    bar.add_css_class(
        "raix-bar",
    );

    bar.set_valign(
        Align::Center,
    );

    bar.set_height_request(
        config.bar.height,
    );


    // ========================================================
    // WIDTH
    // ========================================================

    if config.bar.width > 0 {
        /*
         * Fixed width.
         *
         * Important:
         * Layer-shell window itself is allowed to
         * determine its requested size.
         */
        bar.set_hexpand(false);

        bar.set_width_request(
            config.bar.width,
        );

        bar.set_halign(
            Align::Center,
        );

        window.set_default_size(
            config.bar.width,
            config.bar.height,
        );
    } else {
        /*
         * Full screen width.
         */
        bar.set_hexpand(true);

        bar.set_halign(
            Align::Fill,
        );

        window.set_default_size(
            -1,
            config.bar.height,
        );
    }


    // ========================================================
    // CENTER BOX
    // ========================================================

    let center_box =
        CenterBox::new();

    center_box.add_css_class(
        "bar-center",
    );

    center_box.set_hexpand(true);

    center_box.set_halign(
        Align::Fill,
    );

    center_box.set_valign(
        Align::Center,
    );


    // ========================================================
    // ZONES
    // ========================================================

    let left =
        GtkBox::new(
            Orientation::Horizontal,
            config.spacing.module,
        );

    left.add_css_class(
        "left-zone",
    );

    left.set_halign(
        Align::Start,
    );

    left.set_valign(
        Align::Center,
    );


    let center =
        GtkBox::new(
            Orientation::Horizontal,
            config.spacing.module,
        );

    center.add_css_class(
        "center-zone",
    );

    center.set_halign(
        Align::Center,
    );

    center.set_valign(
        Align::Center,
    );


    let right =
        GtkBox::new(
            Orientation::Horizontal,
            config.spacing.module,
        );

    right.add_css_class(
        "right-zone",
    );

    right.set_halign(
        Align::End,
    );

    right.set_valign(
        Align::Center,
    );


    // ========================================================
    // STATE
    // ========================================================

    let state =
        Rc::new(
            RefCell::new(
                AppState::new(),
            ),
        );


    // ========================================================
    // CONFIG DRIVEN MODULES
    // ========================================================

    for module in
        &config.layout.left
    {
        add_module(
            &left,
            module,
            &config,
            state.clone(),
        );
    }

    for module in
        &config.layout.center
    {
        add_module(
            &center,
            module,
            &config,
            state.clone(),
        );
    }

    for module in
        &config.layout.right
    {
        add_module(
            &right,
            module,
            &config,
            state.clone(),
        );
    }


    // ========================================================
    // CENTERBOX
    // ========================================================

    center_box.set_start_widget(
        Some(&left),
    );

    center_box.set_center_widget(
        Some(&center),
    );

    center_box.set_end_widget(
        Some(&right),
    );

    bar.append(
        &center_box,
    );

    root.append(&bar);

    window.set_child(
        Some(&root),
    );


    // ========================================================
    // INITIAL VALUES
    // ========================================================

    update_all_modules(
        state.clone(),
        &config,
    );


    // ========================================================
    // TIMERS
    // ========================================================

    start_module_timers(
        state.clone(),
        &config,
    );


    // ========================================================
    // WORKSPACES
    // ========================================================

    if config.workspaces.enabled {
        let workspace_box =
            state
                .borrow()
                .workspace_box
                .clone();

        let workspace_config =
            config.workspaces.clone();

        update_workspaces(
            &workspace_box,
            &workspace_config,
        );

        glib::timeout_add_local(
            Duration::from_millis(
                workspace_config
                    .interval
                    .max(100),
            ),
            move || {
                update_workspaces(
                    &workspace_box,
                    &workspace_config,
                );

                glib::ControlFlow::Continue
            },
        );
    }


    window.present();
}


// ============================================================
// ADD MODULE
// ============================================================

fn add_module(
    parent: &GtkBox,
    name: &str,
    config: &Config,
    state: Rc<RefCell<AppState>>,
) {
    match name {

        "logo" => {
            add_logo(
                parent,
                config,
            );
        }

        "workspaces" => {
            if config.workspaces.enabled {
                let workspace_box =
                    state
                        .borrow()
                        .workspace_box
                        .clone();

                workspace_box
                    .add_css_class(
                        "workspace-container",
                    );

                parent.append(
                    &workspace_box,
                );
            }
        }

        "cpu" => {
            if config.cpu.enabled {
                add_generic_module(
                    parent,
                    "cpu",
                    &config.cpu,
                    state,
                );
            }
        }

        "ram" => {
            if config.ram.enabled {
                add_generic_module(
                    parent,
                    "ram",
                    &config.ram,
                    state,
                );
            }
        }

        "network" => {
            if config.network.enabled {
                add_generic_module(
                    parent,
                    "network",
                    &config.network,
                    state,
                );
            }
        }

        "volume" => {
            if config.volume.enabled {
                add_generic_module(
                    parent,
                    "volume",
                    &config.volume,
                    state,
                );
            }
        }

        "battery" => {
            if config.battery.enabled {
                add_generic_module(
                    parent,
                    "battery",
                    &config.battery,
                    state,
                );
            }
        }

        "clock" => {
            if config.clock.enabled {
                add_clock(
                    parent,
                    config,
                    state,
                );
            }
        }

        "separator" => {
            let label =
                Label::new(Some("│"));

            label.add_css_class(
                "module",
            );

            label.set_opacity(
                0.35,
            );

            parent.append(
                &label,
            );
        }

        unknown => {
            eprintln!(
                "RaixBar: unknown module `{unknown}`"
            );
        }
    }
}


// ============================================================
// LOGO
//
// IMPORTANT:
//
// logo.enabled
//     = master switch
//
// logo.icon_enabled
//     = icon switch
//
// logo_text.enabled
//     = text switch
//
// They are completely independent.
// ============================================================

fn add_logo(
    parent: &GtkBox,
    config: &Config,
) {
    if !config.logo.enabled {
        return;
    }


    // --------------------------------------------------------
    // ICON
    // --------------------------------------------------------

    if config.logo.icon_enabled
        && !config.logo.icon.is_empty()
    {
        let label =
            Label::new(
                Some(
                    &config.logo.icon,
                ),
            );

        label.add_css_class(
            "logo-icon",
        );

        parent.append(
            &label,
        );
    }


    // --------------------------------------------------------
    // TEXT
    // --------------------------------------------------------

    if config.logo_text.enabled
        && !config.logo_text.text.is_empty()
    {
        let label =
            Label::new(
                Some(
                    &config.logo_text.text,
                ),
            );

        label.add_css_class(
            "logo-text",
        );

        parent.append(
            &label,
        );
    }
}


// ============================================================
// GENERIC MODULE
// ============================================================

fn add_generic_module(
    parent: &GtkBox,
    name: &str,
    module: &ModuleConfig,
    state: Rc<RefCell<AppState>>,
) {
    let label =
        Label::new(Some("..."));

    label.add_css_class(
        "module",
    );

    label.add_css_class(
        &format!("module-{name}"),
    );

    label.set_halign(
        Align::Center,
    );

    label.set_valign(
        Align::Center,
    );

    if !module.tooltip.is_empty() {
        label.set_tooltip_text(
            Some(&module.tooltip),
        );
    }

    setup_clicks(
        &label,
        &module.left_click,
        &module.right_click,
        &module.middle_click,
        &module.scroll_up,
        &module.scroll_down,
    );

    state
        .borrow_mut()
        .labels
        .insert(
            name.to_string(),
            label.clone(),
        );

    parent.append(
        &label,
    );
}


// ============================================================
// CLOCK
// ============================================================

fn add_clock(
    parent: &GtkBox,
    config: &Config,
    state: Rc<RefCell<AppState>>,
) {
    let label =
        Label::new(Some("--:--"));

    label.add_css_class(
        "module",
    );

    label.add_css_class(
        "module-clock",
    );

    label.set_halign(
        Align::Center,
    );

    label.set_valign(
        Align::Center,
    );

    setup_clicks(
        &label,
        &config.clock.left_click,
        &config.clock.right_click,
        &config.clock.middle_click,
        "",
        "",
    );

    state
        .borrow_mut()
        .labels
        .insert(
            "clock".into(),
            label.clone(),
        );

    parent.append(
        &label,
    );
}


// ============================================================
// CLICKS
// ============================================================

fn setup_clicks(
    widget: &Label,

    left: &str,
    right: &str,
    middle: &str,

    scroll_up: &str,
    scroll_down: &str,
) {
    if !left.is_empty()
        || !right.is_empty()
        || !middle.is_empty()
    {
        let gesture =
            GestureClick::new();

        let left =
            left.to_string();

        let right =
            right.to_string();

        let middle =
            middle.to_string();

        gesture.connect_released(
            move |gesture, _, _, _| {
                match gesture.current_button() {
                    1 =>
                        run_command(&left),

                    2 =>
                        run_command(&middle),

                    3 =>
                        run_command(&right),

                    _ => {}
                }
            },
        );

        widget.add_controller(
            gesture,
        );
    }


    if !scroll_up.is_empty()
        || !scroll_down.is_empty()
    {
        let controller =
            EventControllerScroll::new(
                EventControllerScrollFlags::VERTICAL,
            );

        let up =
            scroll_up.to_string();

        let down =
            scroll_down.to_string();

        controller.connect_scroll(
            move |_, _, dy| {
                if dy < 0.0 {
                    run_command(&up);
                }

                if dy > 0.0 {
                    run_command(&down);
                }

                glib::Propagation::Stop
            },
        );

        widget.add_controller(
            controller,
        );
    }
}


// ============================================================
// COMMAND
// ============================================================

fn run_command(command: &str) {
    let command =
        command.trim();

    if command.is_empty() {
        return;
    }

    if let Err(error) =
        Command::new("sh")
            .arg("-c")
            .arg(command)
            .spawn()
    {
        eprintln!(
            "RaixBar command `{command}` failed: {error}"
        );
    }
}


// ============================================================
// TIMERS
// ============================================================

fn start_module_timers(
    state: Rc<RefCell<AppState>>,
    config: &Config,
) {
    start_generic_timer(
        state.clone(),
        "cpu",
        &config.cpu,
        cpu_usage,
    );

    start_generic_timer(
        state.clone(),
        "ram",
        &config.ram,
        ram_usage,
    );

    start_generic_timer(
        state.clone(),
        "network",
        &config.network,
        network_status,
    );

    start_generic_timer(
        state.clone(),
        "volume",
        &config.volume,
        volume_status,
    );

    start_generic_timer(
        state.clone(),
        "battery",
        &config.battery,
        battery_status,
    );


    if config.clock.enabled {
        let state =
            state.clone();

        let cfg =
            config.clock.clone();

        glib::timeout_add_local(
            Duration::from_millis(
                cfg.interval.max(200),
            ),
            move || {
                if let Some(label) =
                    state
                        .borrow()
                        .labels
                        .get("clock")
                        .cloned()
                {
                    update_clock(
                        &label,
                        &cfg.format,
                    );
                }

                glib::ControlFlow::Continue
            },
        );
    }
}


// ============================================================
// GENERIC TIMER
// ============================================================

fn start_generic_timer<F>(
    state: Rc<RefCell<AppState>>,
    name: &str,
    config: &ModuleConfig,
    getter: F,
)
where
    F: Fn() -> String + 'static,
{
    if !config.enabled {
        return;
    }

    let name =
        name.to_string();

    let cfg =
        config.clone();

    glib::timeout_add_local(
        Duration::from_millis(
            cfg.interval.max(100),
        ),
        move || {
            if let Some(label) =
                state
                    .borrow()
                    .labels
                    .get(&name)
                    .cloned()
            {
                let value =
                    getter();

                label.set_text(
                    &format_module(
                        &cfg.format,
                        &cfg.icon,
                        &value,
                    ),
                );
            }

            glib::ControlFlow::Continue
        },
    );
}


// ============================================================
// INITIAL UPDATE
// ============================================================

fn update_all_modules(
    state: Rc<RefCell<AppState>>,
    config: &Config,
) {
    update_label(
        &state,
        "cpu",
        format_module(
            &config.cpu.format,
            &config.cpu.icon,
            &cpu_usage(),
        ),
    );

    update_label(
        &state,
        "ram",
        format_module(
            &config.ram.format,
            &config.ram.icon,
            &ram_usage(),
        ),
    );

    update_label(
        &state,
        "network",
        format_module(
            &config.network.format,
            &config.network.icon,
            &network_status(),
        ),
    );

    update_label(
        &state,
        "volume",
        format_module(
            &config.volume.format,
            &config.volume.icon,
            &volume_status(),
        ),
    );

    update_label(
        &state,
        "battery",
        format_module(
            &config.battery.format,
            &config.battery.icon,
            &battery_status(),
        ),
    );


    if let Some(label) =
        state
            .borrow()
            .labels
            .get("clock")
            .cloned()
    {
        update_clock(
            &label,
            &config.clock.format,
        );
    }
}


fn update_label(
    state:
        &Rc<RefCell<AppState>>,

    name: &str,
    text: String,
) {
    if let Some(label) =
        state
            .borrow()
            .labels
            .get(name)
            .cloned()
    {
        label.set_text(
            &text,
        );
    }
}


// ============================================================
// FORMAT
// ============================================================

fn format_module(
    format: &str,
    icon: &str,
    value: &str,
) -> String {
    format
        .replace(
            "{icon}",
            icon,
        )
        .replace(
            "{value}",
            value,
        )
}


// ============================================================
// CLOCK
// ============================================================

fn update_clock(
    label: &Label,
    format: &str,
) {
    match Command::new("date")
        .arg(
            format!("+{format}"),
        )
        .output()
    {
        Ok(output)
            if output.status.success() =>
        {
            let text =
                String::from_utf8_lossy(
                    &output.stdout,
                )
                .trim()
                .to_string();

            if !text.is_empty() {
                label.set_text(
                    &text,
                );

                return;
            }
        }

        _ => {}
    }

    label.set_text(
        "--:--",
    );
}


// ============================================================
// CPU
// ============================================================

fn read_cpu_stat()
    -> Option<(u64, u64)>
{
    let data =
        fs::read_to_string(
            "/proc/stat",
        )
        .ok()?;

    let line =
        data.lines()
            .find(|line| {
                line.starts_with("cpu ")
            })?;

    let values:
        Vec<u64> =
        line
            .split_whitespace()
            .skip(1)
            .filter_map(
                |v| v.parse().ok(),
            )
            .collect();

    if values.len() < 5 {
        return None;
    }

    let idle =
        values[3] + values[4];

    let total =
        values.iter().sum();

    Some((
        total,
        idle,
    ))
}


thread_local! {
    static PREVIOUS_CPU:
        RefCell<Option<(u64, u64)>> =
        RefCell::new(None);
}


fn cpu_usage() -> String {
    let current =
        match read_cpu_stat() {
            Some(value) =>
                value,

            None =>
                return "0".into(),
        };

    PREVIOUS_CPU.with(
        |previous| {
            let mut previous =
                previous.borrow_mut();

            let usage =
                if let Some(old) =
                    *previous
                {
                    let total_delta =
                        current
                            .0
                            .saturating_sub(
                                old.0,
                            );

                    let idle_delta =
                        current
                            .1
                            .saturating_sub(
                                old.1,
                            );

                    if total_delta == 0 {
                        0
                    } else {
                        (
                            total_delta
                                .saturating_sub(
                                    idle_delta,
                                )
                                as f64
                                / total_delta
                                    as f64
                                * 100.0
                        )
                        .round()
                            as u64
                    }
                } else {
                    0
                };

            *previous =
                Some(current);

            usage.to_string()
        },
    )
}


// ============================================================
// RAM
// ============================================================

fn ram_usage() -> String {
    let data =
        match fs::read_to_string(
            "/proc/meminfo",
        ) {
            Ok(value) =>
                value,

            Err(_) =>
                return "0".into(),
        };

    let mut total =
        0u64;

    let mut available =
        0u64;

    for line in data.lines() {
        if let Some(value) =
            line.strip_prefix(
                "MemTotal:",
            )
        {
            total =
                value
                    .split_whitespace()
                    .next()
                    .and_then(
                        |v| v.parse().ok(),
                    )
                    .unwrap_or(0);
        }

        if let Some(value) =
            line.strip_prefix(
                "MemAvailable:",
            )
        {
            available =
                value
                    .split_whitespace()
                    .next()
                    .and_then(
                        |v| v.parse().ok(),
                    )
                    .unwrap_or(0);
        }
    }

    if total == 0 {
        return "0".into();
    }

    let used =
        total.saturating_sub(
            available,
        );

    (
        used.saturating_mul(100)
            / total
    )
    .to_string()
}


// ============================================================
// NETWORK
// ============================================================

fn network_status() -> String {
    let entries =
        match fs::read_dir(
            "/sys/class/net",
        ) {
            Ok(value) =>
                value,

            Err(_) =>
                return "offline".into(),
        };

    let mut wireless =
        None;

    let mut ethernet =
        None;

    for entry in entries.flatten() {
        let path =
            entry.path();

        let name =
            entry
                .file_name()
                .to_string_lossy()
                .to_string();

        if name == "lo" {
            continue;
        }

        let state =
            fs::read_to_string(
                path.join(
                    "operstate",
                ),
            )
            .unwrap_or_default()
            .trim()
            .to_string();

        if state != "up" {
            continue;
        }

        if path.join(
            "wireless",
        ).exists()
        {
            wireless =
                Some(name);
        } else {
            ethernet =
                Some(name);
        }
    }

    wireless
        .or(ethernet)
        .unwrap_or_else(
            || "offline".into(),
        )
}


// ============================================================
// VOLUME
// ============================================================

fn volume_status() -> String {
    if let Ok(output) =
        Command::new("wpctl")
            .args([
                "get-volume",
                "@DEFAULT_AUDIO_SINK@",
            ])
            .output()
    {
        if output.status.success() {
            let text =
                String::from_utf8_lossy(
                    &output.stdout,
                );

            for part in
                text.split_whitespace()
            {
                if let Some(value) =
                    part.strip_suffix('%')
                {
                    if let Ok(number) =
                        value.parse::<f64>()
                    {
                        return number
                            .round()
                            .to_string();
                    }
                }
            }

            for part in
                text.split_whitespace()
            {
                if let Ok(value) =
                    part.parse::<f64>()
                {
                    if (
                        0.0..=2.0
                    )
                        .contains(
                            &value,
                        )
                    {
                        return (
                            value * 100.0
                        )
                        .round()
                        .to_string();
                    }
                }
            }
        }
    }

    if let Ok(output) =
        Command::new("pactl")
            .args([
                "get-sink-volume",
                "@DEFAULT_SINK@",
            ])
            .output()
    {
        if output.status.success() {
            let text =
                String::from_utf8_lossy(
                    &output.stdout,
                );

            for part in
                text.split_whitespace()
            {
                if let Some(value) =
                    part.strip_suffix('%')
                {
                    if let Ok(number) =
                        value.parse::<u32>()
                    {
                        return number
                            .to_string();
                    }
                }
            }
        }
    }

    "0".into()
}


// ============================================================
// BATTERY
// ============================================================

fn battery_status() -> String {
    let base =
        PathBuf::from(
            "/sys/class/power_supply",
        );

    let entries =
        match fs::read_dir(
            &base,
        ) {
            Ok(value) =>
                value,

            Err(_) =>
                return "--".into(),
        };

    for entry in
        entries.flatten()
    {
        let name =
            entry
                .file_name()
                .to_string_lossy()
                .to_string();

        if !name.starts_with(
            "BAT",
        ) {
            continue;
        }

        if let Ok(value) =
            fs::read_to_string(
                entry
                    .path()
                    .join("capacity"),
            )
        {
            return value
                .trim()
                .to_string();
        }
    }

    "--".into()
}


// ============================================================
// WORKSPACE INFO
// ============================================================

#[derive(Debug, Clone)]
struct WorkspaceInfo {
    id: i32,
    active: bool,
    urgent: bool,
}


// ============================================================
// UPDATE WORKSPACES
// ============================================================

fn update_workspaces(
    container: &GtkBox,
    config: &WorkspaceConfig,
) {
    // GTK4 Box is not IntoIterator.
    //
    // Correct way to remove children:
    // first_child() -> remove()
    //

    while let Some(child) =
        container.first_child()
    {
        container.remove(
            &child,
        );
    }


    let workspaces =
        hyprland_workspaces();


    if !config.numbers.is_empty() {
        let mut found =
            HashMap::new();

        for workspace
            in &workspaces
        {
            found.insert(
                workspace.id,
                workspace.clone(),
            );
        }

        for id in
            &config.numbers
        {
            let workspace =
                if let Some(existing) =
                    found.get(id)
                {
                    existing.clone()
                } else if config.show_empty {
                    WorkspaceInfo {
                        id: *id,
                        active: false,
                        urgent: false,
                    }
                } else {
                    continue;
                };

            add_workspace_button(
                container,
                workspace,
                config,
            );
        }

        return;
    }


    for workspace
        in workspaces
    {
        add_workspace_button(
            container,
            workspace,
            config,
        );
    }
}


// ============================================================
// WORKSPACE BUTTON
// ============================================================

fn add_workspace_button(
    container: &GtkBox,
    workspace: WorkspaceInfo,
    config: &WorkspaceConfig,
) {
    let id =
        workspace.id;

    let text =
        config
            .format
            .replace(
                "{id}",
                &id.to_string(),
            );

    let label =
        Label::new(
            Some(&text),
        );

    label.add_css_class(
        "workspace",
    );


    if workspace.active {
        label.add_css_class(
            "active",
        );
    }

    if workspace.urgent {
        label.add_css_class(
            "urgent",
        );
    }


    let left =
        config
            .left_click
            .replace(
                "{id}",
                &id.to_string(),
            );

    let right =
        config
            .right_click
            .replace(
                "{id}",
                &id.to_string(),
            );

    let middle =
        config
            .middle_click
            .replace(
                "{id}",
                &id.to_string(),
            );


    setup_clicks(
        &label,
        &left,
        &right,
        &middle,
        "",
        "",
    );


    if config.scroll_switch {
        let controller =
            EventControllerScroll::new(
                EventControllerScrollFlags::VERTICAL,
            );

        controller.connect_scroll(
            move |_, _, dy| {
                if dy < 0.0 {
                    run_command(
                        "hyprctl dispatch workspace e+1",
                    );
                } else if dy > 0.0 {
                    run_command(
                        "hyprctl dispatch workspace e-1",
                    );
                }

                glib::Propagation::Stop
            },
        );

        label.add_controller(
            controller,
        );
    }


    container.append(
        &label,
    );
}


// ============================================================
// HYPRLAND WORKSPACES
// ============================================================

fn hyprland_workspaces()
    -> Vec<WorkspaceInfo>
{
    let active =
        get_active_workspace();

    let output =
        match Command::new("hyprctl")
            .args([
                "-j",
                "workspaces",
            ])
            .output()
        {
            Ok(output)
                if output.status.success() =>
            {
                output
            }

            _ =>
                return Vec::new(),
        };

    let text =
        String::from_utf8_lossy(
            &output.stdout,
        );

    parse_workspace_json(
        &text,
        active,
    )
}


// ============================================================
// ACTIVE WORKSPACE
// ============================================================

fn get_active_workspace()
    -> Option<i32>
{
    let output =
        Command::new("hyprctl")
            .args([
                "-j",
                "activeworkspace",
            ])
            .output()
            .ok()?;

    if !output.status.success() {
        return None;
    }

    let text =
        String::from_utf8_lossy(
            &output.stdout,
        );

    extract_json_number(
        &text,
        "id",
    )
}


// ============================================================
// WORKSPACE JSON
// ============================================================

fn parse_workspace_json(
    text: &str,
    active_id: Option<i32>,
) -> Vec<WorkspaceInfo> {
    let mut result =
        Vec::new();

    let mut remaining =
        text;

    while let Some(pos) =
        remaining.find("\"id\"")
    {
        remaining =
            &remaining[pos + 4..];

        let colon =
            match remaining.find(':') {
                Some(value) =>
                    value,

                None =>
                    break,
            };

        let value =
            remaining[
                colon + 1..
            ]
            .trim_start();

        let number =
            value
                .chars()
                .take_while(
                    |c| {
                        c.is_ascii_digit()
                            || *c == '-'
                    },
                )
                .collect::<String>();

        let id =
            match number.parse::<i32>() {
                Ok(value) =>
                    value,

                Err(_) => {
                    if remaining.len()
                        > 1
                    {
                        remaining =
                            &remaining[1..];
                    } else {
                        break;
                    }

                    continue;
                }
            };

        let end =
            remaining.find('}');

        let object =
            match end {
                Some(end) =>
                    &remaining[..end],

                None =>
                    remaining,
            };

        let urgent =
            object.contains(
                "\"urgent\":true",
            );

        let active =
            active_id
                .map(
                    |value| value == id,
                )
                .unwrap_or(false);

        if !result.iter().any(
            |workspace:
                &WorkspaceInfo| {
                workspace.id == id
            },
        ) {
            result.push(
                WorkspaceInfo {
                    id,
                    active,
                    urgent,
                },
            );
        }

        remaining =
            match end {
                Some(end) =>
                    &remaining[
                        end + 1..
                    ],

                None =>
                    break,
            };
    }

    result.sort_by_key(
        |workspace|
            workspace.id,
    );

    result
}


// ============================================================
// JSON NUMBER
// ============================================================

fn extract_json_number(
    text: &str,
    key: &str,
) -> Option<i32> {
    let needle =
        format!("\"{key}\"");

    let position =
        text.find(
            &needle,
        )?;

    let rest =
        &text[
            position
                + needle.len()..
        ];

    let colon =
        rest.find(':')?;

    let value =
        rest[
            colon + 1..
        ]
        .trim_start();

    let number =
        value
            .chars()
            .take_while(
                |c| {
                    c.is_ascii_digit()
                        || *c == '-'
                },
            )
            .collect::<String>();

    number.parse::<i32>().ok()
}