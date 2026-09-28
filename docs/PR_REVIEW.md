# PR review and release boundaries

PR: https://github.com/itsjerryokolo/cryptopunks_substreams/pull/1

## Publication approval

Do not publish automatically. The user explicitly requires confirmation again immediately before publication. Approval to review, test, authenticate, or merge is not publication approval. Keep the PR review and publishing decisions separate. No package has been published.

## Findings fixed in this review

1. **Transfer-triggered bid refunds were missing.** `transferPunk` silently clears/refunds a bid if the recipient is the bidder. `store_bid_resets` now records native-transfer and sale recipients. `map_bid_changes` emits the matching closure even without a `PunkBidWithdrawn` event.
2. **Previously cleared raw bids could close again.** Raw entered/withdrawn events do not record all ownership-triggered clears. Active-bid lookup now compares the last raw bid with its Punk/recipient reset marker before the current ordinal. Comparing `(block_number, ordinal)` also handles ordinals restarting in each block. A later new bid reopens normally.
3. **Address-only bidder keys conflated different Punks and retained replaced bids.** `bids_state` now contains canonical `Punk: <id>` snapshots only. Consumers should use `map_bid_changes` to build their own bidder/Punk index. This is a breaking lookup change in the unreleased v0.2.0 and requires fresh state.

The reset store depends on raw sales/transfers, not resolved sales, so there is no circular dependency. Resolution reads the state before the sale; the sale's own reset cannot invalidate the bid being accepted. Reset markers for another recipient do not cancel an unrelated standing bid.

## What this PR supports

- Contract-filtered raw assignments, bids, asks, sales, native/wrapped transfers, proxy events, and scheduled metadata.
- Resolved accepted-bid sale prices, canonical per-Punk bid state, and listing-change snapshots.
- Repeatable offline checks, bounded live receipt comparisons, and development/production comparisons.

Historical Bid/Ask protobuf messages are event/change records. Their `open` value describes that event, not whether that historical record is still the current offer. Closed bid snapshots preserve the prior amount for audit; active queries must filter on `open`.

## What needs a separate implementation before the corresponding deployment

- A complete ownership model combining initial assignments, native sales/transfers, wrapping, unwrapping, and ERC-721 transfers. `punk_state` is only latest native-transfer state.
- Validated daily metrics and reporting tables, if added. The old unpopulated account/summary schema and entity converters were removed from this package.
- PostgreSQL ingestion, cursor resume, restart/reorg behavior, and query acceptance tests.

These are not needed to inspect raw event streams or resolved sales. They are required before advertising the corresponding full database application. The release focuses on typed Substreams outputs, with PostgreSQL as the next deployment stage.

## Review evidence

- 23 retained offline regression tests, including transfer refunds, unrelated-recipient transfers, stale-bid suppression, reopening after refunds, cross-block ordinals, replacements/withdrawals, and same-block ordering.
- Historical range 3914494–3944493: 2,655 bid changes compared field-for-field against an independent chronological reducer of the original market receipt logs. Coverage: 373 replacements, 372 withdrawals, 46 accepted bids, and five purchases refunding the buyer's existing bid (plus 1,859 new bids).
- The reducer models the contract's single highest bid, not a mock Substreams store. Both the Rust pipeline and reference reducer consume successful real-chain events through the same provider; this is not second-provider verification.
- Development and production output match exactly across all 2,655 bid changes. Development reported 31,624 processed blocks; production estimated 62,000 and reported 22,180 with cached inputs. These are observed processing counts, not a general cost estimate.
- The original 37 resolved sales remain unchanged after the reset-history fix.
- A transfer-to-bidder refund and multiple changes to the same Punk within one block were not found in this historical sample. Those specific scenarios are covered by offline regressions, not claimed as live coverage.

Run `bash scripts/review_bid_history.sh` to repeat the bid receipt and mode comparison. See `LIVE_VALIDATION.md` for the smaller baseline checks and cost guards. Passing bounded tests does not prove correctness for every historical/future block or for untested sinks.

## Stream-focused scope update

Removed the unused `graph_out` and seven entity-converter modules, the entity-change crate/import, `schema.graphql`, and their three converter-specific tests. The remaining event/store dependency graph does not require them. This supersedes the earlier legacy Graph Node deployment plan. See [PACKAGE_COMPARISON.md](PACKAGE_COMPARISON.md) for the evidence-backed package comparison.
