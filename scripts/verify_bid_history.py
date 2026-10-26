"""Compare live bid changes against a chronological CryptoPunks receipt replay.

This models the contract's single highest bid per Punk. It does not emulate the
Substreams store engine, rollback, or cursor handling. Amounts remain integer wei.
Usage: python3 scripts/verify_bid_history.py RECEIPTS.jsonl CHANGES.jsonl
"""
import base64
import collections
import json
import sys
from decimal import Decimal, localcontext
from pathlib import Path

SIGNATURES = {
    "5b859394fabae0c1ba88baffe67e751ab5248d2e879028b8c8d6897b0519f56a": "bid",
    "6f30e1ee4d81dcc7a8a478577f65d2ed2edb120565960ac45fe7c50551c87932": "withdraw",
    "58e5d5a525e3b40bc15abaa38b5882678db1ee68befd2f60bafe3a7fd06db9e3": "sale",
    "05af636b70da6819000c49f85b21fa82081c632069bb626f30932034099107d8": "transfer",
    "ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef": "erc20_transfer",
}
ZERO = "0x" + "0" * 40
MARKET = "b47e3cd837ddf8e4c57f05d70ab865de6e193bbb"


def rows(path):
    return [json.loads(line) for line in Path(path).read_text().splitlines()]


def replay(receipts):
    active = {}
    last_move = {}
    last_bid_event = {}
    coverage = collections.Counter()
    examples = {}
    expected = []
    for row in receipts:
        block = row["@block"]
        preceding_transfer = {}
        for event in row["@data"]["events"]:
            log = event["log"]
            assert base64.b64decode(log["address"]).hex() == MARKET
            topics = [base64.b64decode(t) for t in log["topics"]]
            data = base64.b64decode(log.get("data", ""))
            kind = SIGNATURES.get(topics[0].hex())
            tx = event["txHash"]
            token = int.from_bytes(data if kind == "transfer" else topics[1], "big")
            change = None
            case = None
            if kind == "erc20_transfer":
                preceding_transfer[tx] = "0x" + topics[2][-20:].hex()
            elif kind == "bid":
                bidder = "0x" + topics[2][-20:].hex()
                value = int.from_bytes(data, "big")
                case = "replacement" if token in active else "new_bid"
                assert value > active.get(token, (None, 0))[1]
                if last_move.get(token) == block:
                    coverage["same_block_move_then_bid"] += 1
                    examples.setdefault("same_block_move_then_bid", (block, token, tx))
                if last_bid_event.get(token) == block:
                    coverage["same_block_bid_changes"] += 1
                    examples.setdefault("same_block_bid_changes", (block, token, tx))
                last_bid_event[token] = block
                active[token] = (bidder, value)
                change = (token, bidder, value, True)
            elif kind == "withdraw":
                bidder = "0x" + topics[2][-20:].hex()
                value = int.from_bytes(data, "big")
                if last_bid_event.get(token) == block:
                    coverage["same_block_bid_changes"] += 1
                    examples.setdefault("same_block_bid_changes", (block, token, tx))
                last_bid_event[token] = block
                assert active.pop(token) == (bidder, value)
                case = "withdrawal"
                change = (token, bidder, value, False)
            elif kind in ("sale", "transfer"):
                buyer = "0x" + topics[3 if kind == "sale" else 2][-20:].hex()
                accepted = kind == "sale" and buyer == ZERO
                if accepted:
                    buyer = preceding_transfer[tx]
                    assert token in active and active[token][0] == buyer
                last_move[token] = block
                if token in active and active[token][0] == buyer:
                    _, value = active.pop(token)
                    case = "transfer_refund" if kind == "transfer" else "accepted_bid" if accepted else "purchase_refund"
                    change = (token, buyer, value, False)
            if case:
                coverage[case] += 1
                examples.setdefault(case, (block, token, tx))
            if change:
                expected.append((*change, block, log.get("blockIndex", 0), int(log["ordinal"]), "0x" + tx))
    return expected, coverage, examples


def verify(receipts_path, changes_path):
    expected, coverage, examples = replay(rows(receipts_path))
    observed = []
    with localcontext() as ctx:
        ctx.prec = 100
        for row in rows(changes_path):
            for bid in row["@data"].get("bids", []):
                wei = Decimal(bid["amount"]) * Decimal(10**18)
                assert wei == int(wei)
                observed.append((int(bid.get("tokenId", 0)), bid["from"], int(wei), bid["open"] == "true", int(bid["blockNumber"]), bid.get("logIndex", 0), int(bid["ordinal"]), bid["trxHash"]))
    assert observed == expected, f"Mismatch: expected {len(expected)} changes, observed {len(observed)}; first mismatches: {[(a,b) for a,b in zip(expected, observed) if a != b][:3]}"
    assert coverage["replacement"] and coverage["withdrawal"] and coverage["accepted_bid"] and coverage["purchase_refund"], coverage
    print(f"All {len(expected)} bid changes match raw receipts: {dict(coverage)}")
    print(f"Example (block, Punk, transaction): {examples}")


if __name__ == "__main__":
    verify(*sys.argv[1:])
