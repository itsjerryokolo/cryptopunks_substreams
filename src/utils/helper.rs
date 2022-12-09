pub fn get_traits(i: &str) -> String {
    i.split(|first: char| first == ',' || first.is_numeric())
        .skip(2)
        .map(|first| format!("{},", first))
        .collect()
}
pub fn get_gender(i: &str) -> String {
    i.split(" ").take(1).collect()
}
