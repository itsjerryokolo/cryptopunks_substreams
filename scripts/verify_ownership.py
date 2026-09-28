#!/usr/bin/env python3
"""Independently ABI-decode native market receipts and compare SQL ownership.
Uses the same PG* / PSQL environment as verify_sql.py; run on an identical range.
"""
import base64
import json
import sys
from pathlib import Path
from verify_sql import query

ASSIGN = '8a0e37b73a0d9c82e205d4d1a3ff3d0b57ce5f4d7bccf6bac03336dc101cb7ba'
TRANSFER = '05af636b70da6819000c49f85b21fa82081c632069bb626f30932034099107d8'
SALE = '58e5d5a525e3b40bc15abaa38b5882678db1ee68befd2f60bafe3a7fd06db9e3'
GENERIC = 'ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef'
ZERO = '0x' + '0' * 40


def verify(path):
    expected = []
    owners = {}
    for line in Path(path).read_text().splitlines():
        block = json.loads(line)
        preceding = {}
        for event in block['@data']['events']:
            log = event['log']
            assert base64.b64decode(log['address']).hex() == 'b47e3cd837ddf8e4c57f05d70ab865de6e193bbb'
            topics = [base64.b64decode(t) for t in log['topics']]
            sig = topics[0].hex()
            tx = '0x' + event['txHash']
            address = lambda i: '0x' + topics[i][-20:].hex()
            data = base64.b64decode(log.get('data', ''))
            if sig == GENERIC:
                preceding[(tx, address(1))] = address(2)
                continue
            if sig == ASSIGN:
                kind, token, sender, recipient = 'assignment', int.from_bytes(data, 'big'), ZERO, address(1)
            elif sig == TRANSFER:
                kind, token, sender, recipient = 'transfer', int.from_bytes(data, 'big'), address(1), address(2)
            elif sig == SALE:
                kind, token, sender, recipient = 'sale', int.from_bytes(topics[1], 'big'), address(2), address(3)
                if recipient == ZERO:
                    recipient = preceding[(tx, sender)]
            else:
                continue
            expected.append(dict(token_id=token, kind=kind, from_address=sender, to_address=recipient, tx_hash=tx, block_number=block['@block'], ordinal=int(log['ordinal']), log_index=log.get('blockIndex', 0)))
            owners[token] = recipient
    assert expected, 'No ownership reference events'
    observed = query("SELECT json_agg(row_to_json(o)) FROM (SELECT token_id, kind, from_address, to_address, tx_hash, block_number, ordinal, log_index FROM ownership_history WHERE asset='native' ORDER BY block_number, ordinal) o")
    assert observed == expected, 'Native ownership history differs from raw receipts'
    current = query("SELECT json_agg(row_to_json(o)) FROM (SELECT token_id, native_owner, effective_holder FROM current_ownership) o")
    assert {r['token_id']: r['native_owner'] for r in current} == owners
    assert all(r['native_owner'] == r['effective_holder'] for r in current), 'Use the early pre-wrapper range for this check'
    print(f'PASS: {len(expected)} ownership events and {len(owners)} native owners match raw receipts.')


if __name__ == '__main__':
    verify(sys.argv[1])
