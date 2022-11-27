mod abi;
mod pb;
use abi::cryptopunks;
use hex_literal::hex;
use pb::cryptopunks as punks;
use substreams::prelude::*;
use substreams::{log, store::StoreAddInt64, Hex};
use substreams_ethereum::Event;
use substreams_ethereum::{pb::eth::v2 as eth, NULL_ADDRESS};

// Cryptopunks Contract
const TRACKED_CONTRACT: [u8; 20] = hex!("b47e3cd837dDF8e4c57F05d70Ab865de6e193BBB");

substreams_ethereum::init!();

/// Extracts transfers events from the contract
#[substreams::handlers::map]
fn map_transfers(blk: eth::Block) -> Result<punks::Transfers, substreams::errors::Error> {
    Ok(punks::Transfers {
        transfers: blk
            .events::<abi::cryptopunks::events::PunkTransfer>(&[&TRACKED_CONTRACT])
            .map(|(transfer, log)| {
                substreams::log::info!("NFT Transfer seen");

                punks::Transfer {
                    from: transfer.from,
                    to: transfer.to,
                    block_number: blk.number,
                    timestamp: blk.timestamp_seconds(),
                    trx_hash: log.receipt.transaction.hash.clone(),
                    token_id: transfer.punk_index.to_u64(),
                    ordinal: log.block_index() as u64,
                }
            })
            .collect(),
    })
}
#[substreams::handlers::map]
fn map_assigns(blk: eth::Block) -> Result<punks::Assigns, substreams::errors::Error> {
    let mut assigns: Vec<punks::Assign> = Vec::new();
    for log in blk.logs() {
        if let Some(assign_event) = abi::cryptopunks::events::Assign::match_and_decode(log) {
            substreams::log::info!("Assign Event Found");
            assigns.push(punks::Assign {
                to: assign_event.to,
                token_id: assign_event.punk_index.to_u64(),
                trx_hash: log.receipt.transaction.hash.clone(),
                block_number: blk.number,
                timestamp: blk.timestamp_seconds(),
                ordinal: log.block_index() as u64,
            });
        }
    }
    Ok(punks::Assigns { assigns })
}
