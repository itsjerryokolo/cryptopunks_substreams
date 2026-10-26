#!/usr/bin/env python3
"""Verify the additional official website samples and RGBA/SVG pixel agreement."""
import datetime
import json
from decimal import Decimal
from pathlib import Path
import sys
import xml.etree.ElementTree as ET


def verify(capture_dir):
    fixture = json.loads((Path(__file__).resolve().parents[1] / 'tests/fixtures/official-website-sep30.json').read_text())
    for expected in fixture['metadata']:
        token = expected['token_id']
        rows = [json.loads(line) for line in (capture_dir / f'metadata-{token}.jsonl').read_text().splitlines()]
        assert len(rows) == 1 and rows[0]['@block'] == 13057090 - token
        values = rows[0]['@data']['metadatas']
        assert len(values) == 1
        value = values[0]
        assert int(value['tokenId']) == token
        assert value['punkType'] == expected['punk_type']
        assert set(value['traits'].split(',')) == set(expected['traits'])
        rgba = bytes.fromhex(value['image'].removeprefix('0x'))
        assert len(rgba) == 24 * 24 * 4
        prefix = 'data:image/svg+xml;utf8,'
        assert value['svg'].startswith(prefix)
        svg = ET.fromstring(value['svg'][len(prefix):])
        assert svg.attrib['viewBox'] == '0 0 24 24'
        rendered = bytearray(24 * 24 * 4)
        for rect in svg:
            assert rect.tag == '{http://www.w3.org/2000/svg}rect'
            x, y, width, height = [int(rect.attrib[k]) for k in ['x', 'y', 'width', 'height']]
            assert 0 <= x < x + width <= 24 and 0 <= y < y + height <= 24
            color = rect.attrib['fill']
            assert color.startswith('#') and len(color) in (7, 9)
            pixel = bytes.fromhex(color[1:])
            if len(pixel) == 3:
                pixel += b'\xff'
            for py in range(y, y + height):
                for px in range(x, x + width):
                    offset = (py * 24 + px) * 4
                    rendered[offset:offset+4] = pixel
        assert rendered == rgba, f'RGBA/SVG image disagreement for {token}'
        print(f"Metadata #{token}: official {expected['punk_type']} / {', '.join(expected['traits'])}; all 576 RGBA/SVG pixels match")
    sales = [s for line in (capture_dir / 'sale-5822.jsonl').read_text().splitlines() for s in json.loads(line)['@data'].get('sales', [])]
    expected = fixture['sales'][0]
    actual = [s for s in sales if s['trxHash'] == expected['tx_hash'] and int(s.get('tokenId', 0)) == expected['token_id']]
    assert len(actual) == 1
    sale = actual[0]
    assert sale['from'] == expected['seller'] and sale['to'] == expected['buyer']
    assert not sale.get('bidAccepted', False), 'Raw accepted-bid price is not a resolved price'
    assert Decimal(sale['amount']) == Decimal(expected['amount_eth'])
    assert int(sale['blockNumber']) == expected['block']
    assert datetime.datetime.fromtimestamp(int(sale['timestamp']), datetime.timezone.utc).date().isoformat() == expected['date']
    print('Punk #5822: 8,000 ETH direct sale matches official website parties/date/hash and Etherscan block.')
    expected = fixture['recent_transfer']
    receipts = json.loads((Path(__file__).resolve().parents[1] / 'tests/fixtures/mainnet-transfer-26089666.json').read_text())['result']
    assert receipts['status'] == '0x1' and receipts['transactionHash'] == expected['tx_hash']
    changes = [c for line in (capture_dir / 'recent-ownership.jsonl').read_text().splitlines() for c in json.loads(line)['@data'].get('changes', [])]
    matched = [c for c in changes if c['trxHash'] == expected['tx_hash'] and int(c.get('tokenId', 0)) == expected['token_id']]
    assert len(matched) == 1
    c = matched[0]
    receipt_log = next(l for l in receipts['logs'] if int(l['logIndex'], 16) == c['logIndex'])
    assert receipt_log['address'] == '0xb47e3cd837ddf8e4c57f05d70ab865de6e193bbb'
    assert receipt_log['topics'][0] == '0x05af636b70da6819000c49f85b21fa82081c632069bb626f30932034099107d8'
    assert int(receipt_log['data'], 16) == expected['token_id']
    assert c['asset'] == 'native' and c['kind'] == 'transfer'
    for field, topic in [('from', 1), ('to', 2)]:
        assert c[field] == expected[field] == '0x' + receipt_log['topics'][topic][-40:]
    assert int(c['blockNumber']) == int(receipt_log['blockNumber'], 16) == expected['block']
    assert c['blockHash'] == receipt_log['blockHash']
    assert datetime.datetime.fromtimestamp(int(c['timestamp']), datetime.timezone.utc).date().isoformat() == expected['date']
    print('Recent Punk #4201 transfer: official website and separate public RPC receipt match.')


if __name__ == '__main__':
    verify(Path(sys.argv[1]))
