#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
cli="${SUBSTREAMS:-substreams}"
package="itsjerryokolo-cryptopunks-v0.2.0.spkg"
output_dir="${1:-$(mktemp -d "${TMPDIR:-/tmp}/cryptopunks-live.XXXXXX")}"
mkdir -p "$output_dir"
output_dir="$(cd "$output_dir" && pwd)"
# Authenticate with `substreams auth` first. The CLI reads .substreams.env.
run_check() {
  local label="$1" module="$2" start="$3" count="$4" limit="$5"
  shift 5
  echo "Checking $label (start $start, count $count)"
  "$cli" run "$package" "$module" -s "$start" -t "+$count" \
    -o jsonl --max-retries 0 --limit-processed-blocks "$limit" "$@" \
    > "$output_dir/cryptopunks-live-$label.jsonl" \
    2> "$output_dir/cryptopunks-live-$label.log"
}
# Sequential streams respect accounts with a two-stream concurrency limit.
run_check assigns map_assigns 3919682 1 1000
run_check sales map_sales 10919494 1000 1000
run_check transfers-wrap map_transfers 10951800 60 1000
run_check wrapped map_wrapped_transfers 10951736 1000 1000
run_check metadata-fixed map_metadata 13047091 2 1000
run_check resolved map_resolved_sales 3914494 10000 11000
run_check resolved-production map_resolved_sales 3914494 10000 23000 --production-mode
python3 scripts/verify_live.py "$output_dir"
echo "Captures: $output_dir"
