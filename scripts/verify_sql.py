#!/usr/bin/env python3
"""Compare a live SQL backfill with saved stream output and official site samples.

PSQL must be on PATH (or set PSQL); normal PGHOST/PGPORT/PGUSER/PGDATABASE/
PGPASSWORD variables configure the connection. Never prints a DSN or password.
"""
import argparse
import collections
import datetime
from decimal import Decimal, getcontext, ROUND_HALF_UP
import json
import os
from pathlib import Path
import subprocess


getcontext().prec = 100

def query(sql):
    result = subprocess.run([os.environ.get('PSQL', 'psql'), '-X', '-v', 'ON_ERROR_STOP=1', '-At', '-c', sql], check=True, text=True, capture_output=True)
    return json.loads(result.stdout)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('sales_jsonl', type=Path, nargs='?')
    parser.add_argument('--receipts', type=Path, help='Also independently decode market receipts and replay bid state')
    args = parser.parse_args()
    if args.sales_jsonl is None:
        if args.receipts is None:
            parser.error('provide a sales capture or --receipts')
        database = query("SELECT coalesce(json_agg(row_to_json(s)), '[]') FROM (SELECT *, amount_eth::text AS exact_amount FROM sales ORDER BY block_number, ordinal) s")
        verify_receipts(args.receipts, database)
        return
    records = []
    for line in args.sales_jsonl.read_text().splitlines():
        row = json.loads(line)
        records.extend(row.get('@data', {}).get('sales', []))
    assert records, 'No reference sales found'
    database = query("SELECT coalesce(json_agg(row_to_json(s)), '[]') FROM (SELECT *, amount_eth::text AS exact_amount FROM sales ORDER BY block_number, ordinal) s")
    assert len(database) == len(records), (len(database), len(records))
    indexed = {(r['tx_hash'], r['log_index']): r for r in database}
    totals = collections.defaultdict(lambda: [0, Decimal(0)])
    for r in records:
        d = indexed[(r['trxHash'], r.get('logIndex', 0))]
        assert d['token_id'] == int(r.get('tokenId', 0))
        assert d['buyer'] == r['to'] and d['seller'] == r['from']
        assert d['bid_accepted'] == r.get('bidAccepted', False)
        assert Decimal(d['exact_amount']) == Decimal(r['amount'])
        assert d['block_number'] == int(r['blockNumber'])
        assert d['block_hash'] == r['blockHash']
        day = datetime.datetime.fromtimestamp(int(r['timestamp']), datetime.timezone.utc).date().isoformat()
        totals[day][0] += 1
        totals[day][1] += Decimal(r['amount'])
    daily = query("SELECT json_agg(row_to_json(d)) FROM (SELECT day, sales_count, volume_eth::text AS volume FROM daily_market_summary ORDER BY day) d")
    assert {d['day']: [d['sales_count'], Decimal(d['volume'])] for d in daily} == dict(totals)
    if args.receipts:
        verify_receipts(args.receipts, database)
    samples = json.loads((Path(__file__).resolve().parents[1] / 'tests/fixtures/official-website-sales.json').read_text())['sales']
    for sample in samples:
        matched = [d for d in database if d['tx_hash'] == sample['tx_hash'] and d['token_id'] == sample['token_id']]
        assert len(matched) == 1
        d = matched[0]
        for field in ['seller', 'buyer', 'bid_accepted']:
            assert d[field] == sample[field], (sample['token_id'], field)
        assert Decimal(d['exact_amount']) == Decimal(sample['amount_eth'])
        assert datetime.datetime.fromtimestamp(d['timestamp'], datetime.timezone.utc).date().isoformat() == sample['date']
        print(f"Official website match: Punk #{sample['token_id']}, {sample['amount_eth']} ETH, buyer, seller, date and transaction")
    print(f"PASS: {len(records)} SQL sales and {len(daily)} daily totals match the stream; {len(samples)} official website samples match.")


def verify_receipts(path, database):
    import base64
    from verify_bid_history import replay, rows, SIGNATURES, ZERO
    receipts = rows(path)
    changes, coverage, _ = replay(receipts)
    bid_prices = {(c[7], c[5]): c[2] for c in changes if not c[3]}
    expected_sales = []
    daily_reference = collections.defaultdict(list)
    punk_reference = collections.defaultdict(list)
    for block in receipts:
        preceding = {}
        for event in block['@data']['events']:
            log = event['log']
            topics = [base64.b64decode(t) for t in log['topics']]
            kind = SIGNATURES.get(topics[0].hex())
            tx = '0x' + event['txHash']
            if kind == 'erc20_transfer':
                preceding[(tx, '0x' + topics[1][-20:].hex())] = '0x' + topics[2][-20:].hex()
            elif kind == 'sale':
                seller, buyer = ['0x' + topics[i][-20:].hex() for i in (2, 3)]
                value = int.from_bytes(base64.b64decode(log.get('data', '')), 'big')
                accepted = buyer == ZERO
                if accepted:
                    buyer = preceding[(tx, seller)]
                    value = bid_prices[(tx, log.get('blockIndex', 0))]
                token = int.from_bytes(topics[1], 'big')
                amount = Decimal(value) / Decimal(10**18)
                expected_sales.append((tx, log.get('blockIndex', 0), token, seller, buyer, amount, accepted))
                day = datetime.datetime.fromisoformat(block['@data']['clock']['timestamp'].replace('Z', '+00:00')).astimezone(datetime.timezone.utc).date().isoformat()
                daily_reference[day].append((amount, buyer, seller))
                punk_reference[(day, token)].append(amount)
    actual = [(r['tx_hash'], r['log_index'], r['token_id'], r['seller'], r['buyer'], Decimal(r['exact_amount']), r['bid_accepted']) for r in database]
    assert actual == expected_sales, 'SQL sale values differ from independent ABI/bid replay'
    sql_bids = query("SELECT json_agg(row_to_json(b)) FROM (SELECT *,amount_eth::text AS exact_amount FROM bid_changes ORDER BY block_number,ordinal) b")
    actual_bids = [(r['token_id'],r['bidder'],int(Decimal(r['exact_amount'])*Decimal(10**18)),r['is_open'],r['block_number'],r['log_index'],r['ordinal'],r['tx_hash']) for r in sql_bids]
    assert actual_bids == changes, 'SQL bid changes differ from independent bid replay'
    current = query("SELECT json_agg(row_to_json(b)) FROM (SELECT token_id,bidder,is_open,amount_eth::text AS amount FROM current_bids ORDER BY token_id) b")
    latest = {c[0]: (c[1], c[3], Decimal(c[2])/Decimal(10**18)) for c in changes}
    assert {r['token_id']: (r['bidder'],r['is_open'],Decimal(r['amount'])) for r in current} == latest, 'Current SQL bids differ from replay'
    daily = query("SELECT json_agg(row_to_json(d)) FROM (SELECT day,sales_count,volume_eth::text AS volume,min_price_eth::text AS minimum,max_price_eth::text AS maximum,average_price_eth::text AS average,unique_buyers,unique_sellers FROM daily_market_summary) d")
    assert {r['day'] for r in daily} == set(daily_reference)
    for r in daily:
        values = daily_reference[r['day']]
        prices = [v[0] for v in values]
        assert r['sales_count'] == len(prices)
        assert Decimal(r['volume']) == sum(prices)
        assert Decimal(r['minimum']) == min(prices) and Decimal(r['maximum']) == max(prices)
        observed_avg = Decimal(r['average'])
        expected_avg = (sum(prices)/len(prices)).quantize(Decimal(1).scaleb(observed_avg.as_tuple().exponent), rounding=ROUND_HALF_UP)
        assert observed_avg == expected_avg
        assert r['unique_buyers'] == len({v[1] for v in values})
        assert r['unique_sellers'] == len({v[2] for v in values})
    daily_punks = query("SELECT json_agg(row_to_json(d)) FROM (SELECT day,token_id,sales_count,volume_eth::text AS volume,min_price_eth::text AS minimum,max_price_eth::text AS maximum FROM daily_punk_summary) d")
    assert {(r['day'],r['token_id']) for r in daily_punks} == set(punk_reference)
    for r in daily_punks:
        prices = punk_reference[(r['day'],r['token_id'])]
        assert (r['sales_count'], Decimal(r['volume']), Decimal(r['minimum']), Decimal(r['maximum'])) == (len(prices),sum(prices),min(prices),max(prices))
    print(f'Current bid snapshots: {len(current)}; full market/day summaries: {len(daily)}; per-Punk/day summaries: {len(daily_punks)}')
    print(f'Independent receipts match: {len(actual)} sales, {len(changes)} bid changes; cases {dict(coverage)}')


if __name__ == '__main__':
    main()
