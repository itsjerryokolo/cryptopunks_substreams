pub enum KeyType {
    Bidder,
    Punk,
    Assignee,
    UserProxy,
    Owner,
    Day,
    Buyer,
    Seller,
}

pub fn generate_key(key: KeyType, val: &str) -> String {
    match key {
        KeyType::Bidder => format!("Bidder: 0x{}", val),
        KeyType::Punk => format!("Punk: {}", val),
        KeyType::Assignee => format!("Assignee: 0x{}", val),
        KeyType::UserProxy => format!("UserProxy: 0x{}", val),
        KeyType::Owner => format!("Owner: {}", val),
        KeyType::Day => format!("Day ID: {}", val),
        KeyType::Buyer => format!("Buyer: 0x{}", val),
        KeyType::Seller => format!("Seller: 0x{}", val),
    }
}
