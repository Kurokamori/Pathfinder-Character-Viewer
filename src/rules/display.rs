//! Small formatting helpers shared across the UI.

/// Format a modifier with an explicit sign (e.g. `+3`, `-1`, `+0`).
pub fn signed(value: i32) -> String {
    if value >= 0 {
        format!("+{value}")
    } else {
        value.to_string()
    }
}

/// Expand a Foundry school abbreviation to its full name.
pub fn school_name(abbr: &str) -> &'static str {
    match abbr {
        "abj" => "Abjuration",
        "con" => "Conjuration",
        "div" => "Divination",
        "enc" => "Enchantment",
        "evo" => "Evocation",
        "ill" => "Illusion",
        "nec" => "Necromancy",
        "trs" => "Transmutation",
        "uni" => "Universal",
        _ => "Unknown",
    }
}

/// Ordinal label for a spell level (0 -> "Cantrips", 1 -> "1st", ...).
pub fn spell_level_label(level: usize) -> String {
    match level {
        0 => "Cantrips".to_string(),
        1 => "1st".to_string(),
        2 => "2nd".to_string(),
        3 => "3rd".to_string(),
        n => format!("{n}th"),
    }
}

/// Compact spell component string, e.g. "V, S, M".
pub fn components_string(v: bool, s: bool, m: bool, f: bool, df: bool) -> String {
    let mut parts: Vec<&str> = Vec::new();
    if v {
        parts.push("V");
    }
    if s {
        parts.push("S");
    }
    if m {
        parts.push("M");
    }
    if f {
        parts.push("F");
    }
    if df {
        parts.push("DF");
    }
    parts.join(", ")
}
