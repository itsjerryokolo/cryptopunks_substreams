pub fn get_traits(i: &str) -> String {
    // The first comma-separated field is the base type (e.g. "Female 2").
    // Digits in accessory names such as "3D Glasses" must remain intact.
    i.split(',')
        .skip(1)
        .map(str::trim)
        .filter(|trait_name| !trait_name.is_empty())
        .collect::<Vec<_>>()
        .join(",")
}
pub fn get_type(i: &str) -> String {
    i.split(',')
        .next()
        .unwrap_or_default()
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_string()
}

pub fn append_0x(i: &str) -> String {
    format!("0x{}", i.trim_start_matches("0x").to_ascii_lowercase())
}
