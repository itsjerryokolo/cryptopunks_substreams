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

pub fn bid_updates(
    input: punks::Bids,
    sales: punks::Sales,
    mut bid_before: impl FnMut(&punks::Sale) -> Option<punks::Bid>,
) -> Vec<punks::Bid> {
    let mut updates = input.bids;
    for sale in sales.sales {
        if let Some(mut bid) =
            bid_before(&sale).filter(|bid| bid.open == "true" && bid.from == sale.to)
        {
            bid.open = "false".to_string();
            bid.ordinal = sale.ordinal;
            bid.log_index = sale.log_index;
            bid.trx_hash = sale.trx_hash;
            bid.block_number = sale.block_number;
            bid.block_hash = sale.block_hash;
            bid.timestamp = sale.timestamp;
            updates.push(bid);
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
