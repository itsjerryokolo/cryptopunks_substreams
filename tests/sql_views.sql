\set ON_ERROR_STOP on
BEGIN;
CREATE SCHEMA cryptopunks_view_test;
SET LOCAL search_path TO cryptopunks_view_test;
\ir ../schema.sql
-- Assignment, custody deposit, ERC721 mint/transfer/burn, and native withdrawal.
INSERT INTO ownership_history VALUES
 ('a',1,'native','assignment','zero','alice','tx',1,'hash',1498251906,1,1),
 ('b',1,'native','transfer','alice','0xb7f7f6c52f2e2fdb1963eab30438024864c313f6','tx',2,'hash',1498251906,2,2),
 ('c',1,'wrapped','mint','0x0000000000000000000000000000000000000000','alice','tx',2,'hash',1498251906,3,3),
 ('d',1,'wrapped','transfer','alice','bob','tx',2,'hash',1498251906,4,4);
DO $$ BEGIN
 IF (SELECT effective_holder FROM current_ownership WHERE token_id=1) IS DISTINCT FROM 'bob' THEN RAISE EXCEPTION 'wrapped holder ordering failed'; END IF;
 IF (SELECT native_owner FROM current_ownership WHERE token_id=1) IS DISTINCT FROM '0xb7f7f6c52f2e2fdb1963eab30438024864c313f6' THEN RAISE EXCEPTION 'native custody lost'; END IF;
END $$;
INSERT INTO ownership_history VALUES
 ('e',1,'wrapped','burn','bob','0x0000000000000000000000000000000000000000','tx',3,'hash',1498251906,5,5);
DO $$ BEGIN
 IF EXISTS (SELECT 1 FROM current_ownership WHERE wrapped_holder IS NOT NULL OR effective_holder IS NOT NULL) THEN RAISE EXCEPTION 'burn retained stale holder'; END IF;
END $$;
INSERT INTO ownership_history VALUES
 ('f',1,'native','transfer','0xb7f7f6c52f2e2fdb1963eab30438024864c313f6','bob','tx',3,'hash',1498251906,6,6);
DO $$ BEGIN
 IF (SELECT effective_holder FROM current_ownership WHERE token_id=1) IS DISTINCT FROM 'bob' THEN RAISE EXCEPTION 'unwrap failed'; END IF;
END $$;
-- Rollback-like deletion of later facts restores the earlier wrapped holder.
DELETE FROM ownership_history WHERE block_number=3;
DO $$ BEGIN
 IF (SELECT wrapped_holder FROM current_ownership WHERE token_id=1) IS DISTINCT FROM 'bob' THEN RAISE EXCEPTION 'ownership deletion failed'; END IF;
END $$;
SET LOCAL TIME ZONE 'Pacific/Honolulu';
INSERT INTO sales VALUES
 ('s1',1,'alice','bob',0.000000000000000001,true,'tx1',1,'hash',86400,1,1),
 ('s2',1,'bob','bob',0,false,'tx2',2,'hash',86401,2,2),
 ('s3',2,'alice','carol',0.1,false,'tx3',3,'hash',86402,3,3);
DO $$ BEGIN
 IF (SELECT volume_eth FROM daily_market_summary) IS DISTINCT FROM 0.100000000000000001 OR (SELECT sales_count FROM daily_market_summary) IS DISTINCT FROM 3 THEN RAISE EXCEPTION 'exact aggregation failed'; END IF;
 IF (SELECT day FROM daily_market_summary) IS DISTINCT FROM DATE '1970-01-02' THEN RAISE EXCEPTION 'UTC day grouping failed'; END IF;
 IF (SELECT sales_count FROM daily_punk_summary WHERE token_id=1) IS DISTINCT FROM 2 THEN RAISE EXCEPTION 'per-Punk summary failed'; END IF;
END $$;
DELETE FROM sales WHERE id='s3';
DO $$ BEGIN
 IF (SELECT volume_eth FROM daily_market_summary) IS DISTINCT FROM 0.000000000000000001 THEN RAISE EXCEPTION 'aggregate deletion failed'; END IF;
END $$;
-- A burned wrapper must not regain the previous holder on a later deposit.
INSERT INTO ownership_history VALUES
 ('g',1,'wrapped','burn','bob','0x0000000000000000000000000000000000000000','tx',4,'hash',1498251906,1,1),
 ('h',1,'native','transfer','wrapper','bob','tx',4,'hash',1498251906,2,2),
 ('i',1,'native','transfer','bob','0xb7f7f6c52f2e2fdb1963eab30438024864c313f6','tx',5,'hash',1498251906,1,1);
DO $$ BEGIN
 IF (SELECT effective_holder FROM current_ownership WHERE token_id=1) IS NOT NULL THEN RAISE EXCEPTION 'rewrap reused burned holder'; END IF;
END $$;
INSERT INTO ownership_history VALUES
 ('j',1,'wrapped','mint','0x0000000000000000000000000000000000000000','carol','tx',5,'hash',1498251906,2,2),
 ('k',2,'wrapped','transfer','alice','bob','tx',5,'hash',1498251906,3,3);
DO $$ BEGIN
 IF (SELECT effective_holder FROM current_ownership WHERE token_id=1) IS DISTINCT FROM 'carol' THEN RAISE EXCEPTION 'rewrap holder incorrect'; END IF;
 IF EXISTS (SELECT 1 FROM current_ownership WHERE token_id=2) THEN RAISE EXCEPTION 'partial wrapped history invented native custody'; END IF;
END $$;
-- Current bids must use block then ordinal, retain closed history, and isolate Punks.
INSERT INTO bid_changes VALUES
 ('b1',1,'alice',1,true,'tx1',1,'hash',86400,999,1),
 ('b2',1,'bob',2,true,'tx2',2,'hash',86400,1,2),
 ('b3',1,'bob',2,false,'tx3',2,'hash',86400,2,3),
 ('b4',2,'bob',3,true,'tx4',2,'hash',86400,3,4);
DO $$ BEGIN
 IF (SELECT id FROM current_bids WHERE token_id=1) IS DISTINCT FROM 'b3' THEN RAISE EXCEPTION 'bid chronology failed'; END IF;
 IF (SELECT count(*) FROM current_bids WHERE is_open) IS DISTINCT FROM 1::bigint THEN RAISE EXCEPTION 'closed bid leaked into active query'; END IF;
END $$;
DELETE FROM bid_changes WHERE id='b3';
DO $$ BEGIN
 IF (SELECT id FROM current_bids WHERE token_id=1 AND is_open) IS DISTINCT FROM 'b2' THEN RAISE EXCEPTION 'bid deletion did not restore prior state'; END IF;
END $$;
-- Every empty aggregate view should have zero rows, not invented zero-valued days.
DELETE FROM sales;
DO $$ BEGIN
 IF EXISTS (SELECT 1 FROM daily_market_summary) OR EXISTS (SELECT 1 FROM daily_punk_summary) THEN RAISE EXCEPTION 'empty aggregates invented rows'; END IF;
END $$;
-- Large uint256-scale value plus one wei must survive database storage/summing.
INSERT INTO sales VALUES
 ('max',3,'alice','bob',115792089237316195423570985008687907853269984665640564039457.584007913129639935,false,'max',10,'hash',86400,1,1),
 ('wei',3,'bob','alice',0.000000000000000001,false,'wei',10,'hash',86400,2,2),
 ('midnight',4,'alice','bob',1,false,'night',10,'hash',86399,3,3);
DO $$ BEGIN
 IF (SELECT volume_eth FROM daily_market_summary WHERE day=DATE '1970-01-02') IS DISTINCT FROM 115792089237316195423570985008687907853269984665640564039457.584007913129639936 THEN RAISE EXCEPTION 'full uint256 scale lost'; END IF;
 IF (SELECT sales_count FROM daily_market_summary WHERE day=DATE '1970-01-01') IS DISTINCT FROM 1::bigint THEN RAISE EXCEPTION 'midnight boundary failed'; END IF;
 IF (SELECT unique_buyers FROM daily_market_summary WHERE day=DATE '1970-01-02') IS DISTINCT FROM 2::bigint THEN RAISE EXCEPTION 'distinct buyers failed'; END IF;
END $$;
ROLLBACK;
