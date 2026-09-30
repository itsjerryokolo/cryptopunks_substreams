# Local PostgreSQL walkthrough

The package now has a PostgreSQL `db_out`, an ownership ledger, and ordinary SQL
views for current ownership, current bids, daily market totals and daily per-Punk
totals. No publication is required to use a local `.spkg`.

## Schema and semantics

- `sales`: resolved original-market sales, with exact PostgreSQL `numeric` ETH
  amounts, accepted-bid flag, buyer/seller and full event provenance.
- `ownership_history`: assignments, sales, native transfers, WrappedPunks
  mints/transfers/burns. Transaction hash plus block log index is a deterministic
  primary key. Native market generic `Transfer` logs are used to recover sale
  buyers, not counted a second time as ownership changes.
- `bid_changes`: reconciled bids including silent refund closures. Closed rows
  retain the bid amount for audit history; it is not additional sale volume.
- `current_ownership`: native contract owner, wrapped holder, effective holder.
  A wrapper custodian is not presented as the beneficial holder. Unknown wrapped
  custody gives a NULL effective holder. Arbitrary contract/proxy addresses are
  retained as recorded; this does not infer real-world ownership.
- `current_bids`: latest snapshot per Punk. Filter `is_open` for active bids.
- `daily_market_summary` / `daily_punk_summary`: UTC-day sale counts and exact
  volume/min/max. Market summaries also expose average price and distinct
  buyers/sellers. PostgreSQL rounds repeating averages; sums remain exact.

Views derive values directly from retained facts, so removing reverted facts
also removes their contribution. They are not materialized and need no refresh.
This favors auditability over maximum query throughput; benchmark a complete
index before committing to a production latency target.

These summaries cover the original CryptoPunks marketplace, including zero-price
and self-sales. They do not include external WrappedPunks exchange prices,
off-chain orders, wash-trade exclusions, or USD conversions. Metadata, proxies
and asks remain available as typed stream outputs; this SQL schema does not yet
mirror every module.

**Current means through the sink's processed cursor, not necessarily chain head.**
Start a full index at block 3914494. A recent bounded ownership-only run can lack
prior native custody (and omit such tokens from `current_ownership`). Keep those
experiments in a separate database.

## Run locally

Use PostgreSQL 14+ and Substreams CLI 1.23.0. Create a fresh local database with a
role allowed to create tables/views. Store credentials in an ignored local env
file or your shell; never put actual credentials in committed examples.

```sh
make check
make pack
substreams auth
# Replace placeholders locally; sslmode=disable is for loopback development only.
export SUBSTREAMS_SINK_DSN='postgres://USER:PASSWORD@127.0.0.1:5432/punks?sslmode=disable'
substreams sink postgres setup itsjerryokolo-cryptopunks-v0.2.0.spkg
substreams sink postgres itsjerryokolo-cryptopunks-v0.2.0.spkg \
  -s 3914494 -t 3924494 --final-blocks-only --batch-block-flush-interval=1
```

The stop block is exclusive. `setup` creates the schema and system tables; use it
once on a fresh database, not as a schema migration. The sink accepts `postgres://`
or `psql://`; do not use `postgresql://` here. Do not use the older standalone sink.

Run the same sink command again to resume; the saved cursor takes precedence over
the supplied start. Increase `-t` to continue farther. Do not delete or manually
advance the cursor. Rebuild/reindex into a new database if the module hash changes;
do not force a mismatched package onto existing history.

```sql
SELECT id, block_num FROM cursors;
SELECT * FROM sales WHERE token_id IN (0, 544, 3134) ORDER BY block_number, ordinal;
SELECT * FROM current_ownership WHERE token_id = 544;
SELECT * FROM daily_market_summary ORDER BY day;
SELECT * FROM current_bids WHERE is_open;
```

For the independent wrapping test, set up **another fresh database**, then select
`db_out_ownership` and blocks 10951800 through 10951859. That stateless output avoids
rebuilding unrelated bid history, and deliberately does not output prices/bids:

```sh
substreams sink postgres substreams.yaml db_out_ownership \
  -s 10951800 -t 10951860 --final-blocks-only --batch-block-flush-interval=1
```

## Reproduce checks

Configure standard `PGHOST`, `PGPORT`, `PGUSER`, `PGDATABASE`, `PGPASSWORD` variables
for psql. Set `PSQL` to its absolute executable path if it is not on PATH.

```sh
psql -X -f tests/sql_views.sql
# Receipts-only mode also checks all aggregate fields and latest bid snapshots:
python3 scripts/verify_sql.py --receipts /path/to/raw-market-events.jsonl
python3 scripts/verify_sql.py /path/to/resolved-sales.jsonl --receipts /path/to/raw-market-events.jsonl
python3 scripts/verify_ownership.py /path/to/raw-market-events.jsonl
# Against the separate wrapped test database:
python3 scripts/verify_wrapped_sql.py /path/to/native-transfers.jsonl /path/to/wrapped-transfers.jsonl
```

The sales and native-ownership captures must cover the same range as the database.
`verify_sql.py` also checks the three observed official-site sales in
`tests/fixtures/official-website-sales.json`. That fixture records a human-readable
website observation, not a continually refreshed website oracle. Re-open its
source URLs when refreshing evidence.

The SQL regression script creates a temporary schema inside a transaction and
rolls it back. It checks exact wei-scale amounts, zero/self-sales, UTC grouping,
same-block wrapping order, unwrapping and deletion of later facts. This is **not**
an induced Substreams chain-reorg test. Live runs used finalized historical blocks;
provider undo delivery remains untested.
