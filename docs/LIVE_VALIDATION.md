# Live validation and deployment walkthrough

Validated on Ethereum mainnet on 2026-09-28 with Substreams CLI 1.23.0.
Authentication uses the local, ignored `.substreams.env`; no credentials belong in this record.

## Stage 1: validate and publish the package

The package name is `itsjerryokolo_cryptopunks`, registry slug `itsjerryokolo-cryptopunks`, version `v0.2.0`. The unscoped `cryptopunks` package is owned by StreamingFast, so this fork uses its own name. The Cargo crate name remains `cryptopunks`.

| Check | Requested range (inclusive) | Result |
|---|---|---|
| Assignment and contract RPC | 3919682 | One assignment, Punk 1838; contract name CRYPTOPUNKS, symbol Ͼ, total supply 10,000. |
| Direct sales | 10919494–10920493 | Three sales. Punk 4662 at block 10919768 has amount 3.1 ETH; buyer, seller, transaction/block hash, ordinal and log index match the captured raw receipt. |
| Native transfers | 10951800–10951859 | Three transfers: Punk 4046 out of WrappedPunks, and Punk 9972 through its proxy into WrappedPunks. |
| Wrapped transfers | 10951736–10952735 | 48 transfers in 47 blocks, including mints and ordinary transfers. |
| Resolved sales, development | 3914494–3924493 | 37 sales in 36 blocks; six accepted bids have nonzero prices recovered from historical bid state. Punk 544 at block 3919706 resolves to 0.01 ETH. |
| Resolved sales, production | Same range | Parsed JSON output exactly matches development mode, including event ordering and amounts. |
| Historical metadata RPC | 13047091–13047092 | Punks 9999 and 9998; valid SVG data URI and 2,304-byte RGBA image each. Corrected trait parsing was rerun live. |

The native transfer windows 12292922–12293221 and 3919682–3919781 returned no events; those successful requests do not establish transfer correctness. The nonempty wrap-window check above and offline regressions cover native decoding.

Production mode initially rejected the 10,000 processing-block guard because it estimated 22,000 blocks including store preparation. The bounded retry used a 23,000 guard and reported 11,529 processed blocks. The development replay reported 10,506 processed blocks for 10,000 requested blocks. Requested ranges and provider processing usage differ; these are observations, not a cost estimate for a full backfill. More than two simultaneous streams were rejected by this account, so the repeatable script runs sequentially.

### Reproduce

Use the current CLI rather than the older globally installed 1.1.x version:

```sh
export SUBSTREAMS=/tmp/cryptopunks-substreams-cli/substreams
DEVELOPER_DIR=/Library/Developer/CommandLineTools make check pack
"$SUBSTREAMS" auth
bash scripts/live_check.sh
```

On other systems use the installed CLI path and omit the macOS `DEVELOPER_DIR` override. These commands consume streaming quota. Each request has an explicit range, processing limit and no automatic retries. Captures go to a temporary directory; `scripts/verify_live.py <directory>` checks them without network access.

`tests/fixtures/mainnet-sale-10919768.json` contains the market-only receipt events captured through `eth_common:all_events` v0.3.3. Protobuf byte fields remain base64. The verifier decodes the raw ABI topics/data independently of this project's sale decoder. This is a receipt comparison using the same provider, not a second-provider consensus check.

The live checks do not prove a complete ownership ledger, every metadata record, full-history bid lifecycle correctness, reorg recovery, or sink ingestion. The remaining limitations in [REFACTOR_REVIEW.md](REFACTOR_REVIEW.md) still apply.

### Publish

```sh
"$SUBSTREAMS" registry verify itsjerryokolo-cryptopunks-v0.2.0.spkg
"$SUBSTREAMS" registry login
"$SUBSTREAMS" publish itsjerryokolo-cryptopunks-v0.2.0.spkg --yes
```

Registry login is separate from streaming authentication. Publishing registers a downloadable package; it does not start a continuously running indexer or create a database. After publication, stream a known sale using the registry package reference to confirm the uploaded artifact works.

## Stage 2: PostgreSQL

Next, create a fresh local database and configure the supported `substreams sink postgres` command. Inspect CLI 1.23.0's relational protobuf mapping mode before deciding whether a dedicated `DatabaseChanges` module is needed. Begin with a bounded sale dataset, query records and exact decimal values, then check resume/cursor behavior. Add a reproducible local deployment and query walkthrough. Docker is installed on this machine but its daemon was not running during the preflight.

## Stage 3: legacy Graph Node

Use an isolated, explicitly pinned pre-v0.42 Graph Node; current Graph Node removed Substreams support. First migrate the entity output to the protobuf contract expected by that version and supply a complete deployable subgraph manifest/schema. The existing legacy graph output and incomplete entity relationships are not ready to deploy as-is. Verify representative entities and a restart before treating the demonstration as working. This stage is a legacy learning exercise, not a supported Subgraph Studio deployment.

## PR review follow-up

The bid-state review added recipient reset history, transfer/purchase refund handling, and a canonical per-Punk bid lookup. The 30,000-block historical comparison covers 2,655 changes, including replacements, withdrawals, accepted bids and purchase refunds. See [PR_REVIEW.md](PR_REVIEW.md) for findings, exact coverage boundaries, and the separate publication-confirmation requirement.
