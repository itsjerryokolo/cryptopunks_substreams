# Validation

In September 2026, a local PostgreSQL 14.19 database ingested **100,000 finalized Ethereum mainnet blocks**, starting at 3,914,494 and stopping before 4,014,494, using Substreams CLI 1.23.0. A separate Python decoder replayed receipts from the original market and compared the results with the indexed rows. Both implementations used receipts from the same provider.

| Result over the recorded range | What matched |
|---|---|
| 572 sales | Buyers, sellers and exact prices in ETH |
| 4,909 bid changes | 2,592 new bids, 1,295 replacements, 856 withdrawals, 152 accepted bids and 14 purchase refunds |
| 2,292 latest bid snapshots | Bidder, Punk, amount and open or closed state |
| 10,769 native ownership events | Event history and all 10,000 native owners at the stop block |
| 20 market-day and 565 per-Punk/day summaries | Counts, volume and price statistics for indexed sales |

Development and production execution modes returned the same 4,909 bid changes. Repeating the first 10,000-block SQL ingestion added zero rows. Extending the stop block resumed from the saved cursor. Day totals cover only the indexed portion of each UTC day.

## Inspect a recorded example

| Case | Evidence |
|---|---|
| Accepted bid | At block 3,919,706, Punk 544's sale resolves to **0.01 ETH** using the bid entered at block 3,919,689. The sale matches the recorded website price, date, parties and transaction. [Sale observations](../tests/fixtures/official-website-sales.json) |
| Direct sale | Punk 4662 sold for **3.1 ETH** at block 10,919,768. The buyer, seller, amount and event position match the market receipt. [Receipt fixture](../tests/fixtures/mainnet-sale-10919768.json) |
| Same-block ordering | A bidder raised its offer on Punk 4936 from 0.05 to 0.1 ETH in two transactions in block 4,009,734. Another Punk's bid fell between them. [Receipt fixture](../tests/fixtures/mainnet-bid-ordering-4009734.json) |
| Wrapped custody | In blocks 10,951,800 through 10,951,859, Punk 4046 burned its wrapped token and returned to native custody. Punk 9972 entered the wrapper and received a wrapped holder. [Reproduce the ownership query](SQL_GUIDE.md#wrapped-ownership) |
| Metadata | For Punks 5822, 4156, 8857 and 0, the type and traits matched the recorded website observations. Each SVG matched its RGBA output at all 576 pixels. [Metadata observations](../tests/fixtures/official-website-sep30.json) |
| Second-provider check | A separate public RPC receipt confirmed the transfer of Punk 4201 in block 26,089,666, including both addresses and the log index. This check covered one transaction. [Receipt fixture](../tests/fixtures/mainnet-transfer-26089666.json) |

The replay stopped before chain head. Full token metadata coverage, hosted operation and recovery from a provider-delivered reorg remain unverified. SQL regressions exercise deletion of reverted rows. The Rust suite covers a transfer directly to a standing bidder, a case absent from this historical sample. [SQL regressions](../tests/sql_views.sql) · [Rust regressions](../src/tests.rs)

## Reproduce the checks

From the repository root, `make check` runs formatting, Clippy and the Rust regressions without a streaming account. To test the SQL views, point `psql` at a local test database where your role can create a schema. The test runs in a transaction and rolls it back.

```sh
make check
psql -X -f tests/sql_views.sql
```

For a live replay, install Python 3 and Substreams CLI 1.23.0, then run the following from the repository root. These requests consume streaming quota. The first script validates bounded sale, bid, transfer and metadata samples. The second checks four fixed metadata records, the 8,000 ETH sale of Punk 5822 and the transfer of Punk 4201. Each prints its capture directory.

```sh
make pack
substreams auth
bash scripts/live_check.sh
bash scripts/additional_samples.sh
```

To reproduce the 100,000-block comparison, ingest through exclusive stop block `4014494` using the PostgreSQL guide. Set `PGHOST`, `PGPORT`, `PGDATABASE` and `PGUSER` for `psql` to connect to that same database. Capture the matching receipt range, check both bid execution modes, then verify the indexed facts and views. [PostgreSQL guide](SQL_GUIDE.md)

```sh
capture_dir=$(mktemp -d)
BLOCK_COUNT=100000 bash scripts/review_bid_history.sh "$capture_dir"
python3 scripts/verify_sql.py --receipts "$capture_dir/receipts.jsonl"
python3 scripts/verify_ownership.py "$capture_dir/receipts.jsonl"
```
