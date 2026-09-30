use crate::{
    abi::{cryptopunks::events as cryptopunks_events, wrappedpunks::events as wrappedpunks_events},
    pb::cryptopunks as punks,
    rpc::{get_contract_data, get_punk_metadata},
    utils::{
        constants::{CRYPTOPUNKS_CONTRACT, WRAPPEDPUNKS_CONTRACT},
        helper::{append_0x, get_traits, get_type},
        math::convert_and_divide,
    },
};
use anyhow::Error;
use substreams::Hex;
use substreams_ethereum::{
    pb::eth::v2::{self as eth, Block},
    Event, NULL_ADDRESS,
};

// Extracts transfers events from the contract
pub fn map_transfers(blk: eth::Block) -> Result<punks::Transfers, Error> {
    Ok(punks::Transfers {
        transfers: blk
            .events::<cryptopunks_events::PunkTransfer>(&[&CRYPTOPUNKS_CONTRACT])
            .map(|(event, log)| punks::Transfer {
                from: append_0x(&Hex(&event.from).to_string()),
                to: append_0x(&Hex(&event.to).to_string()),
                wrapped: (event.to == WRAPPEDPUNKS_CONTRACT).to_string(),
                token_id: event.punk_index.to_u64(),
                trx_hash: append_0x(&Hex(&log.receipt.transaction.hash).to_string()),
                block_number: blk.number,
                timestamp: blk.timestamp_seconds(),
                block_hash: append_0x(&Hex(&blk.hash).to_string()),
                ordinal: log.ordinal(),
                log_index: log.block_index(),
            })
            .collect(),
    })
}

// Extract Assign Events from the Block with matching event signature
pub fn map_assigns(blk: eth::Block) -> Result<punks::Assigns, Error> {
    extract_assigns(&blk, get_contract_data)
}

pub(crate) fn extract_assigns(
    blk: &eth::Block,
    mut contract_data: impl FnMut() -> Result<punks::Contract, Error>,
) -> Result<punks::Assigns, Error> {
    let mut assigns = Vec::new();
    for (event, log) in blk.events::<cryptopunks_events::Assign>(&[&CRYPTOPUNKS_CONTRACT]) {
        let contract = if blk.number == 3_919_682 && assigns.is_empty() {
            Some(contract_data()?)
        } else {
            None
        };
        assigns.push(punks::Assign {
            to: append_0x(&Hex(&event.to).to_string()),
            token_id: event.punk_index.to_u64(),
            contract,
            trx_hash: append_0x(&Hex(&log.receipt.transaction.hash).to_string()),
            block_number: blk.number,
            timestamp: blk.timestamp_seconds(),
            block_hash: append_0x(&Hex(&blk.hash).to_string()),
            ordinal: log.ordinal(),
            log_index: log.block_index(),
        });
    }
    Ok(punks::Assigns { assigns })
}

pub fn map_sales(blk: eth::Block) -> Result<punks::Sales, Error> {
    let mut sales: Vec<punks::Sale> = Vec::new();
    for log in blk
        .logs()
        .filter(|log| log.address() == CRYPTOPUNKS_CONTRACT)
    {
        if let Some(sale_event) = cryptopunks_events::PunkBought::match_and_decode(log) {
            let bid_accepted = sale_event.to_address == NULL_ADDRESS;
            let buyer = if bid_accepted {
                // The contract emits its ERC20-style Transfer immediately before PunkBought.
                log.receipt
                    .receipt
                    .logs
                    .iter()
                    .rev()
                    .filter(|prior| {
                        prior.ordinal < log.ordinal() && prior.address == CRYPTOPUNKS_CONTRACT
                    })
                    .find_map(cryptopunks_events::Transfer::match_and_decode)
                    .filter(|transfer| transfer.from == sale_event.from_address)
                    .map(|transfer| transfer.to)
                    .ok_or_else(|| {
                        Error::msg("accepted bid has no preceding CryptoPunks Transfer")
                    })?
            } else {
                sale_event.to_address.clone()
            };
            sales.push(punks::Sale {
                bid_accepted,
                from: append_0x(&Hex(&sale_event.from_address).to_string()),
                to: append_0x(&Hex(&buyer).to_string()),
                token_id: sale_event.punk_index.to_u64(),
                amount: convert_and_divide(sale_event.value.to_string().as_str())?.to_string(),
                trx_hash: append_0x(&Hex(&log.receipt.transaction.hash).to_string()),
                block_number: blk.number,
                timestamp: blk.timestamp_seconds(),
                ordinal: log.ordinal(),
                log_index: log.block_index(),
                block_hash: append_0x(&Hex(&blk.hash).to_string()),
            });
        }
    }
    Ok(punks::Sales { sales })
}

pub fn map_bids(blk: eth::Block) -> Result<punks::Bids, Error> {
    let mut bids: Vec<punks::Bid> = vec![];
    for log in blk
        .logs()
        .filter(|log| log.address() == CRYPTOPUNKS_CONTRACT)
    {
        if let Some(bidentered_event) = cryptopunks_events::PunkBidEntered::match_and_decode(log) {
            bids.push(punks::Bid {
                from: append_0x(&Hex(&bidentered_event.from_address).to_string()),
                token_id: bidentered_event.punk_index.to_u64(),
                open: "true".to_string(),
                amount: convert_and_divide(bidentered_event.value.to_string().as_str())?
                    .to_string(),
                trx_hash: append_0x(&Hex(&log.receipt.transaction.hash).to_string()),
                block_number: blk.number,
                timestamp: blk.timestamp_seconds(),
                ordinal: log.ordinal(),
                log_index: log.block_index(),
                block_hash: append_0x(&Hex(&blk.hash).to_string()),
            });
        }

        if let Some(bidwithdrawn_event) =
            cryptopunks_events::PunkBidWithdrawn::match_and_decode(log)
        {
            bids.push(punks::Bid {
                from: append_0x(&Hex(&bidwithdrawn_event.from_address).to_string()),
                token_id: bidwithdrawn_event.punk_index.to_u64(),
                open: "false".to_string(),
                amount: convert_and_divide(bidwithdrawn_event.value.to_string().as_str())?
                    .to_string(),
                trx_hash: append_0x(&Hex(&log.receipt.transaction.hash).to_string()),
                block_number: blk.number,
                timestamp: blk.timestamp_seconds(),
                ordinal: log.ordinal(),
                log_index: log.block_index(),
                block_hash: append_0x(&Hex(&blk.hash).to_string()),
            });
        }
    }
    Ok(punks::Bids { bids })
}

pub fn map_asks(blk: eth::Block) -> Result<punks::Asks, Error> {
    let mut asks: Vec<punks::Ask> = vec![];
    for log in blk
        .logs()
        .filter(|log| log.address() == CRYPTOPUNKS_CONTRACT)
    {
        if let Some(askcreated_event) = cryptopunks_events::PunkOffered::match_and_decode(log) {
            asks.push(punks::Ask {
                from: append_0x(
                    &Hex(event_caller(log.receipt.transaction, log.ordinal())?).to_string(),
                ),
                to: append_0x(&Hex(&askcreated_event.to_address).to_string()),
                token_id: askcreated_event.punk_index.to_u64(),
                open: "true".to_string(),
                amount: Some(
                    convert_and_divide(askcreated_event.min_value.to_string().as_str())?
                        .to_string(),
                ),
                trx_hash: append_0x(&Hex(&log.receipt.transaction.hash).to_string()),
                block_number: blk.number,
                timestamp: blk.timestamp_seconds(),
                ordinal: log.ordinal(),
                log_index: log.block_index(),
                block_hash: append_0x(&Hex(&blk.hash).to_string()),
            });
        }
        if let Some(askremoved_event) =
            cryptopunks_events::PunkNoLongerForSale::match_and_decode(log)
        {
            asks.push(punks::Ask {
                from: append_0x(
                    &Hex(event_caller(log.receipt.transaction, log.ordinal())?).to_string(),
                ),
                to: append_0x(&Hex(NULL_ADDRESS).to_string()),
                token_id: askremoved_event.punk_index.to_u64(),
                open: "false".to_string(),
                amount: None,
                trx_hash: append_0x(&Hex(&log.receipt.transaction.hash).to_string()),
                block_number: blk.number,
                timestamp: blk.timestamp_seconds(),
                ordinal: log.ordinal(),
                log_index: log.block_index(),
                block_hash: append_0x(&Hex(&blk.hash).to_string()),
            });
        }
    }
    Ok(punks::Asks { asks })
}

//WRAPPEDPUNKS
pub fn map_user_proxies(blk: eth::Block) -> Result<punks::UserProxies, Error> {
    let mut user_proxies: Vec<punks::UserProxy> = vec![];
    for log in blk
        .logs()
        .filter(|log| log.address() == WRAPPEDPUNKS_CONTRACT)
    {
        if let Some(proxy_registered_event) =
            wrappedpunks_events::ProxyRegistered::match_and_decode(log)
        {
            user_proxies.push(punks::UserProxy {
                user: append_0x(&Hex(&proxy_registered_event.user).to_string()),
                proxy_address: append_0x(&Hex(&proxy_registered_event.proxy).to_string()),
                trx_hash: append_0x(&Hex(&log.receipt.transaction.hash).to_string()),
                block_number: blk.number,
                timestamp: blk.timestamp_seconds(),
                ordinal: log.ordinal(),
                log_index: log.block_index(),
                block_hash: append_0x(&Hex(&blk.hash).to_string()),
            });
        }
    }
    Ok(punks::UserProxies { user_proxies })
}

pub fn map_metadata(blk: Block) -> Result<punks::Metadatas, Error> {
    let Some(token) = metadata_token(blk.number) else {
        return Ok(punks::Metadatas::default());
    };
    let metadata = get_punk_metadata(token)?;
    Ok(punks::Metadatas {
        metadatas: vec![punks::Metadata {
            traits: get_traits(&metadata.attributes),
            token_id: token.to_string(),
            punk_type: get_type(&metadata.attributes),
            svg: metadata.svg,
            image: metadata.image,
            token_uri: format!("https://cryptopunks.app/cryptopunks/details/{token}"),
            contract_uri: "https://cryptopunks.app/cryptopunks".to_string(),
        }],
    })
}

pub(crate) fn metadata_token(block: u64) -> Option<u64> {
    (13_047_091..=13_057_090)
        .contains(&block)
        .then(|| 13_057_090 - block)
}

fn event_caller(transaction: &eth::TransactionTrace, ordinal: u64) -> Result<&[u8], Error> {
    transaction
        .calls
        .iter()
        .find(|call| {
            !call.state_reverted
                && call.address == CRYPTOPUNKS_CONTRACT
                && call.logs.iter().any(|event| event.ordinal == ordinal)
        })
        .map(|call| call.caller.as_slice())
        .or_else(|| (transaction.to == CRYPTOPUNKS_CONTRACT).then_some(transaction.from.as_slice()))
        .ok_or_else(|| {
            Error::msg("cannot resolve CryptoPunks event caller from transaction traces")
        })
}

pub fn map_wrapped_transfers(blk: eth::Block) -> Result<punks::Transfers, Error> {
    Ok(punks::Transfers {
        transfers: blk
            .events::<wrappedpunks_events::Transfer>(&[&WRAPPEDPUNKS_CONTRACT])
            .map(|(event, log)| punks::Transfer {
                from: append_0x(&Hex(&event.from).to_string()),
                to: append_0x(&Hex(&event.to).to_string()),
                wrapped: (event.to != NULL_ADDRESS).to_string(),
                token_id: event.token_id.to_u64(),
                trx_hash: append_0x(&Hex(&log.receipt.transaction.hash).to_string()),
                block_number: blk.number,
                timestamp: blk.timestamp_seconds(),
                block_hash: append_0x(&Hex(&blk.hash).to_string()),
                ordinal: log.ordinal(),
                log_index: log.block_index(),
            })
            .collect(),
    })
}
