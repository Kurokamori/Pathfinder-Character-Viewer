//! Inventory tab: shop browser, owned gear, coins, and encumbrance.

use crate::app::{App, CoinField, EditorTarget, Message};
use crate::model::character::InventoryItem;
use crate::model::compendium::{Item, ItemKind};
use crate::rules::derived::DerivedStats;
use crate::theme::Palette;
use crate::ui::widgets::{self, caption};
use iced::widget::{button, column, container, row, text, text_editor, text_input, Space};
use iced::{Alignment, Element, Length};

pub fn view(app: &App) -> Element<'_, Message> {
    let p = app.palette();
    let d = app.derived();
    column![
        coins_section(app, p),
        row![
            container(shop_panel(app, p)).width(Length::FillPortion(1)),
            container(inventory_panel(app, p, &d)).width(Length::FillPortion(1)),
        ]
        .spacing(18)
        .height(Length::Fill),
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
        widgets::stat_tile(p, "Total", format!("{:.1} gp", c.as_gp()), None),
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
        column![
            kind_row,
            search,
            widgets::browse_list(list),
        ]
        .spacing(10),
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

fn inventory_panel<'a>(app: &'a App, p: Palette, d: &DerivedStats) -> Element<'a, Message> {
    let total_weight: f64 = app
        .character
        .inventory
        .iter()
        .map(|i| i.weight * i.quantity as f64)
        .sum();

    let capacity = row![
        widgets::stat_tile(p, "Weight", format!("{total_weight:.1} lb"), None),
        widgets::stat_tile(p, "Light", format!("{} lb", d.carry_light), None),
        widgets::stat_tile(p, "Medium", format!("{} lb", d.carry_medium), None),
        widgets::stat_tile(p, "Heavy", format!("{} lb", d.carry_heavy), None),
    ]
    .spacing(8);

    let mut list = column![].spacing(6);
    if app.character.inventory.is_empty() {
        list = list.push(caption(p, "No items yet. Buy from the shop or add a custom item."));
    }
    for item in &app.character.inventory {
        let expanded = app.expanded_item == Some(item.uid);
        let template_desc = item
            .source_id
            .as_ref()
            .and_then(|id| app.game.item(id))
            .map(|template| template.description.clone())
            .filter(|d| !d.is_empty());
        let notes = app.editors.get(&EditorTarget::InventoryNotes(item.uid));
        list = list.push(inventory_row(p, item, expanded, template_desc, notes));
    }

    let header = row![
        Space::with_width(Length::Fill),
        widgets::ghost_button(p, "+ Custom Item", Message::InvAdd),
    ];

    widgets::section(
        p,
        "Carried Gear",
        column![
            capacity,
            header,
            widgets::browse_list(list),
        ]
        .spacing(12),
    )
}

fn inventory_row<'a>(
    p: Palette,
    item: &InventoryItem,
    expanded: bool,
    template_desc: Option<String>,
    notes: Option<&'a text_editor::Content>,
) -> Element<'a, Message> {
    let equipable = matches!(
        item.kind,
        ItemKind::Armor | ItemKind::Shield | ItemKind::Weapon
    );
    let uid = item.uid;

    let name = text_input("Item name", &item.name)
        .on_input(move |v| Message::InvSetName(uid, v))
        .padding([6, 10])
        .size(14)
        .style(crate::theme::input(p));

    let small = |value: String, on: fn(u64, String) -> Message| {
        text_input("", &value)
            .on_input(move |v| on(uid, v))
            .padding([4, 8])
            .size(12)
            .width(Length::Fixed(64.0))
            .style(crate::theme::input(p))
    };

    let qty = small(item.quantity.to_string(), Message::InvSetQty);
    let weight = small(format!("{:.1}", item.weight), Message::InvSetWeight);
    let price = small(format!("{:.1}", item.price), Message::InvSetPrice);

    let equip: Element<Message> = if equipable {
        button(text(if item.equipped { "Equipped" } else { "Equip" }).size(11))
            .padding([5, 10])
            .style(crate::theme::tab_button(p, item.equipped))
            .on_press(Message::InvToggleEquip(uid))
            .into()
    } else {
        Space::with_width(0).into()
    };

    let details = button(text(if expanded { "Hide" } else { "Details" }).size(11))
        .padding([5, 10])
        .style(crate::theme::tab_button(p, expanded))
        .on_press(Message::InvExpand(uid));

    let remove = button(text("Remove").size(11))
        .padding([5, 10])
        .style(crate::theme::ghost_button(p))
        .on_press(Message::InvRemove(uid));

    let fields = row![
        labeled(p, "Qty", qty),
        labeled(p, "Weight", weight),
        labeled(p, "Price", price),
        Space::with_width(Length::Fill),
        equip,
        details,
        remove,
    ]
    .spacing(8)
    .align_y(Alignment::End);

    let mut inner = column![name, fields].spacing(8);
    if expanded {
        if let Some(desc) = template_desc {
            inner = inner.push(description_box(p, &desc));
        }
        inner = inner.push(
            column![
                caption(p, "Notes"),
                widgets::growing_editor(
                    p,
                    notes,
                    "Add a description or notes...",
                    EditorTarget::InventoryNotes(uid),
                ),
            ]
            .spacing(4),
        );
    }

    container(inner)
        .padding([10, 12])
        .style(crate::theme::plain_row(p))
        .into()
}

fn labeled<'a>(
    p: Palette,
    label: &'a str,
    input: iced::widget::TextInput<'a, Message>,
) -> Element<'a, Message> {
    column![caption(p, label), input].spacing(2).into()
}
