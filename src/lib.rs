mod abi;
mod pb;
use abi::cryptopunks::events as event;
use hex_literal::hex;
use pb::cryptopunks as punks;
use substreams::prelude::*;
use substreams::store::StoreSet;
use substreams::{log, Hex};
use substreams_ethereum::{pb::eth::v2 as eth, Event};

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
                amount: sale_event.value.to_string(),
                trx_hash: Hex(&log.receipt.transaction.hash).to_string(),
                block_number: blk.number,
                timestamp: blk.timestamp_seconds(),
                ordinal: log.block_index() as u64,
            });
        }
    }
    Ok(punks::Sales { sales })
}

#[substreams::handlers::store]
pub fn store_assigns(assigns: punks::Assigns, output: StoreSetInt64) {
    for assign in assigns.assigns {
        let token_id = assign.token_id as i64;
        output.set(0, &format!("Owner:{}", &assign.to), &token_id);
    }
}

#[substreams::handlers::store]
pub fn store_all_punks(assigns: punks::Assigns, output: StoreSetInt64) {
    for assign in assigns.assigns {
        let token_id = assign.token_id as i64;
        output.set(0, &format!("Punk:{}", &token_id), &token_id);
    }
}

#[substreams::handlers::store]
pub fn store_punk_sales(s: punks::Sales, output: StoreSetProto<punks::Sale>) {
    for sale in s.sales {
        let token_id = sale.token_id as i64;
        let ordinal = sale.ordinal;
        output.set(ordinal, &format!("Punk: {}", &token_id), &sale);
    }
}
