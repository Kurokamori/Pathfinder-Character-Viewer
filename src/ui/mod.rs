//! View layer: the persistent sidebar, tab shell, and per-tab screens.

pub mod combat;
pub mod combat_ref;
pub mod features;
pub mod gallery;
pub mod general;
pub mod images;
pub mod inventory;
pub mod notes;
pub mod roster;
pub mod settings;
pub mod sidebar;
pub mod skills;
pub mod spells;
pub mod widgets;

use crate::app::{App, Message};
use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Alignment, Element, Length};

/// A tab in the character sheet. Class modules choose which appear.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    General,
    Combat,
    CombatRef,
    Skills,
    Spells,
    Hexes,
    Ki,
    Familiar,
    Inventory,
    Features,
    Gallery,
    Notes,
}

impl Tab {
    pub fn label(self) -> &'static str {
        match self {
            Tab::General => "General",
            Tab::Combat => "Combat",
            Tab::CombatRef => "Combat Ref",
            Tab::Skills => "Skills",
            Tab::Spells => "Spells",
            Tab::Hexes => "Hexes",
            Tab::Ki => "Ki & Tricks",
            Tab::Familiar => "Familiar",
            Tab::Inventory => "Inventory",
            Tab::Features => "Features",
            Tab::Gallery => "Gallery",
            Tab::Notes => "Notes",
        }
    }
}

/// Root view: routes between the roster picker and the character sheet.
pub fn view(app: &App) -> Element<'_, Message> {
    let body = if app.on_roster {
        roster::view(app)
    } else if app.show_settings {
        settings::view(app)
    } else {
        sheet_view(app)
    };

    container(body)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(crate::theme::app_bg(app.palette()))
        .into()
}

fn sheet_view(app: &App) -> Element<'_, Message> {
    let sidebar = sidebar::view(app);

    let tabs = tab_bar(app);
    let content = container(tab_content(app))
        .padding(24)
        .width(Length::Fill)
        .height(Length::Fill);

    let main = column![tabs, content].spacing(0).width(Length::Fill).height(Length::Fill);

    row![sidebar, main]
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn tab_bar(app: &App) -> Element<'_, Message> {
    let p = app.palette();
    let mut tabs = row![].spacing(6).align_y(Alignment::Center);
    for tab in &app.tabs {
        let active = *tab == app.active_tab;
        tabs = tabs.push(
            button(text(tab.label()).size(14))
                .padding([8, 16])
                .style(crate::theme::tab_button(p, active))
                .on_press(Message::SelectTab(*tab)),
        );
    }

    let toolbar = row![
        tabs,
        Space::with_width(Length::Fill),
        button(text("Books").size(13))
            .padding([8, 14])
            .style(crate::theme::subtle_button(p))
            .on_press(Message::OpenSettings(true)),
        button(text("Save").size(13))
            .padding([8, 14])
            .style(crate::theme::subtle_button(p))
            .on_press(Message::SaveNow),
        button(text("Export").size(13))
            .padding([8, 14])
            .style(crate::theme::subtle_button(p))
            .on_press(Message::ExportCharacter),
        button(text("Roster").size(13))
            .padding([8, 14])
            .style(crate::theme::subtle_button(p))
            .on_press(Message::ShowRoster),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .width(Length::Fill);

    container(toolbar)
        .padding([12, 20])
        .width(Length::Fill)
        .style(crate::theme::sidebar(p))
        .into()
}

fn tab_content(app: &App) -> Element<'_, Message> {
    match app.active_tab {
        Tab::General => scroll(general::view(app)),
        Tab::Combat => scroll(combat::view(app)),
        Tab::CombatRef => scroll(combat_ref::view(app)),
        Tab::Skills => scroll(skills::view(app)),
        Tab::Notes => scroll(notes::view(app)),
        Tab::Spells => spells::view(app),
        Tab::Hexes => crate::classes::witch::hexes_view(app),
        Tab::Ki => crate::classes::ninja::ki_view(app),
        Tab::Familiar => crate::classes::witch::familiar_view(app),
        Tab::Inventory => inventory::view(app),
        Tab::Features => features::view(app),
        Tab::Gallery => gallery::view(app),
    }
}

/// Wrap a shrink-height tab in a vertical scroll region.
fn scroll(content: Element<'_, Message>) -> Element<'_, Message> {
    scrollable(content).height(Length::Fill).width(Length::Fill).into()
}
