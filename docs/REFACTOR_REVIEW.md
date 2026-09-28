# CryptoPunks refactor review

Reviewed baseline: `1177dec` on `main`. Branch: `codex/cryptopunks-refactor`.

Guidance used: StreamingFast’s `substreams-dev`, `substreams-ethereum`, `substreams-testing`, and `substreams-sink` at revision `8ccccf24f6eeba1f1b1f4db3c0d9d0c95a548293` (skill pack v1.6.0). Instructions were applied to this existing pipeline rather than treating it as a new project. The implementation keeps the compatible legacy Rust runtime/protobuf family; it does not mix modern Substreams 0.7 with the old entity-change crate.

## Fixed in v0.2.0

| Problem | Change and evidence |
|---|---|
| Most handlers matched event signatures without checking the emitter. | Every decoder now checks the CryptoPunks or WrappedPunks contract. One regression sends all supported signatures from a foreign address. |
| Assigns in the bootstrap block were duplicated and contract RPC was repeated for each event. | Emit exactly one assignment per log and attach metadata once. Injected RPC regression checks both counts. |
| Address strings could omit or duplicate `0x`; wrapping compared incompatible formats. | Canonical lowercase addresses and byte-level contract comparisons. Proxy reads use the transfer’s ordinal. |
| Every store write used ordinal zero, or reads looked at end-of-block state. | Event-derived writes use Firehose ordinals; dependent reads use the relevant event position. The block log index has its own field. |
| Accepted bids can emit zero value and buyer from the original contract’s cleared storage alias. | Recover buyer from the market Transfer immediately preceding PunkBought; resolve amount against the prior bid and reject missing/inconsistent bid history. Direct sales never add an unrelated bid value. |
| Bid closures only ran when a bid event was present. | Sales drive closures independently of bid events; updates are sorted alongside subsequent bids. |
| Asks were only stored when a sale matched the transaction. | Store offers/removals independently and create closed snapshots for sales. |
| The ask’s caller was always assumed to be the transaction origin. | Resolve the caller of the successful emitting contract call; use origin only for a direct transaction to CryptoPunks. |
| Ordinary wrapped-token transfers were discarded. | Expose mints, burns, and transfers in the standalone wrapped-transfer map. |
| Assignment entities were emitted from both store aliases; relationship IDs, booleans, hashes, Sale fields, and required fields were inconsistent. | Emit one assignment from the punk key; encode relationship IDs as strings or Bytes as appropriate, real booleans, Bytes hashes, buyer, `type`, `blockHash`, metadata punk relation, and contract totalSupply. GraphQL amount fields now match ETH BigDecimal output. |
| The schema referenced undefined TraitType and a derived relation against a scalar string. | Remove the unused invalid Trait declaration. GraphQL syntax, types, and interfaces validate. This is not a live Graph Node validation. |
| SVG/image positions were reversed, pixels were concatenated decimal bytes, and RPC errors became metadata text. | Named metadata fields, hex raw pixels, and explicit Result errors for failed/incomplete RPC batches. |
| The package rescanned irrelevant blocks. | Pin `ethereum-common` v0.3.3 and attach address block filters to the seven event maps. Keep in-handler allowlists as well. |
| Code generation modified tracked source, protobuf generation was manual, native tests inherited a WASM target, and ethabi was duplicated. | Generate ABI/protobuf bindings into OUT_DIR; remove generated snapshots; make native tests the default and WASM explicit; use ethabi 17 consistently. |
| There were no tests or CI and several Make targets referenced nonexistent modules. | Add offline regressions, strict Clippy, a pinned toolchain, working bounded stream targets, and CI for tests/build. |

The accepted-bid behavior was checked against [the original CryptoPunks contract](https://github.com/larvalabs/cryptopunks/blob/master/contracts/CryptoPunksMarket.sol), specifically `acceptBidForPunk` and `buyPunk`.

## Remaining design and deployment work

1. **Choose a supported sink.** [Graph Node v0.42 release notes](https://github.com/graphprotocol/graph-node/blob/master/NEWS.md#v0420) state that Substreams support was removed. Publishing this `.spkg` is not the same as deploying a queryable subgraph. Options are a supported SQL/custom sink, or an explicitly selected older self-hosted Graph Node. The legacy `substreams.entity.v1.EntityChanges` contract has not been migrated or live-tested against a sink.
2. **Complete the entity model if GraphQL is retained.** `Account`, `Punk`, `MarketplaceDailySnapshot`, and `PunkDailySnapshot` remain schema declarations without writers. Relations to these entities will not resolve into a complete application. `store_volume` is available but not wired to entity output. The schema is valid, but schema validity does not establish complete data coverage.
3. **Define ownership and offer lifecycle semantics.** Native `PunkTransfer`, `PunkBought`, assignment, wrapped mint/burn, and wrapped transfers need a unified model. `map_wrapped_transfers` remains standalone, and `punk_state` is latest native-transfer state, not a full ownership ledger. Bid history records entered/withdrawn events; transfer-triggered bid refunds and historical stale bid closure snapshots need fuller state reconciliation before presenting these snapshots as current offers.
4. **Separate metadata backfill from live event processing.** The inherited metadata schedule fetches one token per block in 13047091–13057090. It intentionally cannot use event-address filtering, since those blocks may contain no relevant logs. Validate the data contract’s historical availability and sample metadata/trait formatting with real RPC responses, then consider a separately validated snapshot or metadata pipeline.
5. **Evaluate a coordinated SDK/protobuf migration separately.** This branch removes duplicate ethabi versions but retains Substreams 0.4 / Ethereum 0.7 / prost 0.11 for compatibility. Upgrade these together with the selected sink contract, followed by real-block and replay tests. No performance multiplier is claimed from the filters without measuring a real backfill.

## Validation record

- Baseline: built natively with zero tests.
- Refactor: offline regression suite passes; formatting and strict Clippy pass.
- Rust 1.90.0 release WASM build passes.
- Substreams CLI 1.23.0 packages the module graph successfully as `cryptopunks-v0.2.0.spkg`.
- GraphQL syntax/type/interface validation passes using the GraphQL library available with the local Graph CLI.
- Authentication subsequently succeeded. Live validation passed for assignments, direct sales, 37 historical resolved sales (six accepted bids), development/production output parity, 48 wrapped transfers, and two historical metadata records. See [LIVE_VALIDATION.md](LIVE_VALIDATION.md) for ranges, receipt evidence, limits, and reproduction commands.
- Live metadata validation exposed trait parsing defects: trailing commas, numeric accessory corruption, and dropped first accessories for nonhuman types. Fixed with a regression; 20 offline tests now pass.

## Next acceptance checks

A subsequent native-transfer capture returned three events in blocks 10951800–10951859. Continue with broader receipt sampling and a real same-block new bid after a sale. The bounded live checks above complement the offline ordering regressions; they do not establish complete full-history correctness.

After selecting a sink, use a **fresh** destination and replay from the required history. Query representative records and verify counts/amounts before scaling up. Do not reuse v0.1.0 store state or cursors.
