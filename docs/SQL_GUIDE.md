# Query CryptoPunks with PostgreSQL

`db_out` writes sales, bid changes and ownership history to PostgreSQL. The following example indexes the first 10,000 blocks of market history and retrieves the accepted bid for Punk 544. [Schema and view definitions](../schema.sql)

## Index the sample

Use a fresh local database, a role with permission to create tables and views, PostgreSQL 14 or later, and Substreams CLI 1.23.0. Run from the repository root. Replace the example connection string with the one for your local database. The unencrypted connection shown here is for loopback use.

```sh
export SUBSTREAMS_SINK_DSN='postgres://USER:PASSWORD@127.0.0.1:5432/punks?sslmode=disable'
make pack
substreams auth
package=itsjerryokolo-cryptopunks-v0.2.0.spkg
substreams sink postgres setup "$package"
substreams sink postgres "$package" \
  -s 3914494 -t 3924494 --final-blocks-only --batch-block-flush-interval=1
psql "$SUBSTREAMS_SINK_DSN"
```

`setup` creates the schema once. The stop block is exclusive. Rerun the sink command with a larger stop block to continue from the saved cursor. If the module hash changes, replay into a fresh database. The recorded 100,000-block check used `-t 4014494`. [Validation](VALIDATION.md)

## Query the result

```sql
SELECT token_id, amount_eth,
       CASE WHEN bid_accepted THEN 'True' ELSE 'False' END AS bid_accepted
FROM sales
WHERE token_id = 544 AND block_number = 3919706;
```

| token_id | amount_eth | bid_accepted |
|---:|---:|:---|
| 544 | 0.01 | True |

```sql
SELECT day, sales_count, volume_eth FROM daily_market_summary ORDER BY day;
SELECT token_id, bidder, amount_eth FROM current_bids WHERE is_open ORDER BY token_id;
SELECT native_owner, wrapped_holder, effective_holder FROM current_ownership WHERE token_id = 544;
SELECT id, block_num FROM cursors;
```

- **Bids.** `current_bids` returns the latest bid change per Punk at the processed cursor. Closed rows retain the previous amount. Filter on `is_open` for active offers.
- **Volume.** The daily market and per-Punk views sum exact decimal ETH amounts for original-market sales, including zero-price sales and sales to the same address. They group by UTC day. Only dates with sales appear. External marketplace trades and off-chain orders are outside their coverage.
- **Ownership.** `native_owner` can be the WrappedPunks contract. During wrapper custody, `wrapped_holder` supplies the effective holder when known. The view returns `NULL` for the effective holder when the wrapped holder is unknown. An ownership-only window with no observed native custody can omit a Punk from this view, even when its wrapped transfer appears in `ownership_history`.

## Wrapped ownership

To examine wrapping without replaying bid history, exit `psql` with `\q` and point `SUBSTREAMS_SINK_DSN` at a **second empty database**. Run the schema setup for that database and use `db_out_ownership` to index a 60-block window.

```sh
substreams sink postgres setup "$package"
substreams sink postgres substreams.yaml db_out_ownership \
  -s 10951800 -t 10951860 --final-blocks-only --batch-block-flush-interval=1
psql "$SUBSTREAMS_SINK_DSN"
```

```sql
SELECT token_id, native_owner, wrapped_holder, effective_holder
FROM current_ownership
WHERE token_id IN (4046, 9972) ORDER BY token_id;
```

At this cutoff Punk 4046 has returned to its native owner and `wrapped_holder` is `NULL`. Punk 9972 is in wrapper custody and its effective holder is the wrapped token holder. These views describe ownership through the sink's processed cursor.
