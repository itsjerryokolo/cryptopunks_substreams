mod abi;
mod events;
mod pb;
mod rpc;
mod state;
#[cfg(test)]
mod tests;
mod utils;

use anyhow::Error;
use utils::helper::append_0x;

use pb::cryptopunks as punks;
use substreams::prelude::*;
use substreams::store::StoreSet;
use substreams::Hex;

use substreams_ethereum::pb::eth::v2 as eth;
use utils::constants::WRAPPEDPUNKS_CONTRACT;
use utils::keyer::{
    generate_key, KeyType::Assignee as Assignee_Key, KeyType::Buyer as Buyer_Key,
    KeyType::Contract as Contract_Key, KeyType::Day as Day_Key, KeyType::Owner as Owner_Key,
    KeyType::Punk as Punk_Key, KeyType::Seller as Seller_Key, KeyType::UserProxy as Proxy_Key,
};
use utils::math::decimal_from_str;

substreams_ethereum::init!();

#[substreams::handlers::map]
fn map_transfers(blk: eth::Block) -> Result<punks::Transfers, Error> {
    events::map_transfers(blk)
}

#[substreams::handlers::map]
fn map_assigns(blk: eth::Block) -> Result<punks::Assigns, Error> {
    events::map_assigns(blk)
}

#[substreams::handlers::map]
fn map_sales(blk: eth::Block) -> Result<punks::Sales, Error> {
    events::map_sales(blk)
}

#[substreams::handlers::map]
fn map_bids(blk: eth::Block) -> Result<punks::Bids, Error> {
    events::map_bids(blk)
}

#[substreams::handlers::map]
fn map_asks(blk: eth::Block) -> Result<punks::Asks, Error> {
    events::map_asks(blk)
}

#[substreams::handlers::map]
fn map_user_proxies(blk: eth::Block) -> Result<punks::UserProxies, Error> {
    events::map_user_proxies(blk)
}

#[substreams::handlers::map]
fn map_metadata(blk: eth::Block) -> Result<punks::Metadatas, Error> {
    events::map_metadata(blk)
}

#[substreams::handlers::map]
fn map_wrapped_transfers(blk: eth::Block) -> Result<punks::Transfers, Error> {
    events::map_wrapped_transfers(blk)
}

// Store writes follow Firehose ordinals, never block-end lookups.
#[substreams::handlers::store]
pub fn store_assigns(input: punks::Assigns, store: StoreSetProto<punks::Assign>) {
    for assign in input.assigns {
        store.set(
            assign.ordinal,
            generate_key(Punk_Key, &assign.token_id.to_string()),
            &assign,
        );
        store.set(
            assign.ordinal,
            generate_key(Assignee_Key, &assign.to),
            &assign,
        );
    }
}

#[substreams::handlers::store]
pub fn punk_state(
    input: punks::Transfers,
    proxies: StoreGetProto<punks::UserProxy>,
    store: StoreSetProto<punks::Transfer>,
) {
    for mut transfer in input.transfers {
        if transfer.to == append_0x(&Hex(WRAPPEDPUNKS_CONTRACT).to_string()) {
            if let Some(proxy) =
                proxies.get_at(transfer.ordinal, generate_key(Proxy_Key, &transfer.from))
            {
                transfer.from = proxy.user;
            }
        }
        store.set(
            transfer.ordinal,
            generate_key(Punk_Key, &transfer.token_id.to_string()),
            &transfer,
        );
    }
}

#[substreams::handlers::store]
pub fn store_bid_events(input: punks::Bids, store: StoreSetProto<punks::Bid>) {
    for bid in input.bids {
        store.set(
            bid.ordinal,
            generate_key(Punk_Key, &bid.token_id.to_string()),
            &bid,
        );
    }
}

#[substreams::handlers::map]
pub fn map_resolved_sales(
    input: punks::Sales,
    bids: StoreGetProto<punks::Bid>,
    resets: StoreGetProto<punks::Bid>,
) -> Result<punks::Sales, Error> {
    state::resolve_sales(input, |sale| {
        active_bid_before(sale.token_id, sale.ordinal, &bids, &resets)
    })
}

fn bid_reset_key(token_id: u64, bidder: &str) -> String {
    format!("Punk: {token_id}:Bidder: {bidder}")
}

fn active_bid_before(
    token_id: u64,
    ordinal: u64,
    bids: &StoreGetProto<punks::Bid>,
    resets: &StoreGetProto<punks::Bid>,
) -> Option<punks::Bid> {
    let before = ordinal.saturating_sub(1);
    let raw = bids.get_at(before, generate_key(Punk_Key, &token_id.to_string()));
    let reset = raw
        .as_ref()
        .and_then(|bid| resets.get_at(before, bid_reset_key(token_id, &bid.from)));
    state::active_bid(raw, reset)
}

#[substreams::handlers::store]
pub fn store_bid_resets(
    sales: punks::Sales,
    transfers: punks::Transfers,
    store: StoreSetProto<punks::Bid>,
) {
    for reset in state::bid_resets(sales, transfers) {
        store.set(
            reset.ordinal,
            bid_reset_key(reset.token_id, &reset.from),
            &reset,
        );
    }
}

#[substreams::handlers::map]
pub fn map_bid_changes(
    input: punks::Bids,
    sales: punks::Sales,
    transfers: punks::Transfers,
    bids: StoreGetProto<punks::Bid>,
    resets: StoreGetProto<punks::Bid>,
) -> Result<punks::Bids, Error> {
    Ok(punks::Bids {
        bids: state::bid_updates(input, sales, transfers, |reset| {
            active_bid_before(reset.token_id, reset.ordinal, &bids, &resets)
        }),
    })
}

#[substreams::handlers::store]
pub fn bids_state(input: punks::Bids, store: StoreSetProto<punks::Bid>) {
    for bid in input.bids {
        // Canonical current bid per punk. The old address-only alias retained stale
        // replaced bids and conflated one bidder's bids on different punks.
        store.set(
            bid.ordinal,
            generate_key(Punk_Key, &bid.token_id.to_string()),
            &bid,
        );
    }
}

#[substreams::handlers::store]
pub fn asks_state(input: punks::Asks, sales: punks::Sales, store: StoreSetProto<punks::Ask>) {
    for ask in state::ask_updates(input, sales) {
        store.set(
            ask.ordinal,
            generate_key(Punk_Key, &ask.token_id.to_string()),
            &ask,
        );
        store.set(ask.ordinal, generate_key(Owner_Key, &ask.from), &ask);
    }
}

#[substreams::handlers::store]
pub fn store_volume(input: punks::Sales, store: StoreAddBigDecimal) {
    for sale in input.sales {
        let amount = decimal_from_str(&sale.amount).expect("validated sale amount");
        for key in [
            generate_key(Contract_Key, ""),
            generate_key(Day_Key, &(sale.timestamp / 86400).to_string()),
            generate_key(Punk_Key, &sale.token_id.to_string()),
            generate_key(Buyer_Key, &sale.to),
            generate_key(Seller_Key, &sale.from),
        ] {
            store.add(sale.ordinal, key, &amount);
        }
    }
}

#[substreams::handlers::store]
pub fn store_sales(input: punks::Sales, store: StoreSetProto<punks::Sale>) {
    for sale in input.sales {
        store.set(
            sale.ordinal,
            generate_key(Punk_Key, &sale.token_id.to_string()),
            &sale,
        );
        store.set(sale.ordinal, generate_key(Buyer_Key, &sale.to), &sale);
        store.set(sale.ordinal, generate_key(Seller_Key, &sale.from), &sale);
    }
}

#[substreams::handlers::store]
pub fn store_user_proxies(input: punks::UserProxies, store: StoreSetProto<punks::UserProxy>) {
    for proxy in input.user_proxies {
        store.set(
            proxy.ordinal,
            generate_key(Proxy_Key, &proxy.proxy_address),
            &proxy,
        );
    }
}

#[substreams::handlers::store]
pub fn contract_metadata(input: punks::Assigns, store: StoreSetProto<punks::Contract>) {
    for assign in input.assigns {
        if let Some(contract) = assign.contract {
            store.set(assign.ordinal, generate_key(Contract_Key, ""), &contract);
        }
    }
}

#[substreams::handlers::store]
pub fn store_metadata(i: punks::Metadatas, o: StoreSetProto<punks::Metadata>) {
    for metadata in i.metadatas {
        o.set(0, generate_key(Punk_Key, &metadata.token_id), &metadata)
    }
}
