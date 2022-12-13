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
        KeyType::Contract => format!("Contract: {}b47e3cd837dDF8e4c57F05d70Ab865de6e193BBB", val),
    }
}

pub fn generate_id(tx_hash: &str, log_index: &str, kind: &str) -> String {
    format!("{}-{}-{}", tx_hash, log_index, kind)
}
