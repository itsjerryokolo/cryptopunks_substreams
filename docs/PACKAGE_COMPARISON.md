# Comparison with cryptopunks@v0.1.3

Compared on 2026-09-28 against the actual [StreamingFast registry package](https://substreams.dev/packages/cryptopunks/v0.1.3), downloaded from `https://spkg.io/v1/packages/cryptopunks/v0.1.3`.

Package SHA-256: `1a333281a2d50f39f91bb5c550667d1a001273667b547f63120045d80839e61c`.

Evidence: CLI `info`, the unpacked manifest, extracted `contract.v1.Events` protobuf, and a live run of that exact package. The registry artifact contains compiled WASM, not original Rust source. This comparison is specific to v0.1.3 and does not claim anything about other packages or future releases.

## What each package provides

| Capability | StreamingFast cryptopunks v0.1.3 | This version |
|---|---|---|
| Original market events | One `map_events` with eight ABI event collections | Separate typed maps for assignments, bids, asks, sales, and native transfers; not a drop-in copy of the eight-collection envelope |
| Accepted-bid sale price/buyer | Raw `PunkBought` values; zero value/buyer observed in the example below | `map_resolved_sales` reconstructs price from prior bid history; buyer recovered from the preceding market Transfer |
| Current bid lifecycle | No store modules or current-bid output in the artifact | `map_bid_changes` / `bids_state`, including replacements, withdrawals, purchase/transfer refunds and stale-bid suppression |
| WrappedPunks and proxies | No wrapped-token/proxy output types or modules in the artifact | Wrapped mints, burns, ordinary transfers, and proxy registration maps |
| On-chain metadata | No token metadata output in `contract.v1.Events` | Traits, type, SVG data URI, raw RGBA pixels; inherited historical backfill schedule |
| Derived volume | No aggregation store in the artifact | `store_volume` per contract/day/Punk/buyer/seller; implemented, aggregate-specific acceptance still pending |
| Event position | Transaction hash, event index, block number/time | Those concepts plus block hash and Firehose ordinal for state ordering |
| Ready-made SQL output | `db_out` plus embedded SQL schema/sink configuration | `db_out` plus PostgreSQL schema; resolved trades, bid lifecycle and ownership facts with bounded ingestion/resume checks |
| Ownership ledger | No ownership store in the artifact | `map_ownership_changes` covers assignments/sales/native transfers/wrapped events; SQL separates native custody from wrapped holders |

Daily SQL metrics additionally provide sale counts, volume, min/max, average and distinct counterparties grouped by UTC day. See [SQL semantics and limitations](SQL_GUIDE.md).

The generic market `Transfer(address,address,uint256)` event is exposed by the reference package. Here it is used internally to recover accepted-bid buyers rather than offered as a separate output collection. These packages have different output contracts and monetary units; consumers must not interchange their schemas blindly.

## Verified accepted-bid example

Ethereum mainnet block **3919706**, Punk **544**, transaction `0xb28b5f2c186bf534e4fc4b8604b1496c9632e42269424f70ef1bdce61ea8ba52`:

| Output | Buyer | Amount |
|---|---|---|
| Reference `map_events.punkBoughts` | `0x0000000000000000000000000000000000000000` | `value = "0"` (raw wei event value) |
| This package `map_resolved_sales.sales` | `0x5b098b00621eda6a96b7a476220661ad265f083f` | `amount = "0.01"` ETH, `bidAccepted = true` |

The reference correctly exposes the original raw event. Our additional value is reconstructing the trade from related receipt events and earlier state, not relabeling the raw decoder as incorrect. The earlier bid is at block 3919689.

Reference reproduction:

```sh
substreams run cryptopunks@v0.1.3 map_events \
  -s 3919706 -t +1 -o jsonl --max-retries 0 --limit-processed-blocks 1000
```

This version requires bid history, so use `scripts/live_check.sh` from the real initial block instead of truncating its state. The bounded 30,000-block bid comparison is documented in `PR_REVIEW.md`.

## Positioning and next work

This version adds interpreted trading data, bid lifecycle state, WrappedPunks coverage, and on-chain metadata. These capabilities already exist in the branch; they are not a list of promised future features. PostgreSQL support now makes interpreted sales, bids and ownership queryable. Ownership history and UTC daily market/per-Punk metrics are now implemented and tested in bounded PostgreSQL runs.

The unused legacy `graph_out`/entity-output path was removed. Publishing the package and running its typed modules do not require GraphQL. Publication still requires the user's explicit confirmation after PR review.
