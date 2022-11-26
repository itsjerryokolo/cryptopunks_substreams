mod abi;
mod pb;
use hex_literal::hex;
use pb::cryptopunks;
use substreams::prelude::*;
use substreams::{log, store::StoreAddInt64, Hex};
use substreams_ethereum::{pb::eth::v2 as eth, NULL_ADDRESS};

// Cryptopunks Contract
const TRACKED_CONTRACT: [u8; 20] = hex!("b47e3cd837dDF8e4c57F05d70Ab865de6e193BBB");

substreams_ethereum::init!();

/// Extracts transfers events from the contract
#[substreams::handlers::map]
fn map_transfers(blk: eth::Block) -> Result<cryptopunks::Transfers, substreams::errors::Error> {
    Ok(cryptopunks::Transfers {
        transfers: blk
            .events::<abi::cryptopunks::events::PunkTransfer>(&[&TRACKED_CONTRACT])
            .map(|(transfer, log)| {
                substreams::log::info!("NFT Transfer seen");

                cryptopunks::Transfer {
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

// /// Store the total balance of NFT tokens for the specific TRACKED_CONTRACT by holder
// #[substreams::handlers::store]
// fn store_transfers(transfers: erc721::Transfers, s: StoreAddInt64) {
//     log::info!("NFT holders state builder");
//     for transfer in transfers.transfers {
//         if transfer.from != NULL_ADDRESS {
//             log::info!("Found a transfer out {}", Hex(&transfer.trx_hash));
//             s.add(transfer.ordinal, generate_key(&transfer.from), -1);
//         }

//         if transfer.to != NULL_ADDRESS {
//             log::info!("Found a transfer in {}", Hex(&transfer.trx_hash));
//             s.add(transfer.ordinal, generate_key(&transfer.to), 1);
//         }
//     }
// }

// fn generate_key(holder: &Vec<u8>) -> String {
//     return format!("total:{}:{}", Hex(holder), Hex(TRACKED_CONTRACT));
// }
