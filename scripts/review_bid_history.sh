#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
cli="${SUBSTREAMS:-substreams}"
output_dir="${1:-$(mktemp -d "${TMPDIR:-/tmp}/cryptopunks-bids.XXXXXX")}"
mkdir -p "$output_dir"
output_dir="$(cd "$output_dir" && pwd)"
package="itsjerryokolo-cryptopunks-v0.2.0.spkg"
block_count="${BLOCK_COUNT:-30000}"
[[ "$block_count" =~ ^[1-9][0-9]{0,5}$ ]] || { echo 'BLOCK_COUNT must be a positive integer with at most six digits' >&2; exit 2; }
# Keep ad-hoc runs bounded; increase deliberately after reviewing expected cost.
(( block_count <= 100000 )) || { echo 'BLOCK_COUNT must be at most 100000' >&2; exit 2; }
receipt_limit=$((block_count + 1000))
state_limit=$((block_count * 3 + 10000))
# A bounded replay from the real initial block, including store preparation.
# Authenticate locally first. These requests consume streaming quota.
"$cli" run https://spkg.io/v1/packages/ethereum-common/v0.3.3 filtered_events \
  --network mainnet -p 'filtered_events=evt_addr:0xb47e3cd837ddf8e4c57f05d70ab865de6e193bbb' \
  -s 3914494 -t "+$block_count" -o jsonl --max-retries 0 --limit-processed-blocks "$receipt_limit" \
  > "$output_dir/receipts.jsonl" 2> "$output_dir/receipts.log"
for mode in development production; do
  args=(--max-retries 0)
  if [[ "$mode" == production ]]; then args+=(--production-mode); fi
  "$cli" run "$package" map_bid_changes -s 3914494 -t "+$block_count" -o jsonl \
    --limit-processed-blocks "$state_limit" "${args[@]}" \
    > "$output_dir/$mode.jsonl" 2> "$output_dir/$mode.log"
  python3 scripts/verify_bid_history.py "$output_dir/receipts.jsonl" "$output_dir/$mode.jsonl"
done
python3 - "$output_dir" <<'PY'
import json, sys
from pathlib import Path
p = Path(sys.argv[1])
a = [json.loads(x) for x in (p/'development.jsonl').read_text().splitlines()]
b = [json.loads(x) for x in (p/'production.jsonl').read_text().splitlines()]
assert a == b, 'development/production output differs'
print('Development and production outputs match exactly.')
PY
echo "Captures: $output_dir"
