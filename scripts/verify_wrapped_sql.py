#!/usr/bin/env python3
"""Check bounded SQL ownership against separately captured native/wrapped maps.
Example range 10951800..10951859, in a separate database, not a partial main index.
"""
import json
import sys
from pathlib import Path
from verify_sql import query


def main(native_path, wrapped_path):
    expected = []
    zero = '0x' + '0' * 40
    for asset, path in [('native', native_path), ('wrapped', wrapped_path)]:
        for line in Path(path).read_text().splitlines():
            block = json.loads(line)
            if not 10951800 <= block['@block'] < 10951860:
                continue
            for t in block['@data'].get('transfers', []):
                kind = 'mint' if asset == 'wrapped' and t['from'] == zero else 'burn' if asset == 'wrapped' and t['to'] == zero else 'transfer'
                expected.append(dict(token_id=int(t.get('tokenId', 0)), asset=asset, kind=kind, from_address=t['from'], to_address=t['to'], tx_hash=t['trxHash'], block_number=int(t['blockNumber']), ordinal=int(t['ordinal']), log_index=t.get('logIndex', 0)))
    expected.sort(key=lambda r: (r['block_number'], r['ordinal']))
    observed = query("SELECT json_agg(row_to_json(o)) FROM (SELECT token_id,asset,kind,from_address,to_address,tx_hash,block_number,ordinal,log_index FROM ownership_history WHERE kind <> 'sale' ORDER BY block_number,ordinal) o")
    assert observed == expected and len(expected) == 6
    owners = query("SELECT json_agg(row_to_json(o)) FROM current_ownership o WHERE token_id IN (4046,9972)")
    owners = {r['token_id']: r for r in owners}
    assert owners[9972]['native_owner'] == '0xb7f7f6c52f2e2fdb1963eab30438024864c313f6'
    assert owners[9972]['wrapped_holder'] == owners[9972]['effective_holder'] == '0x7b1d4f3602f0fe16c4f611e124d3d7b307245ba8'
    assert owners[4046]['native_owner'] == owners[4046]['effective_holder'] == '0xd387a6e4e84a6c86bd90c158c6028a58cc8ac459'
    assert owners[4046]['wrapped_holder'] is None
    print('PASS: 6 native/wrapped events, Punk 9972 wrapped custody and Punk 4046 unwrapped ownership.')


if __name__ == '__main__':
    main(*sys.argv[1:])
