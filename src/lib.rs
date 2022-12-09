mod abi;
mod pb;
mod rpc;
mod utils;

use abi::cryptopunks::events as cryptopunks_events;
use abi::wrappedpunks::events as wrappedpunks_events;
use std::str::FromStr;
use substreams_ethereum::pb::eth::v2::Block;

use pb::cryptopunks as punks;
use substreams::prelude::*;
use substreams::store::StoreSet;
use substreams::{log, scalar::BigInt, Hex};

use rpc::{get_contract_data, get_punk_metadata};
use substreams_ethereum::{pb::eth::v2 as eth, Event, NULL_ADDRESS};
use utils::constants::{CRYPTOPUNKS_CONTRACT, WRAPPEDPUNKS_CONTRACT};
use utils::helper::{get_traits, get_type};
use utils::keyer::{
    generate_key, KeyType::Assignee as Assignee_Key, KeyType::Bidder as Bidder_Key,
    KeyType::Day as Day_Key, KeyType::Owner as Owner_Key, KeyType::Punk as Punk_Key,
    KeyType::UserProxy as Proxy_Key,
};
use utils::math::{convert_and_divide, decimal_from_str};

substreams_ethereum::init!();

// Extracts transfers events from the contract
#[substreams::handlers::map]
fn map_transfers(blk: eth::Block) -> Result<punks::Transfers, substreams::errors::Error> {
    Ok(punks::Transfers {
        transfers: blk
            .events::<cryptopunks_events::PunkTransfer>(&[&CRYPTOPUNKS_CONTRACT])
            .map(|(transfer, log)| {
                log::info!("NFT Transfer seen");
                let from_account = Hex(transfer.from.clone()).to_string();
                let to_account = Hex(transfer.to.clone()).to_string();

                if Hex(transfer.from.clone()).to_string() == Hex(WRAPPEDPUNKS_CONTRACT).to_string()
                {
                    //Unwrap
                    punks::Transfer {
                        from: from_account,
                        to: to_account,
                        block_number: blk.number,
                        wrapped: "false".to_string(),
                        timestamp: blk.timestamp_seconds(),
                        trx_hash: Hex(&log.receipt.transaction.hash).to_string(),
                        token_id: transfer.punk_index.to_u64(),
                        ordinal: log.block_index() as u64,
                    }
                } else if Hex(transfer.to.clone()).to_string()
                    == Hex(WRAPPEDPUNKS_CONTRACT).to_string()
                {
                    //Wrap
                    punks::Transfer {
                        from: Hex(transfer.from).to_string(),
                        to: Hex(transfer.to).to_string(),
                        block_number: blk.number,
                        wrapped: "true".to_string(),
                        timestamp: blk.timestamp_seconds(),
                        trx_hash: Hex(&log.receipt.transaction.hash).to_string(),
                        token_id: transfer.punk_index.to_u64(),
                        ordinal: log.block_index() as u64,
                    }
                } else {
                    //Regular Transfer
                    punks::Transfer {
                        from: Hex(transfer.from).to_string(),
                        to: Hex(transfer.to).to_string(),
                        wrapped: "false".to_string(),
                        block_number: blk.number,
                        timestamp: blk.timestamp_seconds(),
                        trx_hash: Hex(&log.receipt.transaction.hash).to_string(),
                        token_id: transfer.punk_index.to_u64(),
                        ordinal: log.block_index() as u64,
                    }
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
        if let Some(assign_event) = cryptopunks_events::Assign::match_and_decode(log) {
            log::info!("Assign Event Found");
            let contract_calls;

            //We only need to call the contract once
            if blk.number == 3919682 as u64 {
                contract_calls = get_contract_data();

                assigns.push(punks::Assign {
                    to: Hex(&assign_event.to).to_string(),
                    token_id: assign_event.punk_index.to_u64(),
                    trx_hash: Hex(&log.receipt.transaction.hash).to_string(),
                    block_number: blk.number,
                    timestamp: blk.timestamp_seconds(),
                    ordinal: log.block_index() as u64,
                    contract: Some(punks::Contract {
                        total_supply: contract_calls.0,
                        name: contract_calls.1,
                        symbol: contract_calls.2,
                        image_hash: contract_calls.3,
                    }),
                })
            }

            //Create assign
            assigns.push(punks::Assign {
                to: Hex(&assign_event.to).to_string(),
                token_id: assign_event.punk_index.to_u64(),
                trx_hash: Hex(&log.receipt.transaction.hash).to_string(),
                block_number: blk.number,
                timestamp: blk.timestamp_seconds(),
                ordinal: log.block_index() as u64,
                contract: None,
            })
        }
    }
    Ok(punks::Assigns { assigns })
}

#[substreams::handlers::map]
fn map_sales(blk: eth::Block) -> Result<punks::Sales, substreams::errors::Error> {
    let mut sales: Vec<punks::Sale> = Vec::new();
    for log in blk.logs() {
        if let Some(sale_event) = cryptopunks_events::PunkBought::match_and_decode(log) {
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
        if let Some(bidentered_event) = cryptopunks_events::PunkBidEntered::match_and_decode(log) {
            log::info!("Bid Event Found");

            bids.push(punks::Bid {
                from: Hex(&bidentered_event.from_address).to_string(),
                token_id: bidentered_event.punk_index.to_u64(),
                open: "true".to_string(),
                amount: convert_and_divide(bidentered_event.value.to_string().as_str())
                    .unwrap()
                    .to_string(),
                trx_hash: Hex(&log.receipt.transaction.hash).to_string(),
                block_number: blk.number,
                timestamp: blk.timestamp_seconds(),
                ordinal: log.block_index() as u64,
            });
        }

        if let Some(bidwithdrawn_event) =
            cryptopunks_events::PunkBidWithdrawn::match_and_decode(log)
        {
            log::info!("Bid Event Found");

            bids.push(punks::Bid {
                from: Hex(&bidwithdrawn_event.from_address).to_string(),
                token_id: bidwithdrawn_event.punk_index.to_u64(),
                open: "false".to_string(),
                amount: convert_and_divide(bidwithdrawn_event.value.to_string().as_str())
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

#[substreams::handlers::map]
fn map_asks(blk: eth::Block) -> Result<punks::Asks, substreams::errors::Error> {
    let mut asks: Vec<punks::Ask> = vec![];
    for log in blk.logs() {
        if let Some(askcreated_event) = cryptopunks_events::PunkOffered::match_and_decode(log) {
            log::info!("AskCreated Event Found");

            asks.push(punks::Ask {
                from: Hex(&log.receipt.transaction.from).to_string(),
                to: Hex(&askcreated_event.to_address).to_string(),
                token_id: askcreated_event.punk_index.to_u64(),
                open: "true".to_string(),
                amount: Some(
                    convert_and_divide(askcreated_event.min_value.to_string().as_str())
                        .unwrap()
                        .to_string(),
                ),
                trx_hash: Hex(&log.receipt.transaction.hash).to_string(),
                block_number: blk.number,
                timestamp: blk.timestamp_seconds(),
                ordinal: log.block_index() as u64,
            });
        }
        if let Some(askremoved_event) =
            cryptopunks_events::PunkNoLongerForSale::match_and_decode(log)
        {
            log::info!("AskRemoved Event Found");

            asks.push(punks::Ask {
                from: Hex(&log.receipt.transaction.from).to_string(),
                to: Hex(NULL_ADDRESS).to_string(),
                token_id: askremoved_event.punk_index.to_u64(),
                open: "false".to_string(),
                amount: None,
                trx_hash: Hex(&log.receipt.transaction.hash).to_string(),
                block_number: blk.number,
                timestamp: blk.timestamp_seconds(),
                ordinal: log.block_index() as u64,
            });
        }
    }
    Ok(punks::Asks { asks })
}

//WRAPPEDPUNKS
#[substreams::handlers::map]
fn map_user_proxies(blk: eth::Block) -> Result<punks::UserProxies, substreams::errors::Error> {
    let mut user_proxies: Vec<punks::UserProxy> = vec![];
    for log in blk.logs() {
        if let Some(proxy_registered_event) =
            wrappedpunks_events::ProxyRegistered::match_and_decode(log)
        {
            log::info!("User Proxy Event Found");

            user_proxies.push(punks::UserProxy {
                user: Hex(&proxy_registered_event.user).to_string(),
                proxy_address: Hex(&proxy_registered_event.proxy).to_string(),
                trx_hash: Hex(&log.receipt.transaction.hash).to_string(),
                block_number: blk.number,
                timestamp: blk.timestamp_seconds(),
                ordinal: log.block_index() as u64,
            });
        }
    }
    Ok(punks::UserProxies { user_proxies })
}

#[substreams::handlers::map]
fn map_metadata(blk: Block) -> Result<punks::Metadatas, substreams::errors::Error> {
    let end_block = BigInt::from_str("13057090").unwrap();
    let mut metadatas: Vec<punks::Metadata> = vec![];
    log::info!("Metadata handler found");

    if BigInt::from(blk.number).gt(&end_block) {
        return Ok(punks::Metadatas { metadatas });
    }
    let index = end_block - BigInt::from(blk.number);
    let token = index.to_string();

    let call = get_punk_metadata(&token);

    metadatas.push(punks::Metadata {
        traits: get_traits(&call.0),
        token_id: token.to_string(),
        punk_type: get_type(&call.0),
        svg: call.1,
        image: call.2,
        token_uri: format!(
            "https://cryptopunks.app/cryptopunks/details/{}",
            &token.to_string()
        ),
        contract_uri: "https://cryptopunks.app/cryptopunks".to_string(),
    });

    Ok(punks::Metadatas { metadatas })
}

#[substreams::handlers::map]
fn map_wrapped_transfers(blk: eth::Block) -> Result<punks::Transfers, substreams::errors::Error> {
    let mut wrapped_punks: Vec<punks::Transfer> = vec![];
    for log in blk.logs() {
        if let Some(wrappedpunk_transfer_event) =
            wrappedpunks_events::Transfer::match_and_decode(log)
        {
            log::info!("WrappedPunk Event Found");

            let from_account = Hex(wrappedpunk_transfer_event.from.clone()).to_string();
            let to_account = Hex(wrappedpunk_transfer_event.to.clone()).to_string();

            if Hex(wrappedpunk_transfer_event.to.clone()).to_string()
                == Hex(NULL_ADDRESS).to_string()
            {
                wrapped_punks.push(
                    //Wrap
                    punks::Transfer {
                        from: from_account.to_string(),
                        to: to_account.to_string(),
                        block_number: blk.number,
                        wrapped: "true".to_string(),
                        timestamp: blk.timestamp_seconds(),
                        trx_hash: Hex(&log.receipt.transaction.hash).to_string(),
                        token_id: wrappedpunk_transfer_event.token_id.to_u64(),
                        ordinal: log.block_index() as u64,
                    },
                );
            }
            if Hex(wrappedpunk_transfer_event.from.clone()).to_string()
                == Hex(NULL_ADDRESS).to_string()
            {
                wrapped_punks.push(
                    //Unwrap
                    punks::Transfer {
                        from: from_account.to_string(),
                        to: to_account,
                        block_number: blk.number,
                        wrapped: "false".to_string(),
                        timestamp: blk.timestamp_seconds(),
                        trx_hash: Hex(&log.receipt.transaction.hash).to_string(),
                        token_id: wrappedpunk_transfer_event.token_id.to_u64(),
                        ordinal: log.block_index() as u64,
                    },
                );
            } else {
                //Ignore regular wrappedpunks transfers
                continue;
            }
        }
    }
    Ok(punks::Transfers {
        transfers: wrapped_punks,
    })
}

// STORES //
#[substreams::handlers::store]
pub fn store_assigns(i: punks::Assigns, o: StoreSetProto<punks::Assign>) {
    for assign in i.assigns {
        o.set(
            0,
            generate_key(Punk_Key, &assign.token_id.to_string().as_str()),
            &assign,
        );
    }
}

#[substreams::handlers::store]
pub fn punks_assignees(i: punks::Assigns, o: StoreAppend<String>) {
    for assign in i.assigns {
        o.append(
            0,
            generate_key(Assignee_Key, &assign.to),
            assign.token_id.to_string(),
        );
    }
}

#[substreams::handlers::store]
pub fn punk_state(
    i: punks::Transfers,
    i2: StoreGetProto<punks::UserProxy>,
    o: StoreSetProto<punks::Transfer>,
) {
    for mut transfer in i.transfers {
        let token_id = transfer.token_id as i64;
        let from_account = Hex(transfer.from.clone()).to_string();
        let to_account = Hex(transfer.to.clone()).to_string();

        o.set(
            0,
            generate_key(Punk_Key, &token_id.to_string().as_str()),
            &transfer,
        );

        let user_proxy = i2.get_last(generate_key(Proxy_Key, transfer.from.as_str()));

        if let Some(proxy) = user_proxy {
            if from_account == proxy.proxy_address
                && to_account == Hex(WRAPPEDPUNKS_CONTRACT).to_string()
            {
                transfer.wrapped = "true".to_string();
                o.set(
                    0,
                    generate_key(Punk_Key, &token_id.to_string().as_str()),
                    &transfer,
                );
            }
        }
    }
}

#[substreams::handlers::store]
//Updates both the Bid State for the punk and new bids from the Bidder
pub fn bids_state(i: punks::Bids, i2: StoreGetProto<punks::Sale>, o: StoreSetProto<punks::Bid>) {
    for mut bid in i.bids {
        let token_id = bid.token_id as i64;
        o.set(0, generate_key(Bidder_Key, &bid.from), &bid);

        o.set(
            0,
            generate_key(Punk_Key, &token_id.to_string().as_str()),
            &bid,
        );

        let sales = i2.get_last(generate_key(Punk_Key, &token_id.to_string().as_str()));

        if let Some(sale) = sales {
            if bid.from == sale.to {
                bid.open = "false".to_string();
                o.set(
                    0,
                    generate_key(Punk_Key, &token_id.to_string().as_str()),
                    &bid,
                );
            }
        }
    }
}

#[substreams::handlers::store]
//Updates both the Ask State for the punk and new asks from the Owner
pub fn asks_state(i: punks::Asks, blk: eth::Block, o: StoreSetProto<punks::Ask>) {
    for ask in i.asks {
        for log in blk.logs() {
            if let Some(askremoved) = cryptopunks_events::PunkNoLongerForSale::match_and_decode(log)
            {
                let closed_ask = punks::Ask {
                    from: Hex(&log.receipt.transaction.from).to_string(),
                    to: ask.to.clone(),
                    token_id: askremoved.punk_index.to_u64(),
                    open: "false".to_string(),
                    amount: ask.amount.clone(),
                    trx_hash: Hex(&log.receipt.transaction.hash).to_string(),
                    block_number: blk.number,
                    timestamp: blk.timestamp_seconds(),
                    ordinal: log.block_index() as u64,
                };

                o.set(
                    0,
                    generate_key(Punk_Key, askremoved.punk_index.to_string().as_str()),
                    &closed_ask,
                );

                o.set(0, generate_key(Owner_Key, &ask.from), &closed_ask);
            }
        }
    }
}

#[substreams::handlers::store]
pub fn store_all_punks(assigns: punks::Assigns, o: StoreAppend<String>) {
    for assign in assigns.assigns {
        let token_id = assign.token_id as i64;
        o.append(
            0,
            Hex(CRYPTOPUNKS_CONTRACT).to_string(),
            token_id.to_string(),
        );
    }
}

#[substreams::handlers::store]
pub fn store_total_volume(i: punks::Sales, i2: StoreGetProto<punks::Bid>, o: StoreAddBigDecimal) {
    for sale in i.sales {
        let val = decimal_from_str(sale.amount.as_str()).unwrap();
        let token_id = sale.token_id as i64;
        o.add(0, Hex(CRYPTOPUNKS_CONTRACT).to_string(), &val);

        let day_id = sale.timestamp / 86400;

        o.add(0, generate_key(Day_Key, &day_id.to_string().as_str()), &val);

        let sales = i2.get_last(generate_key(Punk_Key, &token_id.to_string().as_str()));

        if let Some(bid) = sales {
            if bid.from == sale.to {
                let amount = BigDecimal::from_str(bid.amount.as_str()).unwrap();
                o.add(0, Hex(CRYPTOPUNKS_CONTRACT).to_string(), amount);
            }
        }
    }
}

#[substreams::handlers::store]
pub fn store_punk_sales(i: punks::Sales, o: StoreSetProto<punks::Sale>) {
    for sale in i.sales {
        let token_id = sale.token_id as i64;
        let ordinal = sale.ordinal;
        o.set(
            ordinal,
            generate_key(Punk_Key, &token_id.to_string().as_str()),
            &sale,
        );
    }
}

#[substreams::handlers::store]
pub fn store_punk_volume(i: punks::Sales, i2: StoreGetProto<punks::Bid>, o: StoreAddBigDecimal) {
    for sale in i.sales {
        let token_id = sale.token_id as i64;
        let val = decimal_from_str(sale.amount.as_str()).unwrap();

        let day_id = sale.timestamp / 86400;

        o.add(0, generate_key(Day_Key, &day_id.to_string().as_str()), &val);

        o.add(
            0,
            generate_key(Punk_Key, &token_id.to_string().as_str()),
            &val,
        );

        let sales = i2.get_last(generate_key(Punk_Key, &token_id.to_string().as_str()));

        if let Some(bid) = sales {
            if bid.from == sale.to {
                let amount = BigDecimal::from_str(bid.amount.as_str()).unwrap();
                o.add(0, Hex(CRYPTOPUNKS_CONTRACT).to_string(), amount);
            }
        }
    }
}

#[substreams::handlers::store]
pub fn store_user_proxies(i: punks::UserProxies, o: StoreSetProto<punks::UserProxy>) {
    for proxy in i.user_proxies {
        o.set(
            0,
            generate_key(Proxy_Key, &proxy.proxy_address.to_string().as_str()),
            &proxy,
        );
    }
}

#[substreams::handlers::store]
pub fn contract_metadata(i: punks::Assigns, o: StoreSetProto<punks::Contract>) {
    //There is only one Assign with the Contract message so we need to filter
    for assign in i.assigns {
        match assign.contract {
            Some(value) => {
                o.set(0, Hex(CRYPTOPUNKS_CONTRACT).to_string(), &value);
            }
            None => continue,
        }
    }
}

#[substreams::handlers::store]
pub fn store_metadata(i: punks::Metadatas, o: StoreSetProto<punks::Metadata>) {
    for metadata in i.metadatas {
        o.set(0, generate_key(Punk_Key, &metadata.token_id), &metadata)
    }
}
