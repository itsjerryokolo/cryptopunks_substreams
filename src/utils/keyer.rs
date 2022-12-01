pub enum KeyType {
    Bidder,
    Owner,
    Punk,
}

pub fn generate_key(key: KeyType, val: &str) -> Option<String> {
    match key {
        KeyType::Bidder => Some(format!("Bidder: {}", val)),
        KeyType::Owner => Some(format!("Owner: {}", val)),
        KeyType::Punk => Some(format!("Punk: {}", val)),
    }
}
