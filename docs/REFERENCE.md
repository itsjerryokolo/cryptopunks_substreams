# Module reference

The package reads Ethereum mainnet blocks and emits typed CryptoPunks records. The manifest defines the dependency graph and initial block for each module. The protobuf schema defines the fields. [Manifest](../substreams.yaml) · [Protobuf schema](../proto/cryptopunks.proto)

## Choose an output

| What you need | Module | Output |
|---|---|---|
| Trade prices and counterparties | `map_resolved_sales` | Direct sales and accepted bids, with the final price in ETH. |
| A sale from a single block | `map_sales` | The event amount and the buyer recovered from the receipt. Accepted bids require historical state for the price. |
| Bid activity | `map_bid_changes` | Entered, replaced, withdrawn and accepted bids, plus closures when ownership passes to the bidder. |
| Original market events | `map_assigns`, `map_bids`, `map_asks`, `map_transfers` | Assignments, explicit bid events, listing changes and native transfers. |
| WrappedPunks activity | `map_wrapped_transfers`, `map_user_proxies` | ERC-721 mints, transfers and burns, and proxy registrations. |
| Ownership history | `map_ownership_changes` | Assignments, sales, native transfers and wrapped events ordered within the block. Each row identifies the asset as `native` or `wrapped`. |
| Token metadata | `map_metadata` | Punk type, comma-separated traits, a 24 × 24 image encoded as hex RGBA bytes with a `0x` prefix, and an SVG data URI. |
| PostgreSQL rows | `db_out`, `db_out_ownership` | `DatabaseChanges` for sales, bids and ownership. The ownership-only output can run over a short window without building bid history. |

Stateful pricing and bids require history from block **3,914,494**. Requesting a recent output block can still trigger that backfill. For metadata, the package makes historical contract calls for one token per block from **13,047,091 through 13,057,090**, starting with Punk 9999 and ending with Punk 0. An incomplete or failed RPC batch returns an error.

## How prices and bids are resolved

The original market can emit `PunkBought` with a zero buyer and value when a seller accepts a bid. `map_sales` recovers the buyer from the preceding market `Transfer` in the transaction and sets `bid_accepted`. `map_resolved_sales` reads the active bid immediately before the sale and checks both Punk and bidder before using its amount. Missing or inconsistent bid history fails that lookup.

`store_bid_events` tracks the latest explicit bid event per Punk. `store_bid_resets` tracks the latest ownership change per Punk and recipient. Combining both at the event ordinal lets `map_bid_changes` close a bid when its bidder receives the Punk, even when the market emits no withdrawal event. A later bid can reopen normally. `bids_state` stores the latest snapshot under `Punk: <id>`. A closed snapshot retains its amount for history. Filter the latest snapshot on `open == "true"` for active bids.

Amounts in protobuf are decimal ETH strings. Order event records by `(block_number, ordinal)`. The Firehose `ordinal` orders events within one block. The SQL event key uses the transaction hash and `log_index`, the log's position in the block. `block_hash` identifies the block that produced the record.

## Other stores

| Store | Value |
|---|---|
| `store_sales`, `store_volume` | Latest resolved sale by Punk, buyer or seller, and cumulative ETH volume by contract, UTC day, Punk, buyer or seller. |
| `asks_state` | Listing snapshots updated by offers, removals and sales. |
| `store_assigns`, `contract_metadata` | Latest assignment by Punk or assignee, and contract metadata read at the assignment bootstrap in block 3,919,682. |
| `store_user_proxies`, `punk_state` | Proxy-to-user lookup, and the latest native transfer per Punk with proxy senders resolved on deposits into WrappedPunks. For full custody history use `map_ownership_changes`. |
| `store_metadata` | Token metadata keyed by Punk ID. |

## Migrating from v0.1.0

v0.2.0 changes module dependencies, filters, store keys and event ordinals. Start with an empty destination and replay from block 3,914,494 using fresh store state and a new cursor. Consumers that used `ordinal` as a log index should read `log_index`. Replace address-only bid lookups with per-Punk `bids_state` or build a bidder index from `map_bid_changes`.

The former `graph_out`, `map_*_entities` and `schema.graphql` integration was removed. Consumers can read the typed maps or use the PostgreSQL sink. [PostgreSQL guide](SQL_GUIDE.md)
