//! Pure state transitions. Store access is injected so ordering is testable offline.
use crate::pb::cryptopunks as punks;
use anyhow::Error;

pub fn resolve_sales(
    input: punks::Sales,
    mut bid_before: impl FnMut(&punks::Sale) -> Option<punks::Bid>,
) -> Result<punks::Sales, Error> {
    let mut sales = Vec::with_capacity(input.sales.len());
    for mut sale in input.sales {
        if sale.bid_accepted {
            let bid = bid_before(&sale)
                .filter(|bid| {
                    bid.open == "true" && bid.from == sale.to && bid.token_id == sale.token_id
                })
                .ok_or_else(|| {
                    Error::msg(format!(
                        "missing active bid for accepted sale of punk {} at block {}",
                        sale.token_id, sale.block_number
                    ))
                })?;
            sale.amount = bid.amount;
        }
        sales.push(sale);
    }
    Ok(punks::Sales { sales })
}

/// Ownership changes silently clear a bid when the recipient is its bidder.
/// Keep markers per (punk, recipient), independent of raw bid history.
pub fn bid_resets(sales: punks::Sales, transfers: punks::Transfers) -> Vec<punks::Bid> {
    let mut resets: Vec<_> = sales
        .sales
        .into_iter()
        .map(|sale| punks::Bid {
            from: sale.to,
            token_id: sale.token_id,
            open: "false".into(),
            trx_hash: sale.trx_hash,
            block_number: sale.block_number,
            block_hash: sale.block_hash,
            timestamp: sale.timestamp,
            ordinal: sale.ordinal,
            log_index: sale.log_index,
            ..Default::default()
        })
        .chain(transfers.transfers.into_iter().map(|transfer| punks::Bid {
            from: transfer.to,
            token_id: transfer.token_id,
            open: "false".into(),
            trx_hash: transfer.trx_hash,
            block_number: transfer.block_number,
            block_hash: transfer.block_hash,
            timestamp: transfer.timestamp,
            ordinal: transfer.ordinal,
            log_index: transfer.log_index,
            ..Default::default()
        }))
        .collect();
    resets.sort_by_key(|bid| bid.ordinal);
    resets
}

pub fn active_bid(raw: Option<punks::Bid>, reset: Option<punks::Bid>) -> Option<punks::Bid> {
    raw.filter(|bid| {
        bid.open == "true"
            && !reset.is_some_and(|reset| {
                reset.token_id == bid.token_id
                    && reset.from == bid.from
                    && (reset.block_number, reset.ordinal) >= (bid.block_number, bid.ordinal)
            })
    })
}

pub fn bid_updates(
    input: punks::Bids,
    sales: punks::Sales,
    transfers: punks::Transfers,
    mut bid_before: impl FnMut(&punks::Bid) -> Option<punks::Bid>,
) -> Vec<punks::Bid> {
    let mut updates = input.bids;
    for mut reset in bid_resets(sales, transfers) {
        if let Some(bid) = bid_before(&reset).filter(|bid| {
            bid.open == "true" && bid.from == reset.from && bid.token_id == reset.token_id
        }) {
            reset.amount = bid.amount;
            updates.push(reset);
        }
    }
    updates.sort_by_key(|bid| bid.ordinal);
    updates
}

pub fn ask_updates(input: punks::Asks, sales: punks::Sales) -> Vec<punks::Ask> {
    let mut updates = input.asks;
    // Sales close listings, including accepted bids which emit no NoLongerForSale.
    for sale in sales.sales {
        updates.push(punks::Ask {
            from: sale.from,
            to: sale.to,
            token_id: sale.token_id,
            open: "false".to_string(),
            amount: Some(sale.amount),
            trx_hash: sale.trx_hash,
            block_number: sale.block_number,
            block_hash: sale.block_hash,
            timestamp: sale.timestamp,
            ordinal: sale.ordinal,
            log_index: sale.log_index,
        });
    }
    updates.sort_by_key(|ask| ask.ordinal);
    updates
}
