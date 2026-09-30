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

## Additional validation — 2026-09-30

Revalidated branch `codex/cryptopunks-refactor` after the September 28 checks.
No production-code defect was found in these additional samples. Added a real
receipt-based Rust regression, stronger SQL assertions and repeatable validators.

### Expanded historical range and persistent database

Expanded the bounded replay to **[3914494,4014494)**: 100,000 blocks. The saved
local PostgreSQL database resumed from its September 28 cursor and caught up.
Independently decoded receipts matched:

- **572 sales**, including exact wei-to-ETH prices and counterparties.
- **4,909 bid changes**: 2,592 new bids, 1,295 replacements, 856 withdrawals,
  152 accepted bids, and 14 purchase refunds.
- **2,292 latest bid snapshots**, including closed state and bidder/Punk isolation.
- **10,769 native ownership events** and all **10,000 native owners** at the cutoff.
- **20 market-day summaries**: counts, volume, min/max, PostgreSQL-rounded average,
  distinct buyers and distinct sellers. Also **565 per-Punk/day summaries**.

Daily results are totals of indexed facts within the range; boundary days need
not be complete calendar days. Development and production bid output matched
field-for-field over all 4,909 changes. Development reported 104,317 processed
blocks; the production run used cached inputs (202,000 cached, zero newly processed).
This is correctness evidence, not a general performance/cost benchmark.

**New live ordering case:** block **4009734**, Punk **4936**, the same bidder raises
its bid from **0.05 to 0.1 ETH** across separate transactions at ordinals **1076**
and **1302**, with another Punk's bid in between. The receipt capture is committed
as `tests/fixtures/mainnet-bid-ordering-4009734.json`; its bid-log subset now has a
Rust regression. This closes the earlier same-block-bid-change coverage gap.
A transfer-to-bidder refund still was not encountered in the live sample.

### Additional website and image checks

Read the official website again on September 30 and verified these on-chain
metadata outputs:

| Punk | Type | Attributes |
|---|---|---|
| [5822](https://www.cryptopunks.app/cryptopunks/details/5822) | Alien | Bandana |
| [4156](https://www.cryptopunks.app/cryptopunks/details/4156) | Ape | Bandana |
| [8857](https://www.cryptopunks.app/cryptopunks/details/8857) | Zombie | Wild Hair, 3D Glasses |
| [0](https://www.cryptopunks.app/cryptopunks/details/0) | Female | Earring, Blonde Bob, Green Eye Shadow |

For each token, all **576 pixels** in the returned SVG agree with its raw RGBA
output, including alpha. Token zero also exercises the end of the metadata window.
This checks representation consistency; it is not a screenshot comparison.

Punk **5822's 8,000 ETH sale**, February 12, 2022, transaction
`0xd7cb135a789ed54cabab54ea3d5a30ad907f51e1b7846981980ada8478facfb7`,
matched the official website by value, seller, buyer, date and transaction hash.
[Etherscan](https://etherscan.io/tx/0xd7cb135a789ed54cabab54ea3d5a30ad907f51e1b7846981980ada8478facfb7)
confirmed block **14193462** and the direct-purchase receipt. This stateless sale
check does not claim a full 2022 bid-state backfill.

### Recent finalized sample and separate provider

Used a public, credential-free RPC to discover a finalized cutoff and streamed
**[26089381,26090381)**, a 1,000-block window observed on September 30. It produced
one ownership change: Punk **4201**, native transfer at block **26089666**,
transaction `0x37e3db1c862ce830a38519415ea1d11d223f7b8f384b6f05b752e14ff6cce955`.

The successful transaction receipt from `https://ethereum-rpc.publicnode.com`
matched the contract, token ID, sender, recipient, transaction hash, block hash,
block number and log index. The [official site](https://www.cryptopunks.app/cryptopunks/details/4201)
also displayed that September 30 transfer and recipient
`0x19df2bdd01aad26e4c27668808deb0540245cca3` as owner when observed.
The external receipt is stored in `tests/fixtures/mainnet-transfer-26089666.json`.

The public provider's bulk `eth_getLogs` requests failed with connection resets,
so this is **second-provider verification of the observed transaction**, not a
proof that no events are missing from the entire recent window. No company RPC
credentials or company infrastructure were used. Historical replay comparisons
still use StreamingFast receipts and a separately written interpretation.

### Offline checks and reproduction

**26 Rust tests**, strict Clippy, formatting, WASM build and package creation pass.
Expanded PostgreSQL tests pass for rewrapping after burn, partial custody history,
current-bid ordering across blocks, deletion restoring earlier bids, empty views,
UTC midnight boundaries, distinct counterparties and full uint256-scale ETH plus
one wei. Assertions now reject unexpected SQL NULL results explicitly.

```sh
SUBSTREAMS=/path/to/substreams BLOCK_COUNT=100000 bash scripts/review_bid_history.sh
SUBSTREAMS=/path/to/substreams bash scripts/additional_samples.sh
# Against the matching extended local SQL database, using standard PG* variables:
python3 scripts/verify_sql.py --receipts /path/to/receipts.jsonl
python3 scripts/verify_ownership.py /path/to/receipts.jsonl
psql -X -f tests/sql_views.sql
```

The additional-samples script uses fixed recorded blocks and website observations;
it does not claim to refresh the website or move its cutoff on subsequent runs.
No registry publication, merge, hosted deployment, full head backfill or induced
Substreams undo/reorg test was performed. Those release boundaries remain.
