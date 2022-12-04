pub enum KeyType {
    Bidder,
    Punk,
    Assignee,
    UserProxy,
    Owner,
}

pub fn generate_key(key: KeyType, val: &str) -> String {
    match key {
        KeyType::Bidder => format!("Bidder: {}", val),
        KeyType::Punk => format!("Punk: {}", val),
        KeyType::Assignee => format!("Assignee: {}", val),
        KeyType::UserProxy => format!("UserProxy: {}", val),
        KeyType::Owner => format!("Owner: {}", val),
    }
}
