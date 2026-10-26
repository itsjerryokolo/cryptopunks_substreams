//! Append-only facts: ordinary SQL views derive state and aggregates from retained facts.
use crate::pb::{cryptopunks as p, database as db};

const ZERO: &str = "0x0000000000000000000000000000000000000000";

pub fn ownership_changes(
    assigns: p::Assigns,
    sales: p::Sales,
    transfers: p::Transfers,
    wrapped: p::Transfers,
) -> p::OwnershipChanges {
    let mut changes = Vec::new();
    for a in assigns.assigns {
        changes.push(p::OwnershipChange {
            token_id: a.token_id,
            asset: "native".into(),
            kind: "assignment".into(),
            from: ZERO.into(),
            to: a.to,
            trx_hash: a.trx_hash,
            block_number: a.block_number,
            timestamp: a.timestamp,
            block_hash: a.block_hash,
            ordinal: a.ordinal,
            log_index: a.log_index,
        });
    }
    for s in sales.sales {
        changes.push(p::OwnershipChange {
            token_id: s.token_id,
            asset: "native".into(),
            kind: "sale".into(),
            from: s.from,
            to: s.to,
            trx_hash: s.trx_hash,
            block_number: s.block_number,
            timestamp: s.timestamp,
            block_hash: s.block_hash,
            ordinal: s.ordinal,
            log_index: s.log_index,
        });
    }
    for (asset, events) in [("native", transfers), ("wrapped", wrapped)] {
        for t in events.transfers {
            let kind = if asset == "wrapped" && t.from == ZERO {
                "mint"
            } else if asset == "wrapped" && t.to == ZERO {
                "burn"
            } else {
                "transfer"
            };
            changes.push(p::OwnershipChange {
                token_id: t.token_id,
                asset: asset.into(),
                kind: kind.into(),
                from: t.from,
                to: t.to,
                trx_hash: t.trx_hash,
                block_number: t.block_number,
                timestamp: t.timestamp,
                block_hash: t.block_hash,
                ordinal: t.ordinal,
                log_index: t.log_index,
            });
        }
    }
    changes.sort_by_key(|c| (c.block_number, c.ordinal));
    p::OwnershipChanges { changes }
}

fn field(name: &str, value: impl ToString) -> db::Field {
    db::Field {
        name: name.into(),
        value: value.to_string(),
        update_op: db::field::UpdateOp::Set as i32,
    }
}
fn row(table: &str, tx: &str, log: u32, ordinal: u64, fields: Vec<db::Field>) -> db::TableChange {
    db::TableChange {
        table: table.into(),
        primary_key: Some(db::table_change::PrimaryKey::Pk(format!("{tx}:{log}"))),
        ordinal,
        operation: db::table_change::Operation::Create as i32,
        fields,
    }
}
fn provenance(
    tx: &str,
    block: u64,
    hash: &str,
    timestamp: u64,
    ordinal: u64,
    log: u32,
) -> Vec<db::Field> {
    vec![
        field("tx_hash", tx),
        field("block_number", block),
        field("block_hash", hash),
        field("timestamp", timestamp),
        field("ordinal", ordinal),
        field("log_index", log),
    ]
}
pub fn database_changes(
    ownership: p::OwnershipChanges,
    sales: p::Sales,
    bids: p::Bids,
) -> db::DatabaseChanges {
    let mut table_changes = Vec::new();
    for c in ownership.changes {
        let mut fields = provenance(
            &c.trx_hash,
            c.block_number,
            &c.block_hash,
            c.timestamp,
            c.ordinal,
            c.log_index,
        );
        fields.extend([
            field("token_id", c.token_id),
            field("asset", c.asset),
            field("kind", c.kind),
            field("from_address", c.from),
            field("to_address", c.to),
        ]);
        table_changes.push(row(
            "ownership_history",
            &c.trx_hash,
            c.log_index,
            c.ordinal,
            fields,
        ));
    }
    for s in sales.sales {
        let mut fields = provenance(
            &s.trx_hash,
            s.block_number,
            &s.block_hash,
            s.timestamp,
            s.ordinal,
            s.log_index,
        );
        fields.extend([
            field("token_id", s.token_id),
            field("seller", s.from),
            field("buyer", s.to),
            field("amount_eth", s.amount),
            field("bid_accepted", s.bid_accepted),
        ]);
        table_changes.push(row("sales", &s.trx_hash, s.log_index, s.ordinal, fields));
    }
    for b in bids.bids {
        let mut fields = provenance(
            &b.trx_hash,
            b.block_number,
            &b.block_hash,
            b.timestamp,
            b.ordinal,
            b.log_index,
        );
        fields.extend([
            field("token_id", b.token_id),
            field("bidder", b.from),
            field("amount_eth", b.amount),
            field("is_open", b.open),
        ]);
        table_changes.push(row(
            "bid_changes",
            &b.trx_hash,
            b.log_index,
            b.ordinal,
            fields,
        ));
    }
    table_changes.sort_by_key(|c| c.ordinal);
    db::DatabaseChanges { table_changes }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mint_transfer_burn_preserve_order_and_identity() {
        let transfer = |from: &str, to: &str, ordinal, log_index| p::Transfer {
            from: from.into(),
            to: to.into(),
            ordinal,
            log_index,
            token_id: 9972,
            trx_hash: "0xabc".into(),
            block_number: 42,
            ..Default::default()
        };
        let changes = ownership_changes(
            p::Assigns::default(),
            p::Sales::default(),
            p::Transfers::default(),
            p::Transfers {
                transfers: vec![
                    transfer("b", ZERO, 30, 3),
                    transfer(ZERO, "a", 10, 1),
                    transfer("a", "b", 20, 2),
                ],
            },
        );
        assert_eq!(
            changes
                .changes
                .iter()
                .map(|c| c.kind.as_str())
                .collect::<Vec<_>>(),
            ["mint", "transfer", "burn"]
        );
        let rows = database_changes(changes, p::Sales::default(), p::Bids::default()).table_changes;
        assert_eq!(rows.len(), 3);
        assert_ne!(rows[0].primary_key, rows[1].primary_key);
        assert_eq!(rows[2].ordinal, 30);
    }
    #[test]
    fn sql_preserves_wei_precision_and_zero_value_sales() {
        for amount in ["0", "0.000000000000000001", "123456789.123456789123456789"] {
            let sale = p::Sale {
                amount: amount.into(),
                bid_accepted: true,
                ..Default::default()
            };
            let rows = database_changes(
                p::OwnershipChanges::default(),
                p::Sales { sales: vec![sale] },
                p::Bids::default(),
            )
            .table_changes;
            assert_eq!(
                rows[0]
                    .fields
                    .iter()
                    .find(|f| f.name == "amount_eth")
                    .unwrap()
                    .value,
                amount
            );
        }
    }
}
