//! Dark visual theme: palette, per-class accent, and reusable widget styles.

use iced::border::Radius;
use iced::widget::{button, checkbox, container, pick_list, text_input};
use iced::{Background, Border, Color, Shadow, Theme, Vector};

pub const RADIUS: f32 = 9.0;
pub const RADIUS_SM: f32 = 6.0;

fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgb8(r, g, b)
}

fn with_alpha(mut c: Color, a: f32) -> Color {
    c.a = a;
    c
}

/// A resolved color palette. Only the accent varies between classes.
#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub bg: Color,
    pub surface: Color,
    pub surface_alt: Color,
    pub raised: Color,
    pub border: Color,
    pub border_soft: Color,
    pub text: Color,
    pub text_dim: Color,
    pub accent: Color,
    pub good: Color,
    pub bad: Color,
    pub warn: Color,
}

impl Palette {
    pub fn new(accent: Color) -> Self {
        Palette {
            bg: rgb(0x14, 0x16, 0x1b),
            surface: rgb(0x1b, 0x1e, 0x25),
            surface_alt: rgb(0x22, 0x26, 0x2f),
            raised: rgb(0x2a, 0x2f, 0x3a),
            border: rgb(0x35, 0x3c, 0x49),
            border_soft: rgb(0x28, 0x2d, 0x38),
            text: rgb(0xe7, 0xe9, 0xee),
            text_dim: rgb(0x93, 0x9c, 0xad),
            accent,
            good: rgb(0x5a, 0xc4, 0x78),
            bad: rgb(0xe4, 0x64, 0x6e),
            warn: rgb(0xe0, 0xb6, 0x4d),
        }
    }
}

/// The signature accent color for a class tag.
pub fn accent_for(tag: &str) -> Color {
    match tag {
        "witch" => rgb(0xb2, 0x8d, 0xf7),
        "ninja" => rgb(0xe0, 0x5a, 0x63),
        _ => rgb(0x3f, 0xc4, 0xb4),
    }
}

/// The base Iced theme; custom widget styles supply the real look.
pub fn base_theme() -> Theme {
    Theme::custom(
        "PathfinderDark".to_string(),
        iced::theme::Palette {
            background: rgb(0x14, 0x16, 0x1b),
            text: rgb(0xe7, 0xe9, 0xee),
            primary: rgb(0xb2, 0x8d, 0xf7),
            success: rgb(0x5a, 0xc4, 0x78),
            danger: rgb(0xe4, 0x64, 0x6e),
        },
    )
}

fn border(color: Color, width: f32, radius: f32) -> Border {
    Border {
        color,
        width,
        radius: Radius::from(radius),
    }
}

// --- containers -------------------------------------------------------------

pub fn app_bg(p: Palette) -> impl Fn(&Theme) -> container::Style {
    move |_t| container::Style {
        background: Some(Background::Color(p.bg)),
        text_color: Some(p.text),
        ..container::Style::default()
    }
}

pub fn sidebar(p: Palette) -> impl Fn(&Theme) -> container::Style {
    move |_t| container::Style {
        background: Some(Background::Color(p.surface)),
        text_color: Some(p.text),
        border: border(p.border_soft, 1.0, 0.0),
        ..container::Style::default()
    }
}

pub fn card(p: Palette) -> impl Fn(&Theme) -> container::Style {
    move |_t| container::Style {
        background: Some(Background::Color(p.surface_alt)),
        text_color: Some(p.text),
        border: border(p.border_soft, 1.0, RADIUS),
        shadow: Shadow {
            color: with_alpha(Color::BLACK, 0.25),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 8.0,
        },
    }
}

pub fn panel(p: Palette) -> impl Fn(&Theme) -> container::Style {
    move |_t| container::Style {
        background: Some(Background::Color(p.surface)),
        text_color: Some(p.text),
        border: border(p.border_soft, 1.0, RADIUS),
        ..container::Style::default()
    }
}

pub fn stat_box(p: Palette) -> impl Fn(&Theme) -> container::Style {
    move |_t| container::Style {
        background: Some(Background::Color(p.raised)),
        text_color: Some(p.text),
        border: border(p.border, 1.0, RADIUS_SM),
        ..container::Style::default()
    }
}

pub fn accent_strip(p: Palette) -> impl Fn(&Theme) -> container::Style {
    move |_t| container::Style {
        background: Some(Background::Color(with_alpha(p.accent, 0.14))),
        text_color: Some(p.text),
        border: border(with_alpha(p.accent, 0.5), 1.0, RADIUS_SM),
        ..container::Style::default()
    }
}

pub fn tag_pill(p: Palette) -> impl Fn(&Theme) -> container::Style {
    move |_t| container::Style {
        background: Some(Background::Color(p.raised)),
        text_color: Some(p.text_dim),
        border: border(p.border, 1.0, 20.0),
        ..container::Style::default()
    }
}

pub fn selected_row(p: Palette) -> impl Fn(&Theme) -> container::Style {
    move |_t| container::Style {
        background: Some(Background::Color(with_alpha(p.accent, 0.12))),
        text_color: Some(p.text),
        border: border(with_alpha(p.accent, 0.45), 1.0, RADIUS_SM),
        ..container::Style::default()
    }
}

pub fn plain_row(p: Palette) -> impl Fn(&Theme) -> container::Style {
    move |_t| container::Style {
        background: Some(Background::Color(p.surface_alt)),
        text_color: Some(p.text),
        border: border(p.border_soft, 1.0, RADIUS_SM),
        ..container::Style::default()
    }
}

pub fn divider(p: Palette) -> impl Fn(&Theme) -> container::Style {
    move |_t| container::Style {
        background: Some(Background::Color(p.border_soft)),
        ..container::Style::default()
    }
}

// --- buttons ----------------------------------------------------------------

pub fn accent_button(p: Palette) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_t, status| {
        let base = p.accent;
        let bg = match status {
            button::Status::Hovered => with_alpha(base, 0.9),
            button::Status::Pressed => with_alpha(base, 0.75),
            button::Status::Disabled => with_alpha(base, 0.3),
            button::Status::Active => base,
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: rgb(0x14, 0x16, 0x1b),
            border: border(Color::TRANSPARENT, 0.0, RADIUS_SM),
            shadow: Shadow::default(),
        }
    }
}

pub fn ghost_button(p: Palette) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_t, status| {
        let bg = match status {
            button::Status::Hovered => p.raised,
            button::Status::Pressed => p.surface_alt,
            _ => Color::TRANSPARENT,
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: p.text,
            border: border(p.border, 1.0, RADIUS_SM),
            shadow: Shadow::default(),
        }
    }
}

pub fn subtle_button(p: Palette) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_t, status| {
        let bg = match status {
            button::Status::Hovered => p.raised,
            button::Status::Pressed => p.surface,
            _ => p.surface_alt,
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: p.text,
            border: border(p.border_soft, 1.0, RADIUS_SM),
            shadow: Shadow::default(),
        }
    }
}

pub fn danger_button(p: Palette) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_t, status| {
        let bg = match status {
            button::Status::Hovered => with_alpha(p.bad, 0.9),
            button::Status::Pressed => with_alpha(p.bad, 0.7),
            button::Status::Disabled => with_alpha(p.bad, 0.3),
            button::Status::Active => with_alpha(p.bad, 0.85),
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: rgb(0x14, 0x16, 0x1b),
            border: border(Color::TRANSPARENT, 0.0, RADIUS_SM),
            shadow: Shadow::default(),
        }
    }
}

pub fn tab_button(p: Palette, active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_t, status| {
        let bg = if active {
            with_alpha(p.accent, 0.16)
        } else {
            match status {
                button::Status::Hovered => p.raised,
                _ => Color::TRANSPARENT,
            }
        };
        let border_color = if active {
            with_alpha(p.accent, 0.55)
        } else {
            Color::TRANSPARENT
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: if active { p.text } else { p.text_dim },
            border: border(border_color, 1.0, RADIUS_SM),
            shadow: Shadow::default(),
        }
    }
}

pub fn list_button(p: Palette, selected: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_t, status| {
        let bg = if selected {
            with_alpha(p.accent, 0.14)
        } else {
            match status {
                button::Status::Hovered => p.raised,
                button::Status::Pressed => p.surface,
                _ => p.surface_alt,
            }
        };
        let border_color = if selected {
            with_alpha(p.accent, 0.5)
        } else {
            p.border_soft
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: p.text,
            border: border(border_color, 1.0, RADIUS_SM),
            shadow: Shadow::default(),
        }
    }
}

// --- inputs -----------------------------------------------------------------

pub fn input(p: Palette) -> impl Fn(&Theme, text_input::Status) -> text_input::Style {
    move |_t, status| {
        let border_color = match status {
            text_input::Status::Focused => p.accent,
            text_input::Status::Hovered => p.border,
            _ => p.border_soft,
        };
        text_input::Style {
            background: Background::Color(p.raised),
            border: border(border_color, 1.0, RADIUS_SM),
            icon: p.text_dim,
            placeholder: p.text_dim,
            value: p.text,
            selection: with_alpha(p.accent, 0.35),
        }
    }
}

pub fn check(p: Palette) -> impl Fn(&Theme, checkbox::Status) -> checkbox::Style {
    move |_t, status| {
        let checked = match status {
            checkbox::Status::Active { is_checked }
            | checkbox::Status::Hovered { is_checked }
            | checkbox::Status::Disabled { is_checked } => is_checked,
        };
        checkbox::Style {
            background: Background::Color(if checked { p.accent } else { p.raised }),
            icon_color: rgb(0x14, 0x16, 0x1b),
            border: border(if checked { p.accent } else { p.border }, 1.0, RADIUS_SM),
            text_color: Some(p.text),
        }
    }
}

pub fn dropdown(p: Palette) -> impl Fn(&Theme, pick_list::Status) -> pick_list::Style {
    move |_t, status| {
        let border_color = match status {
            pick_list::Status::Hovered | pick_list::Status::Opened => p.accent,
            pick_list::Status::Active => p.border_soft,
        };
        pick_list::Style {
            text_color: p.text,
            placeholder_color: p.text_dim,
            handle_color: p.text_dim,
            background: Background::Color(p.raised),
            border: border(border_color, 1.0, RADIUS_SM),
        }
    }
}
