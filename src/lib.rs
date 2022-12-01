mod abi;
mod pb;
mod utils;
use abi::cryptopunks::events as event;
use hex_literal::hex;
use pb::cryptopunks as punks;
use substreams::prelude::*;
use substreams::store::StoreSet;
use substreams::{log, Hex};
use substreams_ethereum::NULL_ADDRESS;
use substreams_ethereum::{pb::eth::v2 as eth, Event};
use utils::keyer::{generate_key, KeyType};
use utils::math::{convert_and_divide, decimal_from_str};

// Cryptopunks Contract
const TRACKED_CONTRACT: [u8; 20] = hex!("b47e3cd837dDF8e4c57F05d70Ab865de6e193BBB");

substreams_ethereum::init!();

/// Extracts transfers events from the contract
#[substreams::handlers::map]
fn map_transfers(blk: eth::Block) -> Result<punks::Transfers, substreams::errors::Error> {
    Ok(punks::Transfers {
        transfers: blk
            .events::<event::PunkTransfer>(&[&TRACKED_CONTRACT])
            .map(|(transfer, log)| {
                log::info!("NFT Transfer seen");

                punks::Transfer {
                    from: Hex(transfer.from).to_string(),
                    to: Hex(transfer.to).to_string(),
                    block_number: blk.number,
                    timestamp: blk.timestamp_seconds(),
                    trx_hash: Hex(&log.receipt.transaction.hash).to_string(),
                    token_id: transfer.punk_index.to_u64(),
                    ordinal: log.block_index() as u64,
                }
            })
            .collect(),
    })
}

// Extract Assign Events from the Block with matching event signature
#[substreams::handlers::map]
fn map_assigns(blk: eth::Block) -> Result<punks::Assigns, substreams::errors::Error> {
    let mut assigns: Vec<punks::Assign> = Vec::new();
    for log in blk.logs() {
        if let Some(assign_event) = event::Assign::match_and_decode(log) {
            log::info!("Assign Event Found");

            //Create assign
            assigns.push(punks::Assign {
                to: Hex(&assign_event.to).to_string(),
                token_id: assign_event.punk_index.to_u64(),
                trx_hash: Hex(&log.receipt.transaction.hash).to_string(),
                block_number: blk.number,
                timestamp: blk.timestamp_seconds(),
                ordinal: log.block_index() as u64,
            });
        }
    }
    Ok(punks::Assigns { assigns })
}

#[substreams::handlers::map]
fn map_sales(blk: eth::Block) -> Result<punks::Sales, substreams::errors::Error> {
    let mut sales: Vec<punks::Sale> = Vec::new();
    for log in blk.logs() {
        if let Some(sale_event) = event::PunkBought::match_and_decode(log) {
            log::info!("Sale Event Found");

            sales.push(punks::Sale {
                from: Hex(&sale_event.from_address).to_string(),
                to: Hex(&sale_event.to_address).to_string(),
                token_id: sale_event.punk_index.to_u64(),
                amount: convert_and_divide(sale_event.value.to_string().as_str())
                    .unwrap()
                    .to_string(),
                trx_hash: Hex(&log.receipt.transaction.hash).to_string(),
                block_number: blk.number,
                timestamp: blk.timestamp_seconds(),
                ordinal: log.block_index() as u64,
            });
        }
    }
    Ok(punks::Sales { sales })
}

#[substreams::handlers::map]
fn map_bids(blk: eth::Block) -> Result<punks::Bids, substreams::errors::Error> {
    let mut bids: Vec<punks::Bid> = vec![];
    for log in blk.logs() {
        if let Some(bid_event) = event::PunkBidEntered::match_and_decode(log) {
            log::info!("Sale Event Found");

            bids.push(punks::Bid {
                from: Hex(&bid_event.from_address).to_string(),
                to: Hex(&bid_event.from_address).to_string(),
                token_id: bid_event.punk_index.to_u64(),
                amount: convert_and_divide(bid_event.value.to_string().as_str())
                    .unwrap()
                    .to_string(),
                trx_hash: Hex(&log.receipt.transaction.hash).to_string(),
                block_number: blk.number,
                timestamp: blk.timestamp_seconds(),
                ordinal: log.block_index() as u64,
            });
        }
    }
    Ok(punks::Bids { bids })
}

#[substreams::handlers::store]
pub fn store_assigns(assigns: punks::Assigns, output: StoreSetProto<punks::Assign>) {
    for assign in assigns.assigns {
        output.set(
            0,
            generate_key(KeyType::Owner, &assign.to).unwrap(),
            &assign,
        );
    }
}

#[substreams::handlers::store]
pub fn punk_state(transfers: punks::Transfers, output: StoreSetProto<punks::Transfer>) {
    for transfer in transfers.transfers {
        let token_id = transfer.token_id as i64;
        if transfer.from == Hex(NULL_ADDRESS).to_string()
            || transfer.to == Hex(NULL_ADDRESS).to_string()
        {
            continue;
        };

        output.set(
            0,
            generate_key(KeyType::Punk, &token_id.to_string().as_str()).unwrap(),
            &transfer,
        );

        output.set(
            0,
            generate_key(KeyType::Owner, &transfer.to).unwrap(),
            &transfer,
        );
    }
}

#[substreams::handlers::store]
pub fn store_bids(bids: punks::Bids, output: StoreSetProto<punks::Bid>) {
    for bidder in bids.bids {
        output.set(
            0,
            generate_key(KeyType::Bidder, &bidder.to).unwrap(),
            &bidder,
        );
    }
}

#[substreams::handlers::store]
pub fn store_all_punks(assigns: punks::Assigns, output: StoreAppend<String>) {
    for assign in assigns.assigns {
        let token_id = assign.token_id as i64;
        output.append(
            0,
            generate_key(KeyType::Punk, &token_id.to_string().as_str()).unwrap(),
            token_id.to_string(),
        );
    }
}

#[substreams::handlers::store]
pub fn store_total_volume(s: punks::Sales, output: StoreAddBigDecimal) {
    for vol in s.sales {
        let val = decimal_from_str(vol.amount.as_str()).unwrap();
        output.add(0, "0xb47e3cd837dDF8e4c57F05d70Ab865de6e193BBB", val);
    }
}

#[substreams::handlers::store]
pub fn store_punk_sales(s: punks::Sales, output: StoreSetProto<punks::Sale>) {
    for sale in s.sales {
        let token_id = sale.token_id as i64;
        let ordinal = sale.ordinal;
        output.set(
            ordinal,
            generate_key(KeyType::Punk, &token_id.to_string().as_str()).unwrap(),
            &sale,
        );
    }
}

#[substreams::handlers::store]
pub fn store_punk_volume(s: punks::Sales, output: StoreAddBigDecimal) {
    for sale in s.sales {
        let token_id = sale.token_id as i64;
        let val = decimal_from_str(sale.amount.as_str()).unwrap();

        output.add(
            0,
            generate_key(KeyType::Punk, &token_id.to_string().as_str()).unwrap(),
            val,
        );
    }
}
