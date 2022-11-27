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

// #[substreams::handlers::store]
// pub fn store_assigns(assigns: punks::Assigns, output: StoreSet<) {
//     for assign in assigns.assigns {
//         output.set(
//             assign.ordinal,
//         );
//     }
// }
