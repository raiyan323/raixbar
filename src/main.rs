use gtk4::gdk;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{
    Align, Application, ApplicationWindow, Box as GtkBox, CenterBox, CssProvider,
    EventControllerScroll, EventControllerScrollFlags, GestureClick, Label, Orientation,
};

use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

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

    custom: HashMap<String, CustomModule>,
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

            custom: HashMap::new(),
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
            width: 0,
            height: 32,
            position: "top".into(),

            margin_top: 0,
            margin_bottom: 0,
            margin_left: 0,
            margin_right: 0,

            exclusive_zone: 32,

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
    bar_border_opacity: f32,
    bar_border_width: i32,
    bar_radius: i32,

    module_background: String,
    module_opacity: f32,
    module_hover_background: String,
    module_hover_opacity: f32,

    module_border_color: String,
    module_border_opacity: f32,
    module_border_width: i32,
    module_radius: i32,

    module_padding: i32,
    module_width: i32,
    module_height: i32,

    text_color: String,
    muted_color: String,
    accent_color: String,

    // Legacy workspace fields.
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
            bar_background: "#000000".into(),
            bar_opacity: 0.65,

            bar_border_color: "#ffffff".into(),
            bar_border_opacity: 0.08,
            bar_border_width: 0,
            bar_radius: 0,

            module_background: "#000000".into(),
            module_opacity: 0.65,
            module_hover_background: "#ffffff".into(),
            module_hover_opacity: 0.10,

            module_border_color: "#ffffff".into(),
            module_border_opacity: 0.10,
            module_border_width: 1,
            module_radius: 8,

            module_padding: 10,
            module_width: 0,
            module_height: 0,

            text_color: "#ffffff".into(),
            muted_color: "#888888".into(),
            accent_color: "#ffffff".into(),

            workspace_background: "transparent".into(),
            workspace_active_background: "#ffffff".into(),
            workspace_hover_background: "#ffffff".into(),
            workspace_urgent_background: "#ffffff".into(),

            workspace_active_color: "#ffffff".into(),
            workspace_inactive_color: "#8f8f98".into(),
            workspace_urgent_color: "#ffffff".into(),

            workspace_border_color: "#ffffff".into(),
            workspace_border_width: 0,
            workspace_radius: 8,

            clock_background: "#000000".into(),
            clock_opacity: 0.65,
            clock_hover_background: "#ffffff".into(),

            clock_border_color: "#ffffff".into(),
            clock_border_width: 1,
            clock_radius: 8,
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
            family: "JetBrainsMono Nerd Font".into(),
            size: 12,
            weight: 600,
            logo_size: 18,
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
        Self {
            left: vec!["logo".into(), "workspaces".into()],

            center: vec!["clock".into()],

            right: vec![
                "cpu".into(),
                "ram".into(),
                "network".into(),
                "volume".into(),
                "battery".into(),
            ],
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
    icon_enabled: bool,

    icon: String,
    icon_color: String,

    background: String,
    hover_background: String,
    opacity: f32,

    border_color: String,
    border_width: i32,
    border_opacity: f32,

    padding: i32,
    radius: i32,

    width: i32,
    height: i32,

    left_click: String,
    right_click: String,
    middle_click: String,
}

impl Default for LogoConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            icon_enabled: true,

            icon: "".into(),
            icon_color: "#ffffff".into(),

            background: "transparent".into(),
            hover_background: "#ffffff".into(),
            opacity: 1.0,

            border_color: "#ffffff".into(),
            border_width: 0,
            border_opacity: 0.0,

            padding: 9,
            radius: 8,

            width: 0,
            height: 0,

            left_click: "wofi --show drun".into(),
            right_click: String::new(),
            middle_click: String::new(),
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
    opacity: f32,

    border_color: String,
    border_width: i32,
    border_opacity: f32,

    padding: i32,
    radius: i32,

    width: i32,
    height: i32,
}

impl Default for LogoTextConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            text: "Raix".into(),
            color: "#ffffff".into(),

            background: "transparent".into(),
            hover_background: "#ffffff".into(),
            opacity: 1.0,

            border_color: "#ffffff".into(),
            border_width: 0,
            border_opacity: 0.0,

            padding: 9,
            radius: 8,

            width: 0,
            height: 0,
        }
    }
}

// ============================================================
// WORKSPACE STATE
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct WorkspaceStateStyle {
    color: String,

    background: String,
    opacity: f32,

    border_color: String,
    border_width: i32,
    border_opacity: f32,

    radius: i32,
}

impl Default for WorkspaceStateStyle {
    fn default() -> Self {
        Self {
            color: "#ffffff".into(),

            background: "#ffffff".into(),
            opacity: 0.15,

            border_color: "#ffffff".into(),
            border_width: 1,
            border_opacity: 0.15,

            radius: 8,
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

    color: String,

    background: String,
    opacity: f32,

    hover_color: String,
    hover_background: String,
    hover_opacity: f32,

    hover_border_color: String,
    hover_border_width: i32,
    hover_border_opacity: f32,
    hover_radius: i32,

    border_color: String,
    border_width: i32,
    border_opacity: f32,

    radius: i32,

    padding: i32,
    gap: i32,

    width: i32,
    height: i32,

    active: WorkspaceStateStyle,
    urgent: WorkspaceStateStyle,

    container_background: String,
    container_opacity: f32,

    container_border_color: String,
    container_border_width: i32,
    container_border_opacity: f32,

    container_radius: i32,
    container_padding: i32,

    left_click: String,
    right_click: String,
    middle_click: String,

    scroll_switch: bool,
    scroll_up: String,
    scroll_down: String,

    interval: u64,
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        Self {
            enabled: true,

            numbers: vec![1, 2, 3, 4, 5],
            show_empty: true,

            format: "{id}".into(),

            color: "#8f8f98".into(),

            background: "transparent".into(),
            opacity: 1.0,

            hover_color: "#ffffff".into(),
            hover_background: "#ffffff".into(),
            hover_opacity: 0.10,

            hover_border_color: "#ffffff".into(),
            hover_border_width: 0,
            hover_border_opacity: 0.15,
            hover_radius: 8,

            border_color: "#ffffff".into(),
            border_width: 0,
            border_opacity: 0.0,

            radius: 8,

            padding: 9,
            gap: 3,

            width: 0,
            height: 30,

            active: WorkspaceStateStyle {
                color: "#ffffff".into(),
                background: "#ffffff".into(),
                opacity: 0.15,

                border_color: "#ffffff".into(),
                border_width: 0,
                border_opacity: 0.0,

                radius: 0,
            },

            urgent: WorkspaceStateStyle {
                color: "#ffffff".into(),
                background: "#ffffff".into(),
                opacity: 0.20,

                border_color: "#ffffff".into(),
                border_width: 0,
                border_opacity: 0.0,

                radius: 0,
            },

            container_background: "#000000".into(),
            container_opacity: 0.35,

            container_border_color: "#ffffff".into(),
            container_border_width: 0,
            container_border_opacity: 0.08,

            container_radius: 8,
            container_padding: 2,

            left_click: "hyprctl dispatch workspace {id}".into(),

            right_click: String::new(),
            middle_click: String::new(),

            scroll_switch: true,

            scroll_up: "hyprctl dispatch workspace e-1".into(),

            scroll_down: "hyprctl dispatch workspace e+1".into(),

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
    border_opacity: f32,

    padding: i32,
    width: i32,
    height: i32,
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

            format: "{icon}  {value}".into(),
            icon: String::new(),
            color: String::new(),

            background: String::new(),
            hover_background: String::new(),

            opacity: -1.0,

            border_color: String::new(),
            border_width: -1,
            border_opacity: -1.0,

            padding: 0,
            width: 0,
            height: 0,
            radius: -1,

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
            format: "{icon}  {value}%".into(),
            icon: "󰍛".into(),
            ..Default::default()
        }
    }

    fn ram() -> Self {
        Self {
            format: "{icon}  {value}%".into(),
            icon: "󰘚".into(),
            ..Default::default()
        }
    }

    fn network() -> Self {
        Self {
            format: "{icon}  {value}".into(),
            icon: "󰖩".into(),
            ..Default::default()
        }
    }

    fn volume() -> Self {
        Self {
            format: "{icon}  {value}%".into(),
            icon: "󰕾".into(),
            ..Default::default()
        }
    }

    fn battery() -> Self {
        Self {
            format: "{icon}  {value}%".into(),
            icon: "󰁹".into(),
            ..Default::default()
        }
    }
}

// ============================================================
// CUSTOM MODULE
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct CustomModule {
    enabled: bool,

    command: String,

    format: String,
    icon: String,
    color: String,

    background: String,
    hover_background: String,

    opacity: f32,

    border_color: String,
    border_width: i32,
    border_opacity: f32,

    padding: i32,
    width: i32,
    height: i32,
    radius: i32,

    interval: u64,

    tooltip: String,

    left_click: String,
    right_click: String,
    middle_click: String,

    scroll_up: String,
    scroll_down: String,
}

impl Default for CustomModule {
    fn default() -> Self {
        Self {
            enabled: true,

            command: String::new(),

            format: "{icon} {value}".into(),
            icon: String::new(),
            color: String::new(),

            background: String::new(),
            hover_background: String::new(),

            opacity: -1.0,

            border_color: String::new(),
            border_width: -1,
            border_opacity: -1.0,

            padding: 0,
            width: 0,
            height: 0,
            radius: -1,

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
    border_opacity: f32,

    font_size: i32,
    font_weight: i32,

    padding: i32,

    width: i32,
    height: i32,

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

            format: "%H:%M".into(),
            color: "#ffffff".into(),

            background: String::new(),
            hover_background: String::new(),

            opacity: -1.0,

            border_color: String::new(),
            border_width: -1,
            border_opacity: -1.0,

            font_size: 12,
            font_weight: 700,

            padding: 12,

            width: 0,
            height: 0,

            radius: -1,

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

    home.join(".config").join("raixbar").join("config.toml")
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
        let config = Config::default();

        if let Ok(data) = toml::to_string_pretty(&config) {
            let _ = fs::write(&path, data);
        }

        return config;
    }

    match fs::read_to_string(&path) {
        Ok(data) => match toml::from_str::<Config>(&data) {
            Ok(config) => config,

            Err(error) => {
                eprintln!("RaixBar config error: {error}");

                Config::default()
            }
        },

        Err(error) => {
            eprintln!("RaixBar cannot read config: {error}");

            Config::default()
        }
    }
}

// ============================================================
// APP STATE
// ============================================================

struct AppState {
    labels: HashMap<String, Label>,
    workspace_box: GtkBox,
}

impl AppState {
    fn new() -> Self {
        Self {
            labels: HashMap::new(),

            workspace_box: GtkBox::new(Orientation::Horizontal, 0),
        }
    }
}

// ============================================================
// COLOR HELPERS
// ============================================================

fn color_with_opacity(color: &str, opacity: f32) -> String {
    let color = color.trim();

    if color.is_empty() {
        return "transparent".into();
    }

    if color.eq_ignore_ascii_case("transparent") {
        return "transparent".into();
    }

    let alpha = opacity.clamp(0.0, 1.0);

    if let Some(hex) = color.strip_prefix('#') {
        let expanded = match hex.len() {
            3 => {
                let mut value = String::with_capacity(6);

                for ch in hex.chars() {
                    value.push(ch);
                    value.push(ch);
                }

                value
            }

            6 => hex.to_string(),

            8 => hex[..6].to_string(),

            _ => return color.to_string(),
        };

        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&expanded[0..2], 16),
            u8::from_str_radix(&expanded[2..4], 16),
            u8::from_str_radix(&expanded[4..6], 16),
        ) {
            return format!("rgba({r},{g},{b},{alpha})");
        }
    }

    if color.starts_with("rgb(") || color.starts_with("rgba(") {
        if let Some(inner) = color
            .split_once('(')
            .and_then(|(_, value)| value.strip_suffix(')'))
        {
            let parts: Vec<&str> = inner.split(',').map(str::trim).collect();

            if parts.len() >= 3 {
                return format!("rgba({},{},{},{alpha})", parts[0], parts[1], parts[2],);
            }
        }
    }

    color.to_string()
}

fn effective_string(individual: &str, global: &str) -> String {
    if individual.trim().is_empty() {
        global.to_string()
    } else {
        individual.to_string()
    }
}

fn effective_i32(individual: i32, global: i32) -> i32 {
    if individual < 0 {
        global
    } else {
        individual
    }
}

fn effective_f32(individual: f32, global: f32) -> f32 {
    if individual < 0.0 {
        global
    } else {
        individual
    }
}

// ============================================================
// INSTALL CSS
// ============================================================

fn install_css(css_data: &str, priority: u32) {
    let provider = CssProvider::new();

    provider.load_from_data(css_data);

    if let Some(display) = gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(&display, &provider, priority);
    }
}

// ============================================================
// MAIN
// ============================================================

fn main() {
    let app = Application::builder()
        .application_id("com.raiyan.raixbar")
        .build();

    app.connect_activate(build_ui);

    app.run();
}

// ============================================================
// BUILD UI
// ============================================================

fn build_ui(app: &Application) {
    let config = load_config();

    let window = ApplicationWindow::builder()
        .application(app)
        .title("RaixBar")
        .decorated(false)
        .resizable(false)
        .build();

    window.init_layer_shell();

    let layer = match config.bar.layer.to_lowercase().as_str() {
        "bottom" => Layer::Bottom,

        "overlay" => Layer::Overlay,

        _ => Layer::Top,
    };

    window.set_layer(layer);

    let keyboard = match config.bar.keyboard_mode.to_lowercase().as_str() {
        "exclusive" => KeyboardMode::Exclusive,

        "on_demand" => KeyboardMode::OnDemand,

        _ => KeyboardMode::None,
    };

    window.set_keyboard_mode(keyboard);

    let bottom = config.bar.position.eq_ignore_ascii_case("bottom");

    let full_width = config.bar.width == 0;

    window.set_anchor(Edge::Left, full_width);

    window.set_anchor(Edge::Right, full_width);

    window.set_anchor(Edge::Top, !bottom);

    window.set_anchor(Edge::Bottom, bottom);

    window.set_margin(Edge::Top, config.bar.margin_top);

    window.set_margin(Edge::Bottom, config.bar.margin_bottom);

    window.set_margin(Edge::Left, config.bar.margin_left);

    window.set_margin(Edge::Right, config.bar.margin_right);

    window.set_exclusive_zone(config.bar.exclusive_zone);

    window.set_namespace(Some("raixbar"));

    // ========================================================
    // GLOBAL CSS
    // ========================================================

    let bar_background = color_with_opacity(&config.style.bar_background, config.style.bar_opacity);

    let bar_border_color = color_with_opacity(
        &config.style.bar_border_color,
        config.style.bar_border_opacity,
    );

    let module_background =
        color_with_opacity(&config.style.module_background, config.style.module_opacity);

    let module_hover_background = color_with_opacity(
        &config.style.module_hover_background,
        config.style.module_hover_opacity,
    );

    let logo_background = color_with_opacity(&config.logo.background, config.logo.opacity);

    let logo_hover_background =
        color_with_opacity(&config.logo.hover_background, config.logo.opacity);

    let logo_border_color =
        color_with_opacity(&config.logo.border_color, config.logo.border_opacity);

    let logo_text_background =
        color_with_opacity(&config.logo_text.background, config.logo_text.opacity);

    let logo_text_hover_background =
        color_with_opacity(&config.logo_text.hover_background, config.logo_text.opacity);

    let logo_text_border_color = color_with_opacity(
        &config.logo_text.border_color,
        config.logo_text.border_opacity,
    );

    // ========================================================
    // WORKSPACE COLORS
    // ========================================================

    let workspace_container_background = color_with_opacity(
        &config.workspaces.container_background,
        config.workspaces.container_opacity,
    );

    let workspace_container_border = color_with_opacity(
        &config.workspaces.container_border_color,
        config.workspaces.container_border_opacity,
    );

    let workspace_background =
        color_with_opacity(&config.workspaces.background, config.workspaces.opacity);

    let workspace_hover_background = color_with_opacity(
        &config.workspaces.hover_background,
        config.workspaces.hover_opacity,
    );

    let workspace_active_background = color_with_opacity(
        &config.workspaces.active.background,
        config.workspaces.active.opacity,
    );

    let workspace_urgent_background = color_with_opacity(
        &config.workspaces.urgent.background,
        config.workspaces.urgent.opacity,
    );

    // ========================================================
    // CLOCK
    // ========================================================

    let clock_background = color_with_opacity(
        &effective_string(&config.clock.background, &config.style.clock_background),
        effective_f32(config.clock.opacity, config.style.clock_opacity),
    );

    let clock_hover_background = color_with_opacity(
        &effective_string(
            &config.clock.hover_background,
            &config.style.clock_hover_background,
        ),
        effective_f32(config.clock.opacity, config.style.clock_opacity),
    );

    let clock_border = color_with_opacity(
        &effective_string(&config.clock.border_color, &config.style.clock_border_color),
        effective_f32(config.clock.border_opacity, 1.0),
    );

    // ========================================================
    // GLOBAL CSS
    // ========================================================

    let css = format!(
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


/* ==========================================================
   BAR
   ========================================================== */

.raix-bar {{
    background:
        {bar_background};

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
}}


/* ==========================================================
   CENTER BOX
   ========================================================== */

.bar-center {{
    min-height:
        {bar_height}px;
}}

.left-zone,
.center-zone,
.right-zone {{
    padding-top: 0;
    padding-bottom: 0;
}}

.left-zone {{
    padding-left:
        {zone}px;
}}

.right-zone {{
    padding-right:
        {zone}px;
}}


/* ==========================================================
   NORMAL MODULE
   ========================================================== */

.module {{
    color:
        {text_color};

    background:
        {module_background};

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
        {vertical}px;

    padding-bottom:
        {vertical}px;

    font-size:
        {font_size}px;

    font-weight:
        {font_weight};
}}

.module:hover {{
    background:
        {module_hover_background};
}}


/* ==========================================================
   LOGO
   ========================================================== */

.logo-icon {{
    color:
        {logo_color};

    background:
        {logo_background};

    border:
        {logo_border_width}px solid
        {logo_border_color};

    border-radius:
        {logo_radius}px;

    font-size:
        {logo_size}px;

    padding-left:
        {logo_padding}px;

    padding-right:
        {logo_padding}px;
}}

.logo-icon:hover {{
    background:
        {logo_hover_background};
}}


/* ==========================================================
   LOGO TEXT
   ========================================================== */

.logo-text {{
    color:
        {logo_text_color};

    background:
        {logo_text_background};

    border:
        {logo_text_border_width}px solid
        {logo_text_border_color};

    border-radius:
        {logo_text_radius}px;

    font-size:
        {font_size}px;

    font-weight:
        800;

    padding-left:
        {logo_text_padding}px;

    padding-right:
        {logo_text_padding}px;
}}

.logo-text:hover {{
    background:
        {logo_text_hover_background};
}}


/* ==========================================================
   WORKSPACE CONTAINER
   ========================================================== */

.workspace-container {{
    background:
        {workspace_container_background};

    border:
        {workspace_container_border_width}px solid
        {workspace_container_border_color};

    border-radius:
        {workspace_container_radius}px;

    padding:
        {workspace_container_padding}px;
}}


/* ==========================================================
   WORKSPACE NORMAL
   ========================================================== */

label.workspace {{
    color: {workspace_color};
    background: {workspace_background};

    padding-left: {workspace_padding}px;
    padding-right: {workspace_padding}px;

    min-width: 14px;
    min-height: {workspace_height}px;

    font-size: {workspace_size}px;
    font-weight: 700;
}}

label.workspace:hover {{
    color: {workspace_hover_color};
    background: {workspace_hover_background};
}}

label.workspace.active {{
    color: #ffffff !important;
    background: {workspace_active_background} !important;
}}

label.workspace.active:hover {{
    color: #ffffff !important;
    background: {workspace_active_background} !important;
}}

label.workspace.urgent {{
    color: {workspace_urgent_color} !important;
    background: {workspace_urgent_background} !important;
}}

label.workspace.urgent:hover {{
    color: {workspace_urgent_color} !important;
    background: {workspace_urgent_background} !important;
}}

/* ==========================================================
   CLOCK
   ========================================================== */

label.module-clock {{
    color:
        {clock_color};

    background:
        {clock_background};

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

label.module-clock:hover {{
    background:
        {clock_hover_background};
}}
"#,
        font = config.font.family,
        bar_height = config.bar.height,
        bar_background = bar_background.trim(),
        bar_border_color = bar_border_color.trim(),
        bar_border_width = config.style.bar_border_width,
        bar_radius = config.style.bar_radius,
        bar_horizontal = config.spacing.bar_horizontal,
        bar_vertical = config.spacing.bar_vertical,
        zone = config.spacing.zone,
        text_color = config.style.text_color.trim(),
        module_background = module_background.trim(),
        module_hover_background = module_hover_background.trim(),
        module_border_color = color_with_opacity(
            &config.style.module_border_color.trim(),
            config.style.module_border_opacity,
        ),
        module_border_width = config.style.module_border_width,
        module_radius = config.style.module_radius,
        module_padding = config.style.module_padding,
        vertical = config.spacing.vertical,
        font_size = config.font.size,
        font_weight = config.font.weight,
        logo_color = config.logo.icon_color.trim(),
        logo_background = logo_background.trim(),
        logo_hover_background = logo_hover_background.trim(),
        logo_border_color = logo_border_color,
        logo_border_width = config.logo.border_width,
        logo_radius = config.logo.radius,
        logo_size = config.font.logo_size,
        logo_padding = config.logo.padding,
        logo_text_color = config.logo_text.color.trim(),
        logo_text_background = logo_text_background.trim(),
        logo_text_hover_background = logo_text_hover_background.trim(),
        logo_text_border_color = logo_text_border_color,
        logo_text_border_width = config.logo_text.border_width,
        logo_text_radius = config.logo_text.radius,
        logo_text_padding = config.logo_text.padding,
        workspace_container_background = workspace_container_background.trim(),
        workspace_container_border_color = workspace_container_border,
        workspace_container_border_width = config.workspaces.container_border_width,
        workspace_container_radius = config.workspaces.container_radius,
        workspace_container_padding = config.workspaces.container_padding,
        workspace_color = config.workspaces.color.trim(),
        workspace_background = workspace_background.trim(),
        workspace_padding = config.workspaces.padding,
        workspace_height = config.workspaces.height,
        workspace_size = config.font.workspace_size,
        workspace_hover_color = config.workspaces.hover_color.trim(),
        workspace_hover_background = workspace_hover_background.trim(),
        workspace_active_background = workspace_active_background.trim(),
        workspace_urgent_color = config.workspaces.urgent.color.trim(),
        workspace_urgent_background = workspace_urgent_background.trim(),
        clock_color = config.clock.color.trim(),
        clock_background = clock_background.trim(),
        clock_hover_background = clock_hover_background.trim(),
        clock_border_color = clock_border,
        clock_border_width =
            effective_i32(config.clock.border_width, config.style.clock_border_width,),
        clock_radius = effective_i32(config.clock.radius, config.style.clock_radius,),
        clock_padding = config.clock.padding,
        clock_font_size = config.clock.font_size,
        clock_font_weight = config.clock.font_weight,
    );

    install_css(&css, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);

    // ========================================================
    // ROOT
    // ========================================================

    let root = GtkBox::new(Orientation::Vertical, 0);

    root.set_hexpand(true);
    root.set_vexpand(true);

    root.set_halign(Align::Fill);

    root.set_valign(Align::Fill);

    // ========================================================
    // BAR
    // ========================================================

    let bar = GtkBox::new(Orientation::Horizontal, 0);

    bar.add_css_class("raix-bar");

    bar.set_valign(Align::Center);

    bar.set_height_request(config.bar.height);

    if config.bar.width > 0 {
        bar.set_hexpand(false);

        bar.set_width_request(config.bar.width);

        bar.set_halign(Align::Center);

        window.set_default_size(config.bar.width, config.bar.height);
    } else {
        bar.set_hexpand(true);

        bar.set_halign(Align::Fill);

        window.set_default_size(-1, config.bar.height);
    }

    // ========================================================
    // CENTER BOX
    // ========================================================

    let center_box = CenterBox::new();

    center_box.add_css_class("bar-center");

    center_box.set_hexpand(true);

    center_box.set_halign(Align::Fill);

    center_box.set_valign(Align::Center);

    let left = GtkBox::new(Orientation::Horizontal, config.spacing.module);

    left.add_css_class("left-zone");

    left.set_halign(Align::Start);

    left.set_valign(Align::Center);

    let center = GtkBox::new(Orientation::Horizontal, config.spacing.module);

    center.add_css_class("center-zone");

    center.set_halign(Align::Center);

    center.set_valign(Align::Center);

    let right = GtkBox::new(Orientation::Horizontal, config.spacing.module);

    right.add_css_class("right-zone");

    right.set_halign(Align::End);

    right.set_valign(Align::Center);

    // ========================================================
    // STATE
    // ========================================================

    let state = Rc::new(RefCell::new(AppState::new()));

    // ========================================================
    // LEFT MODULES
    // ========================================================

    for name in &config.layout.left {
        add_module(&left, name, &config, state.clone());
    }

    // ========================================================
    // CENTER MODULES
    // ========================================================

    for name in &config.layout.center {
        add_module(&center, name, &config, state.clone());
    }

    // ========================================================
    // RIGHT MODULES
    // ========================================================

    for name in &config.layout.right {
        add_module(&right, name, &config, state.clone());
    }

    center_box.set_start_widget(Some(&left));

    center_box.set_center_widget(Some(&center));

    center_box.set_end_widget(Some(&right));

    bar.append(&center_box);

    root.append(&bar);

    window.set_child(Some(&root));

    // ========================================================
    // INITIAL UPDATE
    // ========================================================

    update_all_modules(state.clone(), &config);

    // ========================================================
    // TIMERS
    // ========================================================

    start_module_timers(state.clone(), &config);

    // ========================================================
    // WORKSPACES
    // ========================================================

    if config.workspaces.enabled {
        let workspace_box = state.borrow().workspace_box.clone();

        let workspace_config = config.workspaces.clone();

        update_workspaces(&workspace_box, &workspace_config);

        glib::timeout_add_local(
            Duration::from_millis(workspace_config.interval.max(100)),
            move || {
                update_workspaces(&workspace_box, &workspace_config);

                glib::ControlFlow::Continue
            },
        );
    }

    window.present();
}

// ============================================================
// ADD MODULE
// ============================================================

fn add_module(parent: &GtkBox, name: &str, config: &Config, state: Rc<RefCell<AppState>>) {
    match name {
        "logo" => {
            add_logo(parent, config);
        }

        "workspaces" => {
            if config.workspaces.enabled {
                let workspace_box = state.borrow().workspace_box.clone();

                workspace_box.add_css_class("workspace-container");

                workspace_box.set_spacing(config.workspaces.gap);

                parent.append(&workspace_box);
            }
        }

        "cpu" => {
            if config.cpu.enabled {
                add_builtin_module(parent, "cpu", &config.cpu, config, state);
            }
        }

        "ram" => {
            if config.ram.enabled {
                add_builtin_module(parent, "ram", &config.ram, config, state);
            }
        }

        "network" => {
            if config.network.enabled {
                add_builtin_module(parent, "network", &config.network, config, state);
            }
        }

        "volume" => {
            if config.volume.enabled {
                add_builtin_module(parent, "volume", &config.volume, config, state);
            }
        }

        "battery" => {
            if config.battery.enabled {
                add_builtin_module(parent, "battery", &config.battery, config, state);
            }
        }

        "clock" => {
            if config.clock.enabled {
                add_clock(parent, config, state);
            }
        }

        "separator" => {
            let label = Label::new(Some("│"));

            label.add_css_class("module");

            label.set_opacity(0.35);

            parent.append(&label);
        }

        custom_name if custom_name.starts_with("custom.") => {
            let key = &custom_name["custom.".len()..];

            if let Some(custom) = config.custom.get(key) {
                if custom.enabled {
                    add_custom_module(parent, key, custom, config, state);
                }
            }
        }

        unknown => {
            eprintln!("RaixBar: unknown module `{unknown}`");
        }
    }
}

// ============================================================
// BUILTIN MODULE
// ============================================================

fn add_builtin_module(
    parent: &GtkBox,
    name: &str,
    module: &ModuleConfig,
    config: &Config,
    state: Rc<RefCell<AppState>>,
) {
    let label = Label::new(Some("..."));

    label.add_css_class("module");

    label.add_css_class(&format!("module-{name}"));

    label.set_halign(Align::Center);

    label.set_valign(Align::Center);

    apply_module_css(&label, name, module, config);

    apply_size(&label, module.width, module.height, &config.style);

    if !module.tooltip.is_empty() {
        label.set_tooltip_text(Some(&module.tooltip));
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
        .insert(name.to_string(), label.clone());

    parent.append(&label);
}

// ============================================================
// CUSTOM MODULE
// ============================================================

fn add_custom_module(
    parent: &GtkBox,
    name: &str,
    module: &CustomModule,
    config: &Config,
    state: Rc<RefCell<AppState>>,
) {
    let label = Label::new(Some("..."));

    label.add_css_class("module");

    let class_name = safe_css_name(&format!("custom-{name}"));

    label.add_css_class(&class_name);

    label.set_halign(Align::Center);

    label.set_valign(Align::Center);

    apply_custom_css(&label, &class_name, module, config);

    apply_size(&label, module.width, module.height, &config.style);

    if !module.tooltip.is_empty() {
        label.set_tooltip_text(Some(&module.tooltip));
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
        .insert(format!("custom.{name}"), label.clone());

    parent.append(&label);
}

// ============================================================
// MODULE CSS
// ============================================================

fn apply_module_css(label: &Label, name: &str, module: &ModuleConfig, config: &Config) {
    let opacity = effective_f32(module.opacity, config.style.module_opacity);

    let background = color_with_opacity(
        &effective_string(&module.background, &config.style.module_background),
        opacity,
    );

    let hover_background = color_with_opacity(
        &effective_string(
            &module.hover_background,
            &config.style.module_hover_background,
        ),
        config.style.module_hover_opacity,
    );

    let border_color = color_with_opacity(
        &effective_string(&module.border_color, &config.style.module_border_color),
        effective_f32(module.border_opacity, config.style.module_border_opacity),
    );

    let border_width = effective_i32(module.border_width, config.style.module_border_width);

    let radius = effective_i32(module.radius, config.style.module_radius);

    let padding = if module.padding > 0 {
        module.padding
    } else {
        config.style.module_padding
    };

    let color = if module.color.trim().is_empty() {
        config.style.text_color.clone()
    } else {
        module.color.clone()
    };

    let class_name = format!("raix-module-{name}");

    let css = format!(
        r#"
.{class_name} {{
    color: {color};

    background:
        {background};

    border:
        {border_width}px solid
        {border_color};

    border-radius:
        {radius}px;

    padding-left:
        {padding}px;

    padding-right:
        {padding}px;

    padding-top:
        {vertical}px;

    padding-bottom:
        {vertical}px;
}}

.{class_name}:hover {{
    background:
        {hover_background};
}}
"#,
        class_name = class_name,
        color = color,
        background = background,
        border_width = border_width,
        border_color = border_color,
        radius = radius,
        padding = padding,
        vertical = config.spacing.vertical,
        hover_background = hover_background,
    );

    install_css(&css, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION + 10);

    let _ = label;
}

// ============================================================
// CUSTOM CSS
// ============================================================

fn apply_custom_css(label: &Label, class_name: &str, module: &CustomModule, config: &Config) {
    let opacity = effective_f32(module.opacity, config.style.module_opacity);

    let background = color_with_opacity(
        &effective_string(&module.background, &config.style.module_background),
        opacity,
    );

    let hover_background = color_with_opacity(
        &effective_string(
            &module.hover_background,
            &config.style.module_hover_background,
        ),
        config.style.module_hover_opacity,
    );

    let border_color = color_with_opacity(
        &effective_string(&module.border_color, &config.style.module_border_color),
        effective_f32(module.border_opacity, config.style.module_border_opacity),
    );

    let border_width = effective_i32(module.border_width, config.style.module_border_width);

    let radius = effective_i32(module.radius, config.style.module_radius);

    let padding = if module.padding > 0 {
        module.padding
    } else {
        config.style.module_padding
    };

    let color = if module.color.trim().is_empty() {
        config.style.text_color.clone()
    } else {
        module.color.clone()
    };

    let css = format!(
        r#"
.{class_name} {{
    color:
        {color};

    background:
        {background};

    border:
        {border_width}px solid
        {border_color};

    border-radius:
        {radius}px;

    padding-left:
        {padding}px;

    padding-right:
        {padding}px;

    padding-top:
        {vertical}px;

    padding-bottom:
        {vertical}px;
}}

.{class_name}:hover {{
    background:
        {hover_background};
}}
"#,
        class_name = class_name,
        color = color,
        background = background,
        border_width = border_width,
        border_color = border_color,
        radius = radius,
        padding = padding,
        vertical = config.spacing.vertical,
        hover_background = hover_background,
    );

    install_css(&css, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION + 10);

    let _ = label;
}

// ============================================================
// SIZE
// ============================================================

fn apply_size(label: &Label, width: i32, height: i32, global: &StyleConfig) {
    let final_width = if width > 0 {
        width
    } else {
        global.module_width
    };

    let final_height = if height > 0 {
        height
    } else {
        global.module_height
    };

    if final_width > 0 {
        label.set_width_request(final_width);
    }

    if final_height > 0 {
        label.set_height_request(final_height);
    }
}

// ============================================================
// CLOCK
// ============================================================

fn add_clock(parent: &GtkBox, config: &Config, state: Rc<RefCell<AppState>>) {
    let label = Label::new(Some("--:--"));

    label.add_css_class("module");

    label.add_css_class("module-clock");

    label.set_halign(Align::Center);

    label.set_valign(Align::Center);

    setup_clicks(
        &label,
        &config.clock.left_click,
        &config.clock.right_click,
        &config.clock.middle_click,
        "",
        "",
    );

    if config.clock.width > 0 {
        label.set_width_request(config.clock.width);
    }

    if config.clock.height > 0 {
        label.set_height_request(config.clock.height);
    }

    state
        .borrow_mut()
        .labels
        .insert("clock".into(), label.clone());

    parent.append(&label);
}

// ============================================================
// LOGO
// ============================================================

fn add_logo(parent: &GtkBox, config: &Config) {
    if !config.logo.enabled {
        return;
    }

    if config.logo.icon_enabled && !config.logo.icon.is_empty() {
        let label = Label::new(Some(&config.logo.icon));

        label.add_css_class("logo-icon");

        label.set_halign(Align::Center);

        label.set_valign(Align::Center);

        if config.logo.width > 0 {
            label.set_width_request(config.logo.width);
        }

        if config.logo.height > 0 {
            label.set_height_request(config.logo.height);
        }

        setup_clicks(
            &label,
            &config.logo.left_click,
            &config.logo.right_click,
            &config.logo.middle_click,
            "",
            "",
        );

        parent.append(&label);
    }

    if config.logo_text.enabled && !config.logo_text.text.is_empty() {
        let label = Label::new(Some(&config.logo_text.text));

        label.add_css_class("logo-text");

        if config.logo_text.width > 0 {
            label.set_width_request(config.logo_text.width);
        }

        if config.logo_text.height > 0 {
            label.set_height_request(config.logo_text.height);
        }

        parent.append(&label);
    }
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
    if !left.is_empty() || !right.is_empty() || !middle.is_empty() {
        let gesture = GestureClick::new();

        let left = left.to_string();

        let right = right.to_string();

        let middle = middle.to_string();

        gesture.connect_released(
            move |gesture, _n_press, _x, _y| match gesture.current_button() {
                1 => run_command(&left),

                2 => run_command(&middle),

                3 => run_command(&right),

                _ => {}
            },
        );

        widget.add_controller(gesture);
    }

    if !scroll_up.is_empty() || !scroll_down.is_empty() {
        let controller = EventControllerScroll::new(EventControllerScrollFlags::VERTICAL);

        let up = scroll_up.to_string();

        let down = scroll_down.to_string();

        controller.connect_scroll(move |_, _, dy| {
            if dy < 0.0 {
                run_command(&up);
            } else if dy > 0.0 {
                run_command(&down);
            }

            glib::Propagation::Stop
        });

        widget.add_controller(controller);
    }
}

// ============================================================
// COMMAND
// ============================================================

fn run_command(command: &str) {
    let command = command.trim();

    if command.is_empty() {
        return;
    }

    if let Err(error) = Command::new("sh").arg("-c").arg(command).spawn() {
        eprintln!("RaixBar command `{command}` failed: {error}");
    }
}

fn command_output(command: &str) -> String {
    if command.trim().is_empty() {
        return String::new();
    }

    match Command::new("sh").arg("-c").arg(command).output() {
        Ok(output) => {
            if !output.status.success() {
                return String::new();
            }

            String::from_utf8_lossy(&output.stdout).trim().to_string()
        }

        Err(error) => {
            eprintln!("RaixBar custom command failed: {error}");

            String::new()
        }
    }
}

// ============================================================
// FORMAT
// ============================================================

fn format_module(format: &str, icon: &str, value: &str) -> String {
    format.replace("{icon}", icon).replace("{value}", value)
}

// ============================================================
// CLOCK UPDATE
// ============================================================

fn update_clock(label: &Label, format: &str) {
    match Command::new("date").arg(format!("+{format}")).output() {
        Ok(output) if output.status.success() => {
            let text = String::from_utf8_lossy(&output.stdout).trim().to_string();

            if !text.is_empty() {
                label.set_text(&text);

                return;
            }
        }

        _ => {}
    }

    label.set_text("--:--");
}

// ============================================================
// CPU
// ============================================================

fn read_cpu_stat() -> Option<(u64, u64)> {
    let data = fs::read_to_string("/proc/stat").ok()?;

    let line = data.lines().find(|line| line.starts_with("cpu "))?;

    let values: Vec<u64> = line
        .split_whitespace()
        .skip(1)
        .filter_map(|value| value.parse().ok())
        .collect();

    if values.len() < 5 {
        return None;
    }

    let idle = values[3] + values[4];

    let total = values.iter().sum();

    Some((total, idle))
}

thread_local! {
    static PREVIOUS_CPU:
        RefCell<Option<(u64, u64)>> =
            RefCell::new(None);
}

fn cpu_usage() -> String {
    let current = match read_cpu_stat() {
        Some(value) => value,

        None => return "0".into(),
    };

    PREVIOUS_CPU.with(|previous| {
        let mut previous = previous.borrow_mut();

        let usage = match *previous {
            Some(old) => {
                let total_delta = current.0.saturating_sub(old.0);

                let idle_delta = current.1.saturating_sub(old.1);

                if total_delta == 0 {
                    0
                } else {
                    (total_delta.saturating_sub(idle_delta) as f64 / total_delta as f64 * 100.0)
                        .round() as u64
                }
            }

            None => 0,
        };

        *previous = Some(current);

        usage.to_string()
    })
}

// ============================================================
// RAM
// ============================================================

fn ram_usage() -> String {
    let data = match fs::read_to_string("/proc/meminfo") {
        Ok(value) => value,

        Err(_) => return "0".into(),
    };

    let mut total = 0u64;

    let mut available = 0u64;

    for line in data.lines() {
        if let Some(value) = line.strip_prefix("MemTotal:") {
            total = value
                .split_whitespace()
                .next()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0);
        }

        if let Some(value) = line.strip_prefix("MemAvailable:") {
            available = value
                .split_whitespace()
                .next()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0);
        }
    }

    if total == 0 {
        return "0".into();
    }

    let used = total.saturating_sub(available);

    (used.saturating_mul(100) / total).to_string()
}

// ============================================================
// NETWORK
// ============================================================

fn network_status() -> String {
    let entries = match fs::read_dir("/sys/class/net") {
        Ok(value) => value,

        Err(_) => return "offline".into(),
    };

    let mut wireless = None;

    let mut ethernet = None;

    for entry in entries.flatten() {
        let path = entry.path();

        let name = entry.file_name().to_string_lossy().to_string();

        if name == "lo" {
            continue;
        }

        let state = fs::read_to_string(path.join("operstate"))
            .unwrap_or_default()
            .trim()
            .to_string();

        if state != "up" {
            continue;
        }

        if path.join("wireless").exists() {
            wireless = Some(name);
        } else {
            ethernet = Some(name);
        }
    }

    wireless.or(ethernet).unwrap_or_else(|| "offline".into())
}

// ============================================================
// VOLUME
// ============================================================

fn volume_status() -> String {
    if let Ok(output) = Command::new("wpctl")
        .args(["get-volume", "@DEFAULT_AUDIO_SINK@"])
        .output()
    {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);

            for part in text.split_whitespace() {
                if let Some(value) = part.strip_suffix('%') {
                    if let Ok(number) = value.parse::<f64>() {
                        return number.round().to_string();
                    }
                }
            }

            for part in text.split_whitespace() {
                if let Ok(value) = part.parse::<f64>() {
                    if (0.0..=2.0).contains(&value) {
                        return (value * 100.0).round().to_string();
                    }
                }
            }
        }
    }

    if let Ok(output) = Command::new("pactl")
        .args(["get-sink-volume", "@DEFAULT_SINK@"])
        .output()
    {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);

            for part in text.split_whitespace() {
                if let Some(value) = part.strip_suffix('%') {
                    if let Ok(number) = value.parse::<u32>() {
                        return number.to_string();
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
    let base = PathBuf::from("/sys/class/power_supply");

    let entries = match fs::read_dir(&base) {
        Ok(value) => value,

        Err(_) => return "--".into(),
    };

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();

        if !name.starts_with("BAT") {
            continue;
        }

        if let Ok(value) = fs::read_to_string(entry.path().join("capacity")) {
            return value.trim().to_string();
        }
    }

    "--".into()
}

// ============================================================
// WORKSPACE DATA
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

fn update_workspaces(container: &GtkBox, config: &WorkspaceConfig) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }

    let workspaces = hyprland_workspaces();

    if !config.numbers.is_empty() {
        let mut found = HashMap::new();

        for workspace in &workspaces {
            found.insert(workspace.id, workspace.clone());
        }

        for id in &config.numbers {
            let workspace = match found.get(id) {
                Some(existing) => existing.clone(),

                None if config.show_empty => WorkspaceInfo {
                    id: *id,
                    active: false,
                    urgent: false,
                },

                None => continue,
            };

            add_workspace_button(container, workspace, config);
        }

        return;
    }

    for workspace in workspaces {
        add_workspace_button(container, workspace, config);
    }
}

// ============================================================
// WORKSPACE BUTTON
// ============================================================

fn add_workspace_button(container: &GtkBox, workspace: WorkspaceInfo, config: &WorkspaceConfig) {
    let id = workspace.id;

    let text = config.format.replace("{id}", &id.to_string());

    let label = Label::new(Some(&text));

    // IMPORTANT:
    // Workspace state is represented ONLY
    // by CSS classes.
    //
    // No widget opacity is used.
    label.add_css_class("workspace");

    if workspace.active {
        label.add_css_class("active");
    }

    if workspace.urgent {
        label.add_css_class("urgent");
    }

    label.set_halign(Align::Center);

    label.set_valign(Align::Center);

    // ========================================================
    // CLICK
    // ========================================================

    let left = config.left_click.replace("{id}", &id.to_string());

    let right = config.right_click.replace("{id}", &id.to_string());

    let middle = config.middle_click.replace("{id}", &id.to_string());

    setup_clicks(&label, &left, &right, &middle, "", "");

    // ========================================================
    // SCROLL
    // ========================================================

    if config.scroll_switch {
        let scroll_up = config.scroll_up.clone();

        let scroll_down = config.scroll_down.clone();

        let controller = EventControllerScroll::new(EventControllerScrollFlags::VERTICAL);

        controller.connect_scroll(move |_, _, dy| {
            if dy < 0.0 {
                run_command(&scroll_up);
            } else if dy > 0.0 {
                run_command(&scroll_down);
            }

            glib::Propagation::Stop
        });

        label.add_controller(controller);
    }

    // ========================================================
    // SIZE
    // ========================================================

    if config.width > 0 {
        label.set_width_request(config.width);
    }

    if config.height > 0 {
        label.set_height_request(config.height);
    }

    container.append(&label);
}

// ============================================================
// HYPRLAND WORKSPACES
// ============================================================

fn hyprland_workspaces() -> Vec<WorkspaceInfo> {
    let active = get_active_workspace();

    let output = match Command::new("hyprctl").args(["-j", "workspaces"]).output() {
        Ok(output) if output.status.success() => output,

        _ => return Vec::new(),
    };

    let text = String::from_utf8_lossy(&output.stdout);

    parse_workspace_json(&text, active)
}

fn get_active_workspace() -> Option<i32> {
    let output = Command::new("hyprctl")
        .args(["-j", "activeworkspace"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout);

    extract_json_number(&text, "id")
}

// ============================================================
// SIMPLE HYPRLAND JSON PARSER
// ============================================================

fn parse_workspace_json(text: &str, active_id: Option<i32>) -> Vec<WorkspaceInfo> {
    let mut result = Vec::new();

    let mut remaining = text;

    while let Some(pos) = remaining.find("\"id\"") {
        remaining = &remaining[pos + 4..];

        let colon = match remaining.find(':') {
            Some(value) => value,

            None => break,
        };

        let value = remaining[colon + 1..].trim_start();

        let number = value
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '-')
            .collect::<String>();

        let id = match number.parse::<i32>() {
            Ok(value) => value,

            Err(_) => {
                if remaining.len() > 1 {
                    remaining = &remaining[1..];
                } else {
                    break;
                }

                continue;
            }
        };

        let object_end = find_json_object_end(remaining);

        let object = &remaining[..object_end];

        let urgent = object.contains("\"urgent\":true") || object.contains("\"urgent\": true");

        let active = active_id.map(|value| value == id).unwrap_or(false);

        if !result
            .iter()
            .any(|workspace: &WorkspaceInfo| workspace.id == id)
        {
            result.push(WorkspaceInfo { id, active, urgent });
        }

        if object_end >= remaining.len() {
            break;
        }

        remaining = &remaining[object_end + 1..];
    }

    result.sort_by_key(|workspace| workspace.id);

    result
}

fn find_json_object_end(text: &str) -> usize {
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    for (index, ch) in text.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
                continue;
            }

            if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }

            continue;
        }

        match ch {
            '"' => {
                in_string = true;
            }

            '{' => {
                depth += 1;
            }

            '}' => {
                if depth == 0 {
                    return index;
                }

                depth -= 1;

                if depth == 0 {
                    return index;
                }
            }

            _ => {}
        }
    }

    text.len()
}

fn extract_json_number(text: &str, key: &str) -> Option<i32> {
    let needle = format!("\"{key}\"");

    let position = text.find(&needle)?;

    let rest = &text[position + needle.len()..];

    let colon = rest.find(':')?;

    let value = rest[colon + 1..].trim_start();

    let number = value
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '-')
        .collect::<String>();

    number.parse::<i32>().ok()
}

// ============================================================
// TIMERS
// ============================================================

fn start_module_timers(state: Rc<RefCell<AppState>>, config: &Config) {
    start_builtin_timer(state.clone(), "cpu", &config.cpu, cpu_usage);

    start_builtin_timer(state.clone(), "ram", &config.ram, ram_usage);

    start_builtin_timer(state.clone(), "network", &config.network, network_status);

    start_builtin_timer(state.clone(), "volume", &config.volume, volume_status);

    start_builtin_timer(state.clone(), "battery", &config.battery, battery_status);

    // ========================================================
    // CUSTOM MODULES
    // ========================================================

    for (name, module) in &config.custom {
        if !module.enabled {
            continue;
        }

        let label_name = format!("custom.{name}");

        let cfg = module.clone();

        let state_clone = state.clone();

        glib::timeout_add_local(Duration::from_millis(cfg.interval.max(100)), move || {
            if let Some(label) = state_clone.borrow().labels.get(&label_name).cloned() {
                let value = command_output(&cfg.command);

                let text = format_module(&cfg.format, &cfg.icon, &value);

                label.set_text(&text);
            }

            glib::ControlFlow::Continue
        });
    }

    // ========================================================
    // CLOCK
    // ========================================================

    if config.clock.enabled {
        let state = state.clone();

        let cfg = config.clock.clone();

        glib::timeout_add_local(Duration::from_millis(cfg.interval.max(200)), move || {
            if let Some(label) = state.borrow().labels.get("clock").cloned() {
                update_clock(&label, &cfg.format);
            }

            glib::ControlFlow::Continue
        });
    }
}

fn start_builtin_timer<F>(
    state: Rc<RefCell<AppState>>,
    name: &str,
    config: &ModuleConfig,
    getter: F,
) where
    F: Fn() -> String + 'static,
{
    if !config.enabled {
        return;
    }

    let name = name.to_string();

    let cfg = config.clone();

    glib::timeout_add_local(Duration::from_millis(cfg.interval.max(100)), move || {
        if let Some(label) = state.borrow().labels.get(&name).cloned() {
            let value = getter();

            let text = format_module(&cfg.format, &cfg.icon, &value);

            label.set_text(&text);
        }

        glib::ControlFlow::Continue
    });
}

// ============================================================
// INITIAL UPDATE
// ============================================================

fn update_all_modules(state: Rc<RefCell<AppState>>, config: &Config) {
    update_label(
        &state,
        "cpu",
        format_module(&config.cpu.format, &config.cpu.icon, &cpu_usage()),
    );

    update_label(
        &state,
        "ram",
        format_module(&config.ram.format, &config.ram.icon, &ram_usage()),
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
        format_module(&config.volume.format, &config.volume.icon, &volume_status()),
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

    for (name, module) in &config.custom {
        if !module.enabled {
            continue;
        }

        let key = format!("custom.{name}");

        let value = command_output(&module.command);

        let text = format_module(&module.format, &module.icon, &value);

        update_label(&state, &key, text);
    }

    if let Some(label) = state.borrow().labels.get("clock").cloned() {
        update_clock(&label, &config.clock.format);
    }
}

fn update_label(state: &Rc<RefCell<AppState>>, name: &str, text: String) {
    if let Some(label) = state.borrow().labels.get(name).cloned() {
        label.set_text(&text);
    }
}

// ============================================================
// SAFE CSS NAME
// ============================================================

fn safe_css_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect()
}
