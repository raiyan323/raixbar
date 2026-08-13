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
// RAIXBAR
// GTK4 + Layer Shell
// Waybar-style 3-zone layout
//
// LEFT   |   CENTER   |   RIGHT
// ============================================================


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
    workspaces: WorkspaceConfig,

    cpu: ModuleConfig,
    ram: ModuleConfig,
    network: ModuleConfig,
    volume: ModuleConfig,
    battery: ModuleConfig,

    clock: ClockConfig,

    css: CssConfig,
}


// ============================================================
// BAR CONFIG
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct BarConfig {
    width: i32,
    height: i32,

    position: String,
    alignment: String,

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
            height: 44,

            position: "top".into(),
            alignment: "center".into(),

            margin_top: 0,
            margin_bottom: 0,
            margin_left: 0,
            margin_right: 0,

            exclusive_zone: 52,

            layer: "top".into(),
            keyboard_mode: "none".into(),
        }
    }
}


// ============================================================
// STYLE CONFIG
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct StyleConfig {
    background: String,
    background_hover: String,

    opacity: f32,

    text_color: String,
    muted_color: String,
    accent_color: String,

    border_color: String,
    border_width: i32,
    border_radius: i32,

    module_background: String,
    module_hover_background: String,
    module_radius: i32,

    workspace_background: String,
    workspace_active_background: String,
    workspace_hover_background: String,
    workspace_urgent_background: String,

    workspace_active_color: String,
    workspace_inactive_color: String,
    workspace_urgent_color: String,
}

impl Default for StyleConfig {
    fn default() -> Self {
        Self {
            background:
                "rgba(17,17,27,0.94)".into(),

            background_hover:
                "rgba(255,255,255,0.07)".into(),

            opacity: 1.0,

            text_color:
                "#cdd6f4".into(),

            muted_color:
                "#6c7086".into(),

            accent_color:
                "#89b4fa".into(),

            border_color:
                "rgba(255,255,255,0.08)".into(),

            border_width: 0,

            border_radius: 0,

            module_background:
                "transparent".into(),

            module_hover_background:
                "rgba(255,255,255,0.065)".into(),

            module_radius: 9,

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
        }
    }
}


// ============================================================
// FONT CONFIG
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

            logo_size: 20,
            workspace_size: 12,
        }
    }
}


// ============================================================
// SPACING CONFIG
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct SpacingConfig {
    module: i32,

    horizontal: i32,
    vertical: i32,

    bar_padding: i32,
}

impl Default for SpacingConfig {
    fn default() -> Self {
        Self {
            module: 3,
            horizontal: 10,
            vertical: 3,
            bar_padding: 5,
        }
    }
}


// ============================================================
// LAYOUT CONFIG
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
            left: vec![
                "logo".into(),
                "workspaces".into(),
            ],

            center: vec![
                "clock".into(),
            ],

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
// LOGO CONFIG
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct LogoConfig {
    enabled: bool,

    icon: String,
    text: String,

    icon_color: String,
    text_color: String,

    icon_size: i32,
}

impl Default for LogoConfig {
    fn default() -> Self {
        Self {
            enabled: true,

            icon: "".into(),
            text: "Raix".into(),

            icon_color: "#89b4fa".into(),
            text_color: "#cdd6f4".into(),

            icon_size: 16,
        }
    }
}


// ============================================================
// WORKSPACE CONFIG
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct WorkspaceConfig {
    enabled: bool,

    show_empty: bool,
    show_special: bool,

    numbers: Vec<i32>,

    format: String,

    active_color: String,
    inactive_color: String,
    urgent_color: String,

    active_background: String,
    inactive_background: String,
    hover_background: String,
    urgent_background: String,

    padding: i32,
    radius: i32,

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

            show_empty: true,
            show_special: false,

            numbers: vec![
                1, 2, 3, 4, 5,
                6, 7, 8, 9,
            ],

            format: "{id}".into(),

            active_color:
                "#ffffff".into(),

            inactive_color:
                "#6c7086".into(),

            urgent_color:
                "#f38ba8".into(),

            active_background:
                "rgba(137,180,250,0.20)"
                    .into(),

            inactive_background:
                "transparent".into(),

            hover_background:
                "rgba(255,255,255,0.08)"
                    .into(),

            urgent_background:
                "rgba(243,139,168,0.22)"
                    .into(),

            padding: 9,
            radius: 8,

            scroll_switch: true,

            left_click:
                "hyprctl dispatch workspace {id}"
                    .into(),

            right_click: "".into(),
            middle_click: "".into(),

            interval: 500,
        }
    }
}


// ============================================================
// MODULE CONFIG
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct ModuleConfig {
    enabled: bool,

    format: String,
    icon: String,

    color: String,
    background: String,

    font_size: i32,
    font_weight: i32,

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

            icon: "".into(),

            color:
                "#e8edf5".into(),

            background:
                "transparent".into(),

            font_size: 12,
            font_weight: 650,

            padding: 10,
            radius: 9,

            interval: 1000,

            tooltip: "".into(),

            left_click: "".into(),
            right_click: "".into(),
            middle_click: "".into(),

            scroll_up: "".into(),
            scroll_down: "".into(),
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
// CLOCK CONFIG
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct ClockConfig {
    enabled: bool,

    format: String,

    color: String,

    font_size: i32,
    font_weight: i32,

    padding: i32,
    radius: i32,

    interval: u64,

    left_click: String,
    right_click: String,
    middle_click: String,

    scroll_up: String,
    scroll_down: String,
}

impl Default for ClockConfig {
    fn default() -> Self {
        Self {
            enabled: true,

            format:
                "%a  %d %b  •  %H:%M".into(),

            color:
                "#89b4fa".into(),

            font_size: 12,
            font_weight: 800,

            padding: 14,
            radius: 9,

            interval: 1000,

            left_click: "".into(),
            right_click: "".into(),
            middle_click: "".into(),

            scroll_up: "".into(),
            scroll_down: "".into(),
        }
    }
}


// ============================================================
// CSS CONFIG
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct CssConfig {
    custom: String,
}

impl Default for CssConfig {
    fn default() -> Self {
        Self {
            custom: "".into(),
        }
    }
}


// ============================================================
// CONFIG DEFAULT
// ============================================================

impl Default for Config {
    fn default() -> Self {
        Self {
            bar: BarConfig::default(),
            style: StyleConfig::default(),
            font: FontConfig::default(),
            spacing: SpacingConfig::default(),
            layout: LayoutConfig::default(),

            logo: LogoConfig::default(),
            workspaces:
                WorkspaceConfig::default(),

            cpu: ModuleConfig::cpu(),
            ram: ModuleConfig::ram(),
            network: ModuleConfig::network(),
            volume: ModuleConfig::volume(),
            battery: ModuleConfig::battery(),

            clock:
                ClockConfig::default(),

            css:
                CssConfig::default(),
        }
    }
}


// ============================================================
// CONFIG PATH
// ============================================================

fn config_path() -> PathBuf {
    let home =
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(
                || PathBuf::from("."),
            );

    home.join(".config")
        .join("raixbar")
        .join("config.toml")
}


// ============================================================
// LOAD CONFIG
// ============================================================

fn load_config() -> Config {
    let path =
        config_path();

    if let Some(parent) =
        path.parent()
    {
        let _ =
            fs::create_dir_all(parent);
    }

    if !path.exists() {
        let config =
            Config::default();

        match toml::to_string_pretty(
            &config,
        ) {
            Ok(data) => {
                if let Err(error) =
                    fs::write(
                        &path,
                        data,
                    )
                {
                    eprintln!(
                        "RaixBar: cannot create config: {error}"
                    );
                }
            }

            Err(error) => {
                eprintln!(
                    "RaixBar: cannot serialize config: {error}"
                );
            }
        }

        return config;
    }

    match fs::read_to_string(
        &path,
    ) {
        Ok(data) => {
            match toml::from_str::<Config>(
                &data,
            ) {
                Ok(config) =>
                    config,

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
// MAIN
// ============================================================

fn main() {
    let app =
        Application::builder()
            .application_id(
                "com.raiyan.raixbar",
            )
            .build();

    app.connect_activate(
        build_ui,
    );

    app.run();
}


// ============================================================
// BUILD UI
// ============================================================

fn build_ui(
    app: &Application,
) {
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
        match config.bar.layer
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
    // FULL WIDTH ANCHOR
    // ========================================================

    window.set_anchor(
        Edge::Left,
        true,
    );

    window.set_anchor(
        Edge::Right,
        true,
    );


    let position =
        config.bar.position
            .to_lowercase();


    if position == "bottom" {
        window.set_anchor(
            Edge::Top,
            false,
        );

        window.set_anchor(
            Edge::Bottom,
            true,
        );

        window.set_margin(
            Edge::Bottom,
            config.bar.margin_bottom,
        );
    } else {
        window.set_anchor(
            Edge::Bottom,
            false,
        );

        window.set_anchor(
            Edge::Top,
            true,
        );

        window.set_margin(
            Edge::Top,
            config.bar.margin_top,
        );
    }


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


    // IMPORTANT:
    // gtk4-layer-shell 0.7.x
    window.set_namespace(
        Some("raixbar"),
    );


    // ========================================================
    // CSS
    // ========================================================

    let css =
        CssProvider::new();

    let custom_css =
        config.css.custom.clone();

    let css_data =
        format!(
r#"
* {{
    font-family: "{font}",
        "Noto Sans",
        sans-serif;
}}

window {{
    background: transparent;
}}

.raix-bar {{
    background: {background};

    opacity: {opacity};

    border:
        {border_width}px solid
        {border_color};

    border-radius:
        {border_radius}px;

    padding-left:
        {bar_padding}px;

    padding-right:
        {bar_padding}px;

    min-height:
        {height}px;
}}

.bar-center {{
    min-height:
        {height}px;
}}

.left-zone {{
    padding-left: 0px;
    padding-right: 0px;
}}

.center-zone {{
    padding-left: 0px;
    padding-right: 0px;
}}

.right-zone {{
    padding-left: 0px;
    padding-right: 0px;
}}

.module {{
    background:
        {module_background};

    color:
        {text_color};

    padding-left:
        {module_padding}px;

    padding-right:
        {module_padding}px;

    min-height: 30px;

    border-radius:
        {module_radius}px;

    font-size:
        {font_size}px;

    font-weight:
        {font_weight};

    transition:
        120ms ease;
}}

.module:hover {{
    background:
        {module_hover};
}}

.logo-icon {{
    color:
        {logo_color};

    font-size:
        {logo_size}px;

    font-weight:
        800;

    padding-left: 10px;
    padding-right: 5px;

    border-radius: 9px;
}}

.logo-icon:hover {{
    background:
        {background_hover};
}}

.logo-text {{
    color:
        {logo_text_color};

    font-size:
        {font_size}px;

    font-weight:
        800;

    padding-left: 3px;
    padding-right: 10px;
}}

.workspace-container {{
    background:
        {workspace_background};

    border-radius:
        10px;

    padding: 2px;

    margin-left: 2px;
    margin-right: 2px;
}}

.workspace {{
    color:
        {workspace_inactive};

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

    transition:
        120ms ease;
}}

.workspace:hover {{
    color:
        {text_color};

    background:
        {workspace_hover};
}}

.workspace.active {{
    color:
        {workspace_active};

    background:
        {workspace_active_bg};
}}

.workspace.urgent {{
    color:
        {workspace_urgent};

    background:
        {workspace_urgent_bg};
}}

.module-cpu {{
    color:
        {cpu_color};
}}

.module-ram {{
    color:
        {ram_color};
}}

.module-network {{
    color:
        {network_color};
}}

.module-volume {{
    color:
        {volume_color};
}}

.module-battery {{
    color:
        {battery_color};
}}

.module-clock {{
    color:
        {clock_color};

    font-size:
        {clock_font_size}px;

    font-weight:
        {clock_font_weight};

    padding-left:
        {clock_padding}px;

    padding-right:
        {clock_padding}px;

    border-radius:
        {clock_radius}px;
}}

{custom_css}
"#,
            font =
                config.font.family,

            background =
                config.style.background,

            opacity =
                config.style.opacity,

            border_width =
                config.style.border_width,

            border_color =
                config.style.border_color,

            border_radius =
                config.style.border_radius,

            bar_padding =
                config.spacing.bar_padding,

            height =
                config.bar.height,

            module_background =
                config.style.module_background,

            module_hover =
                config.style.module_hover_background,

            text_color =
                config.style.text_color,

            module_padding =
                config.spacing.horizontal,

            module_radius =
                config.style.module_radius,

            font_size =
                config.font.size,

            font_weight =
                config.font.weight,

            logo_color =
                config.logo.icon_color,

            logo_size =
                config.logo.icon_size,

            background_hover =
                config.style.background_hover,

            logo_text_color =
                config.logo.text_color,

            workspace_background =
                config.style.workspace_background,

            workspace_inactive =
                config.workspaces.inactive_color,

            workspace_padding =
                config.workspaces.padding,

            workspace_radius =
                config.workspaces.radius,

            workspace_size =
                config.font.workspace_size,

            workspace_hover =
                config.workspaces.hover_background,

            workspace_active =
                config.workspaces.active_color,

            workspace_active_bg =
                config.workspaces.active_background,

            workspace_urgent =
                config.workspaces.urgent_color,

            workspace_urgent_bg =
                config.workspaces.urgent_background,

            cpu_color =
                config.cpu.color,

            ram_color =
                config.ram.color,

            network_color =
                config.network.color,

            volume_color =
                config.volume.color,

            battery_color =
                config.battery.color,

            clock_color =
                config.clock.color,

            clock_font_size =
                config.clock.font_size,

            clock_font_weight =
                config.clock.font_weight,

            clock_padding =
                config.clock.padding,

            clock_radius =
                config.clock.radius,
        );


    css.load_from_data(
        &css_data,
    );


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

    root.set_halign(
        Align::Fill,
    );

    root.set_valign(
        Align::Fill,
    );


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

    bar.set_hexpand(true);

    bar.set_halign(
        Align::Fill,
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
    //
    // width = 0
    // means full display width.
    //
    // width > 0
    // means fixed width.
    // ========================================================

    if config.bar.width > 0 {
        bar.set_width_request(
            config.bar.width,
        );

        bar.set_hexpand(false);

        match config.bar
            .alignment
            .to_lowercase()
            .as_str()
        {
            "left" => {
                bar.set_halign(
                    Align::Start,
                );
            }

            "right" => {
                bar.set_halign(
                    Align::End,
                );
            }

            _ => {
                bar.set_halign(
                    Align::Center,
                );
            }
        }
    } else {
        bar.set_hexpand(true);

        bar.set_halign(
            Align::Fill,
        );
    }


    // ========================================================
    // WAYBAR-STYLE 3 ZONES
    // ========================================================
    //
    //      START       CENTER       END
    //
    //      LEFT        CLOCK        RIGHT
    //
    // CenterBox keeps center independently centered.
    // ========================================================

    let center_box =
        CenterBox::new();

    center_box.add_css_class(
        "bar-center",
    );

    center_box.set_hexpand(
        true,
    );

    center_box.set_halign(
        Align::Fill,
    );

    center_box.set_valign(
        Align::Center,
    );


    // ========================================================
    // LEFT ZONE
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


    // ========================================================
    // CENTER ZONE
    // ========================================================

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


    // ========================================================
    // RIGHT ZONE
    // ========================================================

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
    // APP STATE
    // ========================================================

    let state =
        Rc::new(
            RefCell::new(
                AppState::new(),
            ),
        );


    // ========================================================
    // BUILD LEFT
    // ========================================================

    for module
        in &config.layout.left
    {
        add_module(
            &left,
            module,
            &config,
            state.clone(),
        );
    }


    // ========================================================
    // BUILD CENTER
    // ========================================================

    for module
        in &config.layout.center
    {
        add_module(
            &center,
            module,
            &config,
            state.clone(),
        );
    }


    // ========================================================
    // BUILD RIGHT
    // ========================================================

    for module
        in &config.layout.right
    {
        add_module(
            &right,
            module,
            &config,
            state.clone(),
        );
    }


    // ========================================================
    // CONNECT THE THREE ZONES
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


    // ========================================================
    // BAR
    // ========================================================

    bar.append(
        &center_box,
    );


    // ========================================================
    // ROOT
    // ========================================================

    root.append(
        &bar,
    );


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
    // MODULE TIMERS
    // ========================================================

    start_module_timers(
        state.clone(),
        &config,
    );


    // ========================================================
    // WORKSPACE TIMER
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


    // ========================================================
    // SHOW
    // ========================================================

    window.present();
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
            labels:
                HashMap::new(),

            workspace_box:
                GtkBox::new(
                    Orientation::Horizontal,
                    1,
                ),
        }
    }
}


// ============================================================
// MODULE BUILDER
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
                add_clock_module(
                    parent,
                    config,
                    state,
                );
            }
        }

        "separator" => {
            let separator =
                Label::new(
                    Some("│"),
                );

            separator
                .add_css_class(
                    "module",
                );

            separator
                .set_opacity(
                    0.3,
                );

            parent.append(
                &separator,
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
// ============================================================

fn add_logo(
    parent: &GtkBox,
    config: &Config,
) {
    if !config.logo.enabled {
        return;
    }

    if !config.logo.icon.is_empty() {
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

    if !config.logo.text.is_empty() {
        let label =
            Label::new(
                Some(
                    &config.logo.text,
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
        Label::new(
            Some("..."),
        );

    label.add_css_class(
        "module",
    );

    label.add_css_class(
        &format!(
            "module-{name}"
        ),
    );

    label.set_halign(
        Align::Center,
    );

    label.set_valign(
        Align::Center,
    );

    if !module.tooltip.is_empty() {
        label.set_tooltip_text(
            Some(
                &module.tooltip,
            ),
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

fn add_clock_module(
    parent: &GtkBox,
    config: &Config,
    state: Rc<RefCell<AppState>>,
) {
    let label =
        Label::new(
            Some("--:--"),
        );

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
        &config.clock.scroll_up,
        &config.clock.scroll_down,
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
// CLICK / SCROLL
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

        let left_cmd =
            left.to_string();

        let right_cmd =
            right.to_string();

        let middle_cmd =
            middle.to_string();

        gesture.connect_released(
            move |gesture, _, _, _| {
                let button =
                    gesture
                        .current_button();

                match button {
                    1 => {
                        run_command(
                            &left_cmd,
                        );
                    }

                    2 => {
                        run_command(
                            &middle_cmd,
                        );
                    }

                    3 => {
                        run_command(
                            &right_cmd,
                        );
                    }

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
                    run_command(
                        &up,
                    );
                } else if dy > 0.0 {
                    run_command(
                        &down,
                    );
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

fn run_command(
    command: &str,
) {
    if command.trim().is_empty() {
        return;
    }

    let command =
        command.trim();

    if let Err(error) =
        Command::new("sh")
            .arg("-c")
            .arg(command)
            .spawn()
    {
        eprintln!(
            "RaixBar command error `{command}`: {error}"
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
    // --------------------------------------------------------
    // CPU
    // --------------------------------------------------------

    if config.cpu.enabled {
        let state =
            state.clone();

        let cfg =
            config.cpu.clone();

        glib::timeout_add_local(
            Duration::from_millis(
                cfg.interval.max(100),
            ),
            move || {
                if let Some(label) =
                    state
                        .borrow()
                        .labels
                        .get("cpu")
                        .cloned()
                {
                    let value =
                        cpu_usage();

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


    // --------------------------------------------------------
    // RAM
    // --------------------------------------------------------

    if config.ram.enabled {
        let state =
            state.clone();

        let cfg =
            config.ram.clone();

        glib::timeout_add_local(
            Duration::from_millis(
                cfg.interval.max(100),
            ),
            move || {
                if let Some(label) =
                    state
                        .borrow()
                        .labels
                        .get("ram")
                        .cloned()
                {
                    let value =
                        ram_usage();

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


    // --------------------------------------------------------
    // NETWORK
    // --------------------------------------------------------

    if config.network.enabled {
        let state =
            state.clone();

        let cfg =
            config.network.clone();

        glib::timeout_add_local(
            Duration::from_millis(
                cfg.interval.max(100),
            ),
            move || {
                if let Some(label) =
                    state
                        .borrow()
                        .labels
                        .get("network")
                        .cloned()
                {
                    let value =
                        network_status();

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


    // --------------------------------------------------------
    // VOLUME
    // --------------------------------------------------------

    if config.volume.enabled {
        let state =
            state.clone();

        let cfg =
            config.volume.clone();

        glib::timeout_add_local(
            Duration::from_millis(
                cfg.interval.max(100),
            ),
            move || {
                if let Some(label) =
                    state
                        .borrow()
                        .labels
                        .get("volume")
                        .cloned()
                {
                    let value =
                        volume_status();

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


    // --------------------------------------------------------
    // BATTERY
    // --------------------------------------------------------

    if config.battery.enabled {
        let state =
            state.clone();

        let cfg =
            config.battery.clone();

        glib::timeout_add_local(
            Duration::from_millis(
                cfg.interval.max(500),
            ),
            move || {
                if let Some(label) =
                    state
                        .borrow()
                        .labels
                        .get("battery")
                        .cloned()
                {
                    let value =
                        battery_status();

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


    // --------------------------------------------------------
    // CLOCK
    // --------------------------------------------------------

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
// INITIAL VALUES
// ============================================================

fn update_all_modules(
    state: Rc<RefCell<AppState>>,
    config: &Config,
) {
    if let Some(label) =
        state
            .borrow()
            .labels
            .get("cpu")
            .cloned()
    {
        label.set_text(
            &format_module(
                &config.cpu.format,
                &config.cpu.icon,
                &cpu_usage(),
            ),
        );
    }


    if let Some(label) =
        state
            .borrow()
            .labels
            .get("ram")
            .cloned()
    {
        label.set_text(
            &format_module(
                &config.ram.format,
                &config.ram.icon,
                &ram_usage(),
            ),
        );
    }


    if let Some(label) =
        state
            .borrow()
            .labels
            .get("network")
            .cloned()
    {
        label.set_text(
            &format_module(
                &config.network.format,
                &config.network.icon,
                &network_status(),
            ),
        );
    }


    if let Some(label) =
        state
            .borrow()
            .labels
            .get("volume")
            .cloned()
    {
        label.set_text(
            &format_module(
                &config.volume.format,
                &config.volume.icon,
                &volume_status(),
            ),
        );
    }


    if let Some(label) =
        state
            .borrow()
            .labels
            .get("battery")
            .cloned()
    {
        label.set_text(
            &format_module(
                &config.battery.format,
                &config.battery.icon,
                &battery_status(),
            ),
        );
    }


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
    let output =
        Command::new("date")
            .arg(
                format!("+{}", format),
            )
            .output();

    match output {
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
            .find(
                |line| {
                    line.starts_with(
                        "cpu ",
                    )
                },
            )?;

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
        values
            .get(3)
            .copied()
            .unwrap_or(0)
        +
        values
            .get(4)
            .copied()
            .unwrap_or(0);

    let total =
        values
            .iter()
            .sum();

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
            Some(v) => v,
            None =>
                return "0".into(),
        };

    PREVIOUS_CPU.with(
        |previous| {
            let mut previous =
                previous
                    .borrow_mut();

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
                                /
                            total_delta as f64
                                *
                            100.0
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
            Ok(v) => v,
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
                        |v| {
                            v.parse().ok()
                        },
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
                        |v| {
                            v.parse().ok()
                        },
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
            Ok(v) => v,

            Err(_) =>
                return "offline".into(),
        };

    let mut wireless =
        None;

    let mut ethernet =
        None;

    for entry in entries.flatten() {
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
                entry
                    .path()
                    .join(
                        "operstate",
                    ),
            )
            .unwrap_or_default()
            .trim()
            .to_string();

        if state != "up" {
            continue;
        }

        if entry
            .path()
            .join("wireless")
            .exists()
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
    // --------------------------------------------------------
    // PipeWire
    // --------------------------------------------------------

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
                    if (0.0..=2.0)
                        .contains(&value)
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


    // --------------------------------------------------------
    // PulseAudio
    // --------------------------------------------------------

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
            Ok(v) => v,

            Err(_) =>
                return "--".into(),
        };

    for entry in entries.flatten() {
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
                    .join(
                        "capacity",
                    ),
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

fn update_workspaces(
    container: &GtkBox,
    config: &WorkspaceConfig,
) {
    while let Some(child) =
        container.first_child()
    {
        container.remove(
            &child,
        );
    }


    let workspaces =
        hyprland_workspaces();


    // --------------------------------------------------------
    // CONFIGURED WORKSPACES
    // --------------------------------------------------------

    if config.show_empty
        && !config.numbers.is_empty()
    {
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
                found
                    .get(id)
                    .cloned()
                    .unwrap_or(
                        WorkspaceInfo {
                            id: *id,
                            active: false,
                            urgent: false,
                        },
                    );

            add_workspace_button(
                container,
                workspace,
                config,
            );
        }

        return;
    }


    // --------------------------------------------------------
    // ONLY EXISTING WORKSPACES
    // --------------------------------------------------------

    if workspaces.is_empty() {
        let label =
            Label::new(
                Some("󰘧"),
            );

        label.add_css_class(
            "workspace",
        );

        container.append(
            &label,
        );

        return;
    }


    for workspace
        in workspaces
    {
        if !config.numbers.is_empty()
            && !config.numbers.contains(
                &workspace.id,
            )
        {
            continue;
        }

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
        config.format
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


    let click_command =
        config.left_click
            .replace(
                "{id}",
                &id.to_string(),
            );


    let right_command =
        config.right_click
            .replace(
                "{id}",
                &id.to_string(),
            );


    let middle_command =
        config.middle_click
            .replace(
                "{id}",
                &id.to_string(),
            );


    setup_clicks(
        &label,
        &click_command,
        &right_command,
        &middle_command,
        "",
        "",
    );


    // --------------------------------------------------------
    // Workspace scrolling
    // --------------------------------------------------------

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
    let active_id =
        get_active_workspace();


    let output =
        match Command::new(
            "hyprctl",
        )
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
        active_id,
    )
}


// ============================================================
// ACTIVE WORKSPACE
// ============================================================

fn get_active_workspace()
    -> Option<i32>
{
    let output =
        Command::new(
            "hyprctl",
        )
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
// WORKSPACE JSON PARSER
// ============================================================

fn parse_workspace_json(
    text: &str,
    active_id: Option<i32>,
) -> Vec<WorkspaceInfo> {
    let mut result =
        Vec::new();

    let mut remaining =
        text;


    while let Some(id_pos) =
        remaining.find("\"id\"")
    {
        remaining =
            &remaining[
                id_pos + 4..
            ];


        let colon =
            match remaining.find(':') {
                Some(v) => v,
                None => break,
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
                Ok(v) => v,
                Err(_) => continue,
            };


        let next_object =
            remaining.find(
                '}',
            );


        let object =
            match next_object {
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
                    |v| v == id,
                )
                .unwrap_or(false);


        if !result.iter().any(
            |workspace: &WorkspaceInfo| {
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
            if let Some(end) =
                next_object
            {
                &remaining[
                    end + 1..
                ]
            } else {
                ""
            };
    }


    result.sort_by_key(
        |workspace| workspace.id,
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
        format!(
            "\"{}\"",
            key,
        );


    let position =
        text.find(
            &needle,
        )?;


    let rest =
        &text[
            position + needle.len()..
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