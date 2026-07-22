//! Inventory tab: an exhaustive view of carried gear — worn equipment slots,
//! armor and shields, consumables by type, and the full editable item list.

use crate::app::{App, EditorTarget, Message};
use crate::model::character::{EquipSlot, InventoryItem};
use crate::model::compendium::ItemKind;
use crate::rules::derived::DerivedStats;
use crate::theme::Palette;
use crate::ui::widgets::{self, caption};
use iced::widget::{button, column, container, row, scrollable, text, text_editor, text_input, Space};
use iced::{Alignment, Element, Length};

pub fn view(app: &App) -> Element<'_, Message> {
    let p = app.palette();
    let d = app.derived();

    let body = column![
        capacity_section(app, p, &d),
        equipment_section(app, p),
        row![
            container(kind_section(app, p, "Armor", ItemKind::Armor))
                .width(Length::FillPortion(1)),
            container(kind_section(app, p, "Shields", ItemKind::Shield))
                .width(Length::FillPortion(1)),
        ]
        .spacing(18),
        row![
            container(consumable_section(app, p, "Potions", &["potion", "elixir", "oil"]))
                .width(Length::FillPortion(1)),
            container(consumable_section(app, p, "Scrolls", &["scroll"]))
                .width(Length::FillPortion(1)),
            container(consumable_section(app, p, "Wands", &["wand", "staff", "rod"]))
                .width(Length::FillPortion(1)),
        ]
        .spacing(18),
        carried_section(app, p),
    ]
    .spacing(18);

    scrollable(container(body).padding(iced::Padding::ZERO.right(12.0)))
        .height(Length::Fill)
        .width(Length::Fill)
        .into()
}

fn capacity_section<'a>(app: &'a App, p: Palette, d: &DerivedStats) -> Element<'a, Message> {
    let total_weight: f64 = app
        .character
        .inventory
        .iter()
        .map(|i| i.weight * i.quantity as f64)
        .sum();

    let tiles = row![
        widgets::stat_tile(p, "Weight", format!("{total_weight:.1} lb"), None),
        widgets::stat_tile(p, "Light", format!("{} lb", d.carry_light), None),
        widgets::stat_tile(p, "Medium", format!("{} lb", d.carry_medium), None),
        widgets::stat_tile(p, "Heavy", format!("{} lb", d.carry_heavy), None),
        Space::with_width(Length::Fill),
        {
            let (pp, gp, sp, cp) = app.character.coins.normalized();
            widgets::stat_tile(
                p,
                "Wealth",
                format!("{pp} / {gp} / {sp} / {cp}"),
                Some("pp / gp / sp / cp".to_string()),
            )
        },
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    widgets::section(p, "Capacity", tiles)
}

fn equipment_section<'a>(app: &'a App, p: Palette) -> Element<'a, Message> {
    let mut slots = column![].spacing(6);
    for slot in EquipSlot::ALL {
        slots = slots.push(slot_row(app, p, slot));
    }
    widgets::section(
        p,
        "Worn Equipment",
        column![
            caption(p, "Assign one wondrous item per body slot. Armor and shields are worn from their own sections below."),
            slots,
        ]
        .spacing(10),
    )
}

fn slot_row<'a>(app: &'a App, p: Palette, slot: EquipSlot) -> Element<'a, Message> {
    let worn = app.character.inventory.iter().find(|i| i.slot == Some(slot));
    let picker_open = app.slot_picker == Some(slot);

    let label = column![
        text(slot.label()).size(14).color(p.text),
        caption(p, slot.hint()),
    ]
    .spacing(2)
    .width(Length::Fixed(150.0));

    let middle: Element<Message> = match worn {
        Some(item) => text(item.name.clone()).size(14).color(p.text).into(),
        None => text("— empty —").size(13).color(p.text_dim).into(),
    };

    let mut actions = row![].spacing(6).align_y(Alignment::Center);
    actions = actions.push(
        button(text(if worn.is_some() { "Change" } else { "Equip" }).size(11))
            .padding([5, 10])
            .style(crate::theme::tab_button(p, picker_open))
            .on_press(Message::InvSlotPicker(Some(slot))),
    );
    if worn.is_some() {
        actions = actions.push(
            button(text("Unequip").size(11))
                .padding([5, 10])
                .style(crate::theme::ghost_button(p))
                .on_press(Message::InvClearSlot(slot)),
        );
    }

    let head = row![
        label,
        container(middle).width(Length::Fill),
        actions,
    ]
    .spacing(10)
    .align_y(Alignment::Center);

    let mut inner = column![head].spacing(8);
    if picker_open {
        inner = inner.push(slot_picker(app, p, slot));
    }

    container(inner)
        .padding([10, 12])
        .style(crate::theme::plain_row(p))
        .into()
}

fn slot_picker<'a>(app: &'a App, p: Palette, slot: EquipSlot) -> Element<'a, Message> {
    let mut list = column![].spacing(4);
    let mut count = 0;
    for item in &app.character.inventory {
        if !slot_eligible(item) {
            continue;
        }
        count += 1;
        let assigned_elsewhere = item.slot.is_some() && item.slot != Some(slot);
        let subtitle = if assigned_elsewhere {
            format!("currently: {}", item.slot.map(|s| s.label()).unwrap_or(""))
        } else {
            format!("{:.1} lb", item.weight)
        };
        let uid = item.uid;
        list = list.push(
            button(
                column![
                    text(item.name.clone()).size(13).color(p.text),
                    text(subtitle).size(11).color(p.text_dim),
                ]
                .spacing(2),
            )
            .padding([6, 10])
            .width(Length::Fill)
            .style(crate::theme::list_button(p, false))
            .on_press(Message::InvAssignSlot(uid, slot)),
        );
    }

    if count == 0 {
        return caption(p, "No wearable items in your gear. Add or buy an item first.");
    }

    container(scrollable(container(list).padding(iced::Padding::ZERO.right(12.0))))
        .max_height(220.0)
        .padding(8)
        .style(crate::theme::stat_box(p))
        .into()
}

/// Whether an item can be assigned to a worn wondrous slot. Armor, shields, and
/// weapons are worn through their own equip toggles, not the slot grid.
fn slot_eligible(item: &InventoryItem) -> bool {
    !matches!(
        item.kind,
        ItemKind::Armor | ItemKind::Shield | ItemKind::Weapon
    )
}

fn kind_section<'a>(
    app: &'a App,
    p: Palette,
    title: &'a str,
    kind: ItemKind,
) -> Element<'a, Message> {
    let mut list = column![].spacing(6);
    let mut any = false;
    for item in app.character.inventory.iter().filter(|i| i.kind == kind) {
        any = true;
        list = list.push(worn_row(p, item));
    }
    if !any {
        list = list.push(caption(p, "None owned. Buy or add one from the Shop."));
    }
    widgets::section(p, title, list)
}

fn worn_row<'a>(p: Palette, item: &InventoryItem) -> Element<'a, Message> {
    let uid = item.uid;
    let meta = format!("+{} AC · {:.1} lb", item.ac_bonus, item.weight);
    let info = column![
        text(item.name.clone()).size(14).color(p.text),
        text(meta).size(11).color(p.text_dim),
    ]
    .spacing(2)
    .width(Length::Fill);

    let equip = button(text(if item.equipped { "Equipped" } else { "Equip" }).size(11))
        .padding([5, 10])
        .style(crate::theme::tab_button(p, item.equipped))
        .on_press(Message::InvToggleEquip(uid));

    let remove = button(text("Remove").size(11))
        .padding([5, 10])
        .style(crate::theme::ghost_button(p))
        .on_press(Message::InvRemove(uid));

    container(
        row![info, equip, remove]
            .spacing(8)
            .align_y(Alignment::Center),
    )
    .padding([8, 12])
    .style(crate::theme::plain_row(p))
    .into()
}

fn consumable_section<'a>(
    app: &'a App,
    p: Palette,
    title: &'a str,
    keywords: &'a [&'a str],
) -> Element<'a, Message> {
    let mut list = column![].spacing(6);
    let mut any = false;
    for item in &app.character.inventory {
        let name = item.name.to_lowercase();
        if !keywords.iter().any(|k| name.contains(k)) {
            continue;
        }
        any = true;
        list = list.push(consumable_row(p, item));
    }
    if !any {
        list = list.push(caption(p, "None carried."));
    }
    widgets::section(p, title, list)
}

fn consumable_row<'a>(p: Palette, item: &InventoryItem) -> Element<'a, Message> {
    let uid = item.uid;
    let dec = Message::InvSetQty(uid, item.quantity.saturating_sub(1).to_string());
    let inc = Message::InvSetQty(uid, (item.quantity + 1).to_string());

    let stepper = row![
        button(text("−").size(14))
            .padding([1, 9])
            .style(crate::theme::subtle_button(p))
            .on_press(dec),
        container(text(item.quantity.to_string()).size(13).color(p.text))
            .center_x(Length::Fixed(30.0)),
        button(text("+").size(14))
            .padding([1, 9])
            .style(crate::theme::subtle_button(p))
            .on_press(inc),
    ]
    .spacing(4)
    .align_y(Alignment::Center);

    let remove = button(text("×").size(14))
        .padding([1, 9])
        .style(crate::theme::ghost_button(p))
        .on_press(Message::InvRemove(uid));

    container(
        row![
            text(item.name.clone()).size(13).color(p.text).width(Length::Fill),
            stepper,
            remove,
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .padding([7, 12])
    .style(crate::theme::plain_row(p))
    .into()
}

fn carried_section<'a>(app: &'a App, p: Palette) -> Element<'a, Message> {
    let mut list = column![].spacing(6);
    if app.character.inventory.is_empty() {
        list = list.push(caption(p, "No items yet. Buy from the Shop or add a custom item."));
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

    widgets::section(p, "All Carried Gear", column![header, list].spacing(12))
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

    let mut header = row![container(name).width(Length::Fill)].spacing(8);
    if let Some(slot) = item.slot {
        header = header.push(widgets::pill(p, format!("Worn: {}", slot.label())));
    }

    let mut inner = column![header.align_y(Alignment::Center), fields].spacing(8);
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

fn description_box<'a>(p: Palette, text_body: &str) -> Element<'a, Message> {
    container(text(text_body.to_string()).size(12).color(p.text_dim))
        .padding(12)
        .width(Length::Fill)
        .style(crate::theme::stat_box(p))
        .into()
}

fn labeled<'a>(
    p: Palette,
    label: &'a str,
    input: iced::widget::TextInput<'a, Message>,
) -> Element<'a, Message> {
    column![caption(p, label), input].spacing(2).into()
}
