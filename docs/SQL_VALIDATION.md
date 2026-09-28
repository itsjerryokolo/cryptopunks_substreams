# PostgreSQL and official website validation

Observed 2026-09-28, Substreams CLI 1.23.0, PostgreSQL 14.19, Ethereum mainnet.
All runs were local, bounded and used finalized historical blocks. Nothing was
published. This is a sampled acceptance record, not a guarantee for every block.

## End-to-end ingestion

1. Compiled WASM and packed the module with the embedded SQL schema.
2. Created an isolated loopback-only PostgreSQL test cluster and fresh databases.
3. Ran `sink postgres setup`, then `db_out` for [3914494,3924494).
4. Repeated the command: the sink read its saved cursor, wrote zero additional
   rows, and exited successfully. No duplicate facts appeared.
5. Resumed to stop block 3944494 and compared the resulting database against
   a separately captured `map_resolved_sales` stream and an independent Python
   ABI decoder/bid reducer of raw market receipts for [3914494,3944494).
6. Resumed the final packed artifact after removing the conflicting legacy SQL
   descriptor import; cursor/module compatibility and normal JSON decoding passed.

| Check | Result |
|---|---|
| Resolved sales, all values/parties and event identities | 128 matched |
| Reconciled bid changes | 2,655 matched |
| Native ownership events | 10,273 matched |
| Native owners at the bounded cutoff | All 10,000 matched receipt replay |
| UTC market-day counts and exact ETH totals | All 6 matched |
| PostgreSQL restart | No duplicate rows; extension resumed correctly |
| Offline Rust tests / strict Clippy / WASM | 25 tests passed; build/check passed |
| SQL view regressions | Passed, including wei precision and deletion of later facts |

Bid cases: 1,859 new bids, 373 replacements, 372 withdrawals, 46 accepted bids,
and five direct purchases that refunded the buyer's previous bid. The same-Punk
same-block and transfer-to-bidder refund cases remain offline coverage; they were
not encountered in this historical range.

Raw receipts and Rust outputs use the same provider. The independently written
ABI/reducer check catches interpretation differences, but is not a second-provider
chain check. The website below is an additional external presentation check.

Daily volume (ETH):

| UTC date | Sales | Exact volume |
|---|---:|---:|
| 2017-06-23 | 20 | 3.104 |
| 2017-06-24 | 22 | 4.2320000004 |
| 2017-06-25 | 13 | 1.796740178513258200 |
| 2017-06-26 | 18 | 2.428793350712529643 |
| 2017-06-27 | 36 | 7.985475918637670024 |
| 2017-06-28 | 19 | 5.484673563671065333 |

The first/last dates are bounded by the block range, so these are totals of indexed
facts on each date, not a claim that every calendar day is completely indexed.

## Official CryptoPunks website

Read rendered **Sold** rows and their full account/transaction link URLs in the
browser. Compared transaction hash, Punk ID, buyer, seller, UTC date and displayed
ETH amount against PostgreSQL. All three were June 23, 2017 transactions:

| Punk | Website ETH | PostgreSQL ETH | Chain classification |
|---|---:|---:|---|
| [544](https://www.cryptopunks.app/cryptopunks/details/544) | 0.01 | 0.01 | Accepted bid |
| [0](https://www.cryptopunks.app/cryptopunks/details/0) | 0.98 | 0.98 | Accepted bid |
| [3134](https://www.cryptopunks.app/cryptopunks/details/3134) | 0.01 | 0.01 | Direct purchase |

Full addresses, transaction hashes, observation date and source URLs are committed
in `tests/fixtures/official-website-sales.json`. `scripts/verify_sql.py` checks those
records against SQL. It does not pretend to download fresh website observations.
No USD conversion, current website bid, or present-day owner was compared to this
2017 snapshot. These three exact displayed values match; arbitrary website display
rounding would need a separate tolerance rule rather than rounding the stored data.

## Wrapped ownership

A separate database ingested `db_out_ownership` for [10951800,10951860). It contains
10 ownership facts (six native/wrapped transfer events plus four native sales).
The six transfer events match separately captured typed maps field for field:

- Punk 4046: wrapped burn then native transfer at block 10951812. Effective holder
  is `0xd387a6e4e84a6c86bd90c158c6028a58cc8ac459`; wrapped holder is NULL.
- Punk 9972: native deposit then wrapped mint at block 10951859. Native owner is
  the wrapper; wrapped/effective holder is
  `0x7b1d4f3602f0fe16c4f611e124d3d7b307245ba8`.
- Punk 7263: ordinary wrapped transfer is retained in history. Its earlier native
  deposit is outside the bounded window, so it is deliberately absent from the
  combined current-ownership view in this partial database.

## Review fix and limits

The skill's suggested SQL protodefs-v1.0.7 import duplicates deprecated Service
messages bundled by CLI 1.23.0. This caused normal `run -o jsonl` decoding to fail,
even though SQL ingestion and packing succeeded. Removed that legacy import;
the modern SQL service type is supplied by the pinned CLI. Final JSON output and
packed SQL sink resume were then exercised.

The database-change protobuf is an unchanged copy from the official v4.0.0 source,
compiled using this repository's existing prost version. See `proto/THIRD_PARTY.md`.

No full chain-head backfill, live induced reorg, hosted deployment, or complete
metadata backfill was performed. SQL deletion regressions verify view behavior,
not delivery of Substreams undo signals. See [SQL_GUIDE.md](SQL_GUIDE.md) for scope,
reproduction commands, cursor handling and safe fresh-database migration.
