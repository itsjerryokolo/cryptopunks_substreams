#!/usr/bin/env bash
# Reproduce the September 30 website/metadata and recent-transfer checks.
set -euo pipefail
cd "$(dirname "$0")/.."
cli="${SUBSTREAMS:-substreams}"
package=itsjerryokolo-cryptopunks-v0.2.0.spkg
output_dir="${1:-$(mktemp -d "${TMPDIR:-/tmp}/cryptopunks-additional.XXXXXX")}"
mkdir -p "$output_dir"
output_dir="$(cd "$output_dir" && pwd)"
run_sample() {
  local name="$1" module="$2" start="$3" count="$4"
  "$cli" run "$package" "$module" -s "$start" -t "+$count" -o jsonl \
    --max-retries 0 --limit-processed-blocks 1200 \
    > "$output_dir/$name.jsonl" 2> "$output_dir/$name.log"
}
# Do not change production initialBlock: these are stateless modules.
for token in 5822 4156 8857 0; do
  run_sample "metadata-$token" map_metadata "$((13057090-token))" 1
done
run_sample sale-5822 map_sales 14193462 1
# Fixed finalized range observed on 2026-09-30; not a moving 'latest' assertion.
run_sample recent-ownership map_ownership_changes 26089381 1000
python3 scripts/verify_additional_samples.py "$output_dir"
echo "Captures: $output_dir"
