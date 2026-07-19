//! Small reusable building blocks shared across every tab.

use crate::app::{EditorTarget, Message};
use crate::theme::{self, Palette};
use iced::widget::{button, column, container, row, text, text_editor, text_input, Space};
use iced::{Alignment, Element, Length};

/// Parse a signed integer leniently; blank or partial input reads as zero.
pub fn parse_int(s: &str) -> i32 {
    let trimmed = s.trim();
    if trimmed.is_empty() || trimmed == "-" {
        0
    } else {
        trimmed.parse().unwrap_or(0)
    }
}

pub fn parse_u32(s: &str) -> u32 {
    s.trim().parse().unwrap_or(0)
}

pub fn parse_f64(s: &str) -> f64 {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        0.0
    } else {
        trimmed.parse().unwrap_or(0.0)
    }
}

/// A dim, small caption.
pub fn caption<'a>(p: Palette, content: impl text::IntoFragment<'a>) -> Element<'a, Message> {
    text(content).size(12).color(p.text_dim).into()
}

/// A section heading with an accent underline feel.
pub fn heading<'a>(p: Palette, content: impl text::IntoFragment<'a>) -> Element<'a, Message> {
    text(content).size(18).color(p.text).into()
}

/// A rounded card wrapping arbitrary content.
pub fn card<'a>(p: Palette, content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(content)
        .padding(18)
        .width(Length::Fill)
        .style(theme::card(p))
        .into()
}

/// A titled card section.
pub fn section<'a>(
    p: Palette,
    title: impl text::IntoFragment<'a>,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    card(
        p,
        column![heading(p, title), content.into()].spacing(14),
    )
}

/// A horizontal divider line.
pub fn divider(p: Palette) -> Element<'static, Message> {
    container(Space::with_height(1))
        .width(Length::Fill)
        .style(theme::divider(p))
        .into()
}

/// A rounded pill label.
pub fn pill<'a>(p: Palette, content: impl text::IntoFragment<'a>) -> Element<'a, Message> {
    container(text(content).size(12).color(p.text_dim))
        .padding([3, 10])
        .style(theme::tag_pill(p))
        .into()
}

/// A labeled text field laid out vertically.
pub fn labeled_input<'a>(
    p: Palette,
    label: &'a str,
    value: &str,
    on_change: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    column![
        text(label).size(12).color(p.text_dim),
        text_input("", value)
            .on_input(on_change)
            .padding([7, 10])
            .size(14)
            .style(theme::input(p)),
    ]
    .spacing(4)
    .width(Length::Fill)
    .into()
}

/// A labeled numeric field of fixed width.
pub fn number_field<'a>(
    p: Palette,
    label: &'a str,
    value: i32,
    width: f32,
    on_change: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    column![
        text(label).size(12).color(p.text_dim),
        text_input("", &value.to_string())
            .on_input(on_change)
            .padding([7, 10])
            .size(14)
            .width(Length::Fixed(width))
            .style(theme::input(p)),
    ]
    .spacing(4)
    .into()
}

/// A large stat tile: a label, a prominent value, and an optional sub-note.
pub fn stat_tile<'a>(
    p: Palette,
    label: impl text::IntoFragment<'a>,
    value: impl text::IntoFragment<'a>,
    sub: Option<String>,
) -> Element<'a, Message> {
    let mut inner = column![
        text(label).size(11).color(p.text_dim),
        text(value).size(24).color(p.text),
    ]
    .spacing(2)
    .align_x(Alignment::Center);
    if let Some(s) = sub {
        inner = inner.push(text(s).size(11).color(p.text_dim));
    }
    container(inner)
        .padding([12, 16])
        .style(theme::stat_box(p))
        .into()
}

/// A small +/- stepper around a value.
pub fn stepper<'a>(
    p: Palette,
    label: &'a str,
    value: impl text::IntoFragment<'a>,
    dec: Message,
    inc: Message,
) -> Element<'a, Message> {
    column![
        text(label).size(12).color(p.text_dim),
        row![
            button(text("-").size(16))
                .padding([2, 12])
                .style(theme::subtle_button(p))
                .on_press(dec),
            container(text(value).size(16).color(p.text))
                .padding([4, 12])
                .center_x(Length::Fixed(52.0)),
            button(text("+").size(16))
                .padding([2, 12])
                .style(theme::subtle_button(p))
                .on_press(inc),
        ]
        .spacing(6)
        .align_y(Alignment::Center),
    ]
    .spacing(4)
    .into()
}

/// An accent-filled primary button.
pub fn primary_button<'a>(
    p: Palette,
    label: impl text::IntoFragment<'a>,
    msg: Message,
) -> Element<'a, Message> {
    button(text(label).size(14))
        .padding([8, 16])
        .style(theme::accent_button(p))
        .on_press(msg)
        .into()
}

/// A bordered, low-emphasis button.
pub fn ghost_button<'a>(
    p: Palette,
    label: impl text::IntoFragment<'a>,
    msg: Message,
) -> Element<'a, Message> {
    button(text(label).size(14))
        .padding([8, 16])
        .style(theme::ghost_button(p))
        .on_press(msg)
        .into()
}

/// A browse-list row: a selectable title/subtitle plus an add/remove toggle.
pub fn browse_row<'a>(
    p: Palette,
    title: String,
    subtitle: String,
    owned: bool,
    active: bool,
    on_select: Message,
    on_toggle: Message,
) -> Element<'a, Message> {
    let toggle_label = if owned { "Remove" } else { "Add" };
    let toggle_base = button(text(toggle_label).size(12))
        .padding([6, 12])
        .on_press(on_toggle);
    let toggle = if owned {
        toggle_base.style(theme::danger_button(p))
    } else {
        toggle_base.style(theme::accent_button(p))
    };

    let main = button(
        column![
            text(title).size(14).color(p.text),
            text(subtitle).size(11).color(p.text_dim),
        ]
        .spacing(2),
    )
    .padding([8, 12])
    .width(Length::Fill)
    .style(theme::list_button(p, active))
    .on_press(on_select);

    row![main, toggle]
        .spacing(8)
        .align_y(Alignment::Center)
        .into()
}

/// A right-hand detail panel: title, a row of meta pills, and a body.
pub fn detail_panel<'a>(
    p: Palette,
    title: String,
    meta: Vec<String>,
    body: String,
) -> Element<'a, Message> {
    let mut meta_row = row![].spacing(6);
    for m in meta {
        if !m.is_empty() {
            meta_row = meta_row.push(pill(p, m));
        }
    }
    let content = column![
        text(title).size(20).color(p.text),
        meta_row,
        divider(p),
        text(if body.is_empty() {
            "No description available.".to_string()
        } else {
            body
        })
        .size(13)
        .color(p.text),
    ]
    .spacing(12);
    card(p, content)
}

/// Wrap a list column in a vertical scroll region with right padding so the
/// scrollbar never overlaps trailing buttons.
pub fn browse_list<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    iced::widget::scrollable(container(content.into()).padding(iced::Padding::ZERO.right(16.0)))
        .height(Length::Fill)
        .into()
}

/// An editable homebrew entry: name (+ optional level), remove, and a
/// multiline description that grows with its content.
pub fn custom_row<'a>(
    p: Palette,
    which: crate::app::CustomList,
    uid: u64,
    name: &str,
    level: i32,
    description: Option<&'a text_editor::Content>,
    with_level: bool,
) -> Element<'a, Message> {
    let name_input = text_input("Name", name)
        .on_input(move |v| Message::CustomSetName(which, uid, v))
        .padding([6, 10])
        .size(14)
        .style(theme::input(p));

    let mut top = row![container(name_input).width(Length::Fill)].spacing(8);
    if with_level {
        let level_input = text_input("", &level.to_string())
            .on_input(move |v| Message::CustomSetLevel(which, uid, v))
            .padding([6, 8])
            .size(13)
            .width(Length::Fixed(60.0))
            .style(theme::input(p));
        top = top.push(column![caption(p, "Lvl"), level_input].spacing(2));
    }
    top = top.push(
        button(text("Remove").size(12))
            .padding([6, 10])
            .style(theme::ghost_button(p))
            .on_press(Message::CustomRemove(which, uid)),
    );

    let target = EditorTarget::CustomDesc(which, uid);
    let desc = growing_editor(p, description, "Description (optional)", target);

    container(column![top.align_y(Alignment::Center), desc].spacing(8))
        .padding(12)
        .style(theme::plain_row(p))
        .into()
}

/// A multiline text editor that grows in height as its content grows. Bound to
/// an `App::editors` buffer via `target`; renders a disabled placeholder if the
/// buffer is somehow absent (it is created eagerly by `App::sync_editors`).
pub fn growing_editor<'a>(
    p: Palette,
    content: Option<&'a text_editor::Content>,
    placeholder: &'a str,
    target: EditorTarget,
) -> Element<'a, Message> {
    match content {
        Some(c) => text_editor(c)
            .placeholder(placeholder)
            .padding([6, 10])
            .style(theme::editor(p))
            .on_action(move |action| Message::EditorAction(target, action))
            .into(),
        None => info_box(p, String::new()),
    }
}

/// A muted, padded box for descriptive body text.
pub fn info_box<'a>(p: Palette, body: String) -> Element<'a, Message> {
    container(
        text(if body.is_empty() {
            "No description available.".to_string()
        } else {
            body
        })
        .size(12)
        .color(p.text_dim),
    )
    .padding(12)
    .width(Length::Fill)
    .style(theme::stat_box(p))
    .into()
}

/// An expandable card: a clickable header that reveals a description body.
pub fn expandable_card<'a>(
    p: Palette,
    expand: Message,
    expanded: bool,
    title: String,
    subtitle: String,
    description: String,
    action: Option<Element<'a, Message>>,
) -> Element<'a, Message> {
    let header_button = button(
        row![
            column![
                text(title).size(14).color(p.text),
                text(subtitle).size(11).color(p.text_dim),
            ]
            .spacing(2)
            .width(Length::Fill),
            text(if expanded { "Hide" } else { "Details" })
                .size(11)
                .color(p.text_dim),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .padding([8, 12])
    .width(Length::Fill)
    .style(theme::list_button(p, expanded))
    .on_press(expand);

    let mut head = row![header_button].spacing(8).align_y(Alignment::Center);
    if let Some(a) = action {
        head = head.push(a);
    }

    let mut body = column![head].spacing(8);
    if expanded {
        body = body.push(info_box(p, description));
    }
    body.into()
}

/// A centered empty-state message.
pub fn placeholder<'a>(p: Palette, message: &'a str) -> Element<'a, Message> {
    container(text(message).size(14).color(p.text_dim))
        .padding(40)
        .center_x(Length::Fill)
        .into()
}

/// A modifier badge such as `+3`, colored by sign.
pub fn mod_badge<'a>(p: Palette, value: i32) -> Element<'a, Message> {
    let color = if value > 0 {
        p.good
    } else if value < 0 {
        p.bad
    } else {
        p.text_dim
    };
    text(crate::rules::display::signed(value))
        .size(15)
        .color(color)
        .into()
}
