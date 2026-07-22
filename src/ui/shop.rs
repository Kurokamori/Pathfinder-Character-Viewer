//! Shop tab: the compendium browser for buying or granting gear, plus the purse.

use crate::app::{App, CoinField, Message};
use crate::model::compendium::{Item, ItemKind};
use crate::theme::Palette;
use crate::ui::widgets::{self, caption};
use iced::widget::{button, column, container, row, text, text_input, Space};
use iced::{Alignment, Element, Length};

pub fn view(app: &App) -> Element<'_, Message> {
    let p = app.palette();
    column![
        coins_section(app, p),
        container(shop_panel(app, p)).width(Length::Fill).height(Length::Fill),
    ]
    .spacing(18)
    .height(Length::Fill)
    .into()
}

fn coins_section<'a>(app: &'a App, p: Palette) -> Element<'a, Message> {
    let c = &app.character.coins;
    let fields = row![
        widgets::number_field(p, "Platinum", c.pp as i32, 90.0, |v| Message::SetCoins(
            CoinField::Pp,
            v
        )),
        widgets::number_field(p, "Gold", c.gp as i32, 90.0, |v| Message::SetCoins(
            CoinField::Gp,
            v
        )),
        widgets::number_field(p, "Silver", c.sp as i32, 90.0, |v| Message::SetCoins(
            CoinField::Sp,
            v
        )),
        widgets::number_field(p, "Copper", c.cp as i32, 90.0, |v| Message::SetCoins(
            CoinField::Cp,
            v
        )),
        Space::with_width(Length::Fill),
        {
            let (pp, gp, sp, cp) = c.normalized();
            widgets::stat_tile(
                p,
                "Total",
                format!("{pp} / {gp} / {sp} / {cp}"),
                Some("pp / gp / sp / cp".to_string()),
            )
        },
    ]
    .spacing(12)
    .align_y(Alignment::End);
    widgets::section(p, "Coins", fields)
}

fn shop_panel<'a>(app: &'a App, p: Palette) -> Element<'a, Message> {
    let mut kind_row = row![].spacing(6);
    for kind in ItemKind::ALL {
        let active = app.shop_kind == kind;
        kind_row = kind_row.push(
            button(text(kind.label()).size(12))
                .padding([5, 10])
                .style(crate::theme::tab_button(p, active))
                .on_press(Message::ShopKind(kind)),
        );
    }

    let search = text_input("Search shop...", &app.shop_search)
        .on_input(Message::ShopSearch)
        .padding([8, 12])
        .style(crate::theme::input(p));

    let needle = app.shop_search.to_lowercase();
    let mut matches: Vec<&Item> = app
        .game
        .compendium
        .items
        .iter()
        .filter(|i| i.kind == app.shop_kind)
        .filter(|i| app.settings.allows(&i.source))
        .filter(|i| needle.is_empty() || i.name.to_lowercase().contains(&needle))
        .collect();
    matches.sort_by(|a, b| a.name.cmp(&b.name));

    let mut list = column![].spacing(6);
    for item in matches.iter().take(300) {
        list = list.push(shop_row(p, app, item));
    }
    if matches.len() > 300 {
        list = list.push(caption(p, format!("Showing first 300 of {}.", matches.len())));
    }

    widgets::section(
        p,
        "Shop",
        column![kind_row, search, widgets::browse_list(list)].spacing(10),
    )
}

fn shop_row<'a>(p: Palette, app: &App, item: &Item) -> Element<'a, Message> {
    let affordable = app.character.coins.as_gp() >= item.price;
    let selected = app.selected_item.as_deref() == Some(item.id.as_str());
    let subtitle = format!("{:.1} gp · {:.1} lb", item.price, item.weight);
    let buy_base = button(text("Buy").size(12))
        .padding([5, 12])
        .on_press(Message::ShopBuy(item.id.clone()));
    let buy = if affordable {
        buy_base.style(crate::theme::accent_button(p))
    } else {
        buy_base.style(crate::theme::subtle_button(p))
    };

    let add = button(text("Add").size(12))
        .padding([5, 12])
        .style(crate::theme::ghost_button(p))
        .on_press(Message::ShopAdd(item.id.clone()));

    let toggle = if selected {
        Message::ShopSelect(None)
    } else {
        Message::ShopSelect(Some(item.id.clone()))
    };
    let info = button(
        column![
            text(item.name.clone()).size(13).color(p.text),
            text(subtitle).size(11).color(p.text_dim),
        ]
        .spacing(2),
    )
    .padding([6, 10])
    .width(Length::Fill)
    .style(crate::theme::list_button(p, selected))
    .on_press(toggle);

    let top = row![info, add, buy].spacing(8).align_y(Alignment::Center);

    if selected && !item.description.is_empty() {
        column![top, description_box(p, &item.description)]
            .spacing(6)
            .into()
    } else {
        top.into()
    }
}

fn description_box<'a>(p: Palette, text_body: &str) -> Element<'a, Message> {
    container(text(text_body.to_string()).size(12).color(p.text_dim))
        .padding(12)
        .width(Length::Fill)
        .style(crate::theme::stat_box(p))
        .into()
}
