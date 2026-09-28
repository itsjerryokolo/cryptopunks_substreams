mod abi;
mod db;
mod events;
mod pb;
mod rpc;
mod state;
#[cfg(test)]
mod tests;
mod utils;

use anyhow::Error;
use substreams::store;
use utils::helper::append_0x;

use substreams_entity_change::pb::entity::EntityChanges;

use pb::cryptopunks as punks;
use substreams::prelude::*;
use substreams::store::StoreSet;
use substreams::Hex;

use substreams_ethereum::pb::eth::v2 as eth;
use utils::constants::WRAPPEDPUNKS_CONTRACT;
use utils::keyer::{
    generate_key, KeyType::Assignee as Assignee_Key, KeyType::Bidder as Bidder_Key,
    KeyType::Buyer as Buyer_Key, KeyType::Contract as Contract_Key, KeyType::Day as Day_Key,
    KeyType::Owner as Owner_Key, KeyType::Punk as Punk_Key, KeyType::Seller as Seller_Key,
    KeyType::UserProxy as Proxy_Key,
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
) -> Result<punks::Sales, Error> {
    state::resolve_sales(input, |sale| {
        bids.get_at(
            sale.ordinal.saturating_sub(1),
            generate_key(Punk_Key, &sale.token_id.to_string()),
        )
    })
}

#[substreams::handlers::store]
pub fn bids_state(
    input: punks::Bids,
    sales: punks::Sales,
    bids: StoreGetProto<punks::Bid>,
    store: StoreSetProto<punks::Bid>,
) {
    for bid in state::bid_updates(input, sales, |sale| {
        bids.get_at(
            sale.ordinal.saturating_sub(1),
            generate_key(Punk_Key, &sale.token_id.to_string()),
        )
    }) {
        store.set(bid.ordinal, generate_key(Bidder_Key, &bid.from), &bid);
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

//Entity Changes
#[substreams::handlers::map]
pub fn map_metadata_entities(
    metadata_deltas: store::Deltas<DeltaProto<punks::Metadata>>,
) -> Result<EntityChanges, Error> {
    let mut entity_changes: EntityChanges = Default::default();

    db::store_metadata_entity_change(&mut entity_changes, metadata_deltas)?;

    Ok(entity_changes)
}

#[substreams::handlers::map]
pub fn map_contract_entities(
    metadata_deltas: store::Deltas<DeltaProto<punks::Contract>>,
) -> Result<EntityChanges, Error> {
    let mut entity_changes: EntityChanges = Default::default();

    db::store_contract_entity_change(&mut entity_changes, metadata_deltas)?;

    Ok(entity_changes)
}

#[substreams::handlers::map]
pub fn map_transfer_entities(
    transfer_deltas: store::Deltas<DeltaProto<punks::Transfer>>,
) -> Result<EntityChanges, Error> {
    let mut entity_changes: EntityChanges = Default::default();

    db::create_transfer_entity_change(&mut entity_changes, transfer_deltas)?;

    Ok(entity_changes)
}

#[substreams::handlers::map]
pub fn map_assign_entities(
    assign_deltas: store::Deltas<DeltaProto<punks::Assign>>,
) -> Result<EntityChanges, Error> {
    let mut entity_changes: EntityChanges = Default::default();

    db::create_assign_entity_change(&mut entity_changes, assign_deltas)?;

    Ok(entity_changes)
}

#[substreams::handlers::map]
pub fn map_ask_entities(
    ask_deltas: store::Deltas<DeltaProto<punks::Ask>>,
) -> Result<EntityChanges, Error> {
    let mut entity_changes: EntityChanges = Default::default();

    db::create_ask_entity_change(&mut entity_changes, ask_deltas)?;

    Ok(entity_changes)
}

#[substreams::handlers::map]
pub fn map_bid_entities(
    bid_deltas: store::Deltas<DeltaProto<punks::Bid>>,
) -> Result<EntityChanges, Error> {
    let mut entity_changes: EntityChanges = Default::default();

    db::create_bid_entity_change(&mut entity_changes, bid_deltas)?;

    Ok(entity_changes)
}

#[substreams::handlers::map]
pub fn map_sale_entities(
    sale_deltas: store::Deltas<DeltaProto<punks::Sale>>,
) -> Result<EntityChanges, Error> {
    let mut entity_changes: EntityChanges = Default::default();

    db::create_sale_entity_change(&mut entity_changes, sale_deltas)?;

    Ok(entity_changes)
}

#[substreams::handlers::map]
pub fn graph_out(
    metadata_entities: EntityChanges,
    contract_entities: EntityChanges,
    transfer_entities: EntityChanges,
    assign_entities: EntityChanges,
    ask_entities: EntityChanges,
    bid_entities: EntityChanges,
    sale_entities: EntityChanges,
) -> Result<EntityChanges, Error> {
    let mut entity_changes = Vec::new();
    for output in [
        metadata_entities,
        contract_entities,
        transfer_entities,
        assign_entities,
        ask_entities,
        bid_entities,
        sale_entities,
    ] {
        entity_changes.extend(output.entity_changes);
    }
    entity_changes.sort_by_key(|change| change.ordinal);
    Ok(EntityChanges { entity_changes })
}
