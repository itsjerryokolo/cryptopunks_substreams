mod abi;
mod pb;
mod utils;

use abi::cryptopunks::events as cryptopunks_events;
use abi::wrappedpunks::events as wrappedpunks_events;

use pb::cryptopunks as punks;
use substreams::prelude::*;
use substreams::store::StoreSet;
use substreams::{log, Hex};
use substreams_ethereum::{pb::eth::v2 as eth, Event};
use utils::constants::{CRYPTOPUNKS_CONTRACT, WRAPPEDPUNKS_CONTRACT};
use utils::keyer::{
    generate_key, KeyType::Assignee as Assignee_Key, KeyType::Bidder as Bidder_Key,
    KeyType::Owner as Owner_Key, KeyType::Punk as Punk_Key, KeyType::UserProxy as Proxy_Key,
};
use utils::math::{convert_and_divide, decimal_from_str};

substreams_ethereum::init!();

// Extracts transfers events from the contract
#[substreams::handlers::map]
fn map_transfers(
    blk: eth::Block,
    user_proxies: StoreGetProto<punks::UserProxy>,
) -> Result<punks::Transfers, substreams::errors::Error> {
    Ok(punks::Transfers {
        transfers: blk
            .events::<cryptopunks_events::PunkTransfer>(&[&CRYPTOPUNKS_CONTRACT])
            .map(|(transfer, log)| {
                log::info!("NFT Transfer seen");
                let from_account = Hex(transfer.from.clone()).to_string();
                let to_account = Hex(transfer.to.clone()).to_string();

                let user_proxy =
                    user_proxies.get_last(generate_key(Proxy_Key, &from_account).unwrap());

                if let Some(users) = user_proxy {
                    if from_account == users.proxy_address
                        && to_account == Hex(WRAPPEDPUNKS_CONTRACT).to_string()
                    {
                        //Wrap
                        return punks::Transfer {
                            from: from_account,
                            to: to_account,
                            block_number: blk.number,
                            timestamp: blk.timestamp_seconds(),
                            trx_hash: Hex(&log.receipt.transaction.hash).to_string(),
                            token_id: transfer.punk_index.to_u64(),
                            wrapped: true,
                            ordinal: log.block_index() as u64,
                        };
                    }
                }
                if Hex(transfer.from.clone()).to_string() == Hex(WRAPPEDPUNKS_CONTRACT).to_string()
                {
                    //Unwrap
                    punks::Transfer {
                        from: from_account,
                        to: to_account,
                        block_number: blk.number,
                        timestamp: blk.timestamp_seconds(),
                        trx_hash: Hex(&log.receipt.transaction.hash).to_string(),
                        token_id: transfer.punk_index.to_u64(),
                        wrapped: false,
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
                        timestamp: blk.timestamp_seconds(),
                        trx_hash: Hex(&log.receipt.transaction.hash).to_string(),
                        token_id: transfer.punk_index.to_u64(),
                        wrapped: true,
                        ordinal: log.block_index() as u64,
                    }
                } else {
                    //Regular Transfer
                    punks::Transfer {
                        from: Hex(transfer.from).to_string(),
                        to: Hex(transfer.to).to_string(),
                        block_number: blk.number,
                        timestamp: blk.timestamp_seconds(),
                        trx_hash: Hex(&log.receipt.transaction.hash).to_string(),
                        token_id: transfer.punk_index.to_u64(),
                        wrapped: false,
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
        if let Some(bid_event) = cryptopunks_events::PunkBidEntered::match_and_decode(log) {
            log::info!("Bid Event Found");

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

//WRAPPEDPUNKS
#[substreams::handlers::map]
fn map_user_proxies(blk: eth::Block) -> Result<punks::UserProxies, substreams::errors::Error> {
    let mut user_proxies: Vec<punks::UserProxy> = vec![];
    for log in blk.logs() {
        if let Some(proxy_registered_event) =
            wrappedpunks_events::ProxyRegistered::match_and_decode(log)
        {
            log::info!("Bid Event Found");

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

// STORES //
#[substreams::handlers::store]
pub fn store_assigns(assigns: punks::Assigns, output: StoreSetProto<punks::Assign>) {
    for assign in assigns.assigns {
        output.set(
            0,
            generate_key(Punk_Key, &assign.token_id.to_string().as_str()).unwrap(),
            &assign,
        );
    }
}

pub fn punks_assignees(assigns: punks::Assigns, output: StoreAppend<String>) {
    for assign in assigns.assigns {
        output.append(
            0,
            generate_key(Assignee_Key, &assign.to).unwrap(),
            assign.token_id.to_string(),
        );
    }
}

#[substreams::handlers::store]
pub fn punk_state(transfers: punks::Transfers, output: StoreSetProto<punks::Transfer>) {
    for transfer in transfers.transfers {
        let token_id = transfer.token_id as i64;

        output.set(
            0,
            generate_key(Punk_Key, &token_id.to_string().as_str()).unwrap(),
            &transfer,
        );

        output.set(0, generate_key(Owner_Key, &transfer.to).unwrap(), &transfer);
    }
}

#[substreams::handlers::store]
pub fn store_bids(bids: punks::Bids, output: StoreSetProto<punks::Bid>) {
    for bidder in bids.bids {
        output.set(0, generate_key(Bidder_Key, &bidder.to).unwrap(), &bidder);
    }
}

#[substreams::handlers::store]
pub fn store_all_punks(assigns: punks::Assigns, output: StoreAppend<String>) {
    for assign in assigns.assigns {
        let token_id = assign.token_id as i64;
        output.append(
            0,
            generate_key(Punk_Key, &token_id.to_string().as_str()).unwrap(),
            token_id.to_string(),
        );
    }
}

#[substreams::handlers::store]
pub fn store_total_volume(s: punks::Sales, output: StoreAddBigDecimal) {
    for vol in s.sales {
        let val = decimal_from_str(vol.amount.as_str()).unwrap();
        output.add(0, Hex(CRYPTOPUNKS_CONTRACT).to_string(), val);
    }
}

#[substreams::handlers::store]
pub fn store_punk_sales(s: punks::Sales, output: StoreSetProto<punks::Sale>) {
    for sale in s.sales {
        let token_id = sale.token_id as i64;
        let ordinal = sale.ordinal;
        output.set(
            ordinal,
            generate_key(Punk_Key, &token_id.to_string().as_str()).unwrap(),
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
            generate_key(Punk_Key, &token_id.to_string().as_str()).unwrap(),
            val,
        );
    }
}

#[substreams::handlers::store]
pub fn store_user_proxies(u: punks::UserProxies, output: StoreSetProto<punks::UserProxy>) {
    for proxy in u.user_proxies {
        output.set(
            0,
            generate_key(Proxy_Key, &proxy.proxy_address.to_string().as_str()).unwrap(),
            &proxy,
        );
    }
}
