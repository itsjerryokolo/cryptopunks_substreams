pub enum KeyType {
    Bidder,
    Punk,
    Assignee,
    UserProxy,
    Owner,
    Day,
    Buyer,
    Seller,
    Contract,
}

pub fn generate_key(key: KeyType, val: &str) -> String {
    match key {
        KeyType::Bidder => format!("Bidder: {}", val),
        KeyType::Punk => format!("Punk: {}", val),
        KeyType::Assignee => format!("Assignee: {}", val),
        KeyType::UserProxy => format!("UserProxy: {}", val),
        KeyType::Owner => format!("Owner: {}", val),
        KeyType::Day => format!("Day ID: {}", val),
        KeyType::Buyer => format!("Buyer: {}", val),
        KeyType::Seller => format!("Seller: {}", val),
        KeyType::Contract => "Contract: 0xb47e3cd837ddf8e4c57f05d70ab865de6e193bbb".to_string(),
    }
}

pub fn generate_id(tx_hash: &str, log_index: &str, kind: &str) -> String {
    format!("{}-{}-{}", tx_hash, log_index, kind)
}
