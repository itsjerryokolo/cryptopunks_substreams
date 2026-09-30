"""Verify bounded Substreams captures; see docs/LIVE_VALIDATION.md for commands."""
import base64
import json
import sys
from decimal import Decimal
from pathlib import Path

capture_dir = Path(sys.argv[1]) if len(sys.argv) > 1 else Path("/tmp")


def records(name):
    path = capture_dir / f"cryptopunks-live-{name}.jsonl"
    return [json.loads(line) for line in path.read_text().splitlines()]


def events(name, field):
    return [event for row in records(name) for event in row["@data"].get(field, [])]


assigns = events("assigns", "assigns")
assert len(assigns) == 1 and assigns[0]["tokenId"] == "1838"
assert assigns[0]["contract"]["totalSupply"] == "10000"

sales = events("resolved", "sales")
assert len(sales) == 37
assert sum(s.get("bidAccepted", False) for s in sales) == 6
assert records("resolved") == records("resolved-production"), "mode output mismatch"
assert all(Decimal(s["amount"]) > 0 for s in sales if s.get("bidAccepted"))

native = events("transfers-wrap", "transfers")
assert len(native) == 3
assert [t["tokenId"] for t in native] == ["4046", "9972", "9972"]
assert native[-1]["to"] == "0xb7f7f6c52f2e2fdb1963eab30438024864c313f6"

wrapped = events("wrapped", "transfers")
assert len(wrapped) == 48
assert any(t["from"] == "0x" + "0" * 40 for t in wrapped)
assert any(t["from"] != "0x" + "0" * 40 and t["to"] != "0x" + "0" * 40 for t in wrapped)

metadata = events("metadata-fixed", "metadatas")
assert [m["tokenId"] for m in metadata] == ["9999", "9998"]
assert metadata[0]["traits"] == "Mohawk,Nerd Glasses"
assert metadata[1]["traits"] == "Black Lipstick,Wild White Hair,Clown Eyes Green"
for m in metadata:
    assert len(bytes.fromhex(m["image"][2:])) == 24 * 24 * 4
    assert m["svg"].startswith("data:image/svg+xml;utf8,<svg")

fixture = Path(__file__).resolve().parents[1] / "tests/fixtures/mainnet-sale-10919768.json"
raw = json.loads(fixture.read_text())
receipt = next(e for e in raw["@data"]["events"] if e["log"].get("blockIndex") == 45)
log = receipt["log"]
topics = [base64.b64decode(t) for t in log["topics"]]
sale = next(s for s in events("sales", "sales") if s["trxHash"][2:] == receipt["txHash"])
assert int(sale["tokenId"]) == int.from_bytes(topics[1], "big") == 4662
assert sale["from"] == "0x" + topics[2][-20:].hex()
assert sale["to"] == "0x" + topics[3][-20:].hex()
assert Decimal(sale["amount"]) == Decimal(int.from_bytes(base64.b64decode(log["data"]), "big")) / Decimal(10**18) == Decimal("3.1")
assert sale["ordinal"] == log["ordinal"]
assert sale["logIndex"] == log["blockIndex"]
assert sale["blockHash"][2:] == raw["@data"]["clock"]["id"]
print("Live checks passed: assignments, 37 resolved sales (6 accepted bids), mode parity, 48 wrapped transfers, metadata, and receipt comparison.")
