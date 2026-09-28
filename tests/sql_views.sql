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
 IF (SELECT volume_eth FROM daily_market_summary) <> 0.100000000000000001 OR (SELECT sales_count FROM daily_market_summary) <> 3 THEN RAISE EXCEPTION 'exact aggregation failed'; END IF;
 IF (SELECT day FROM daily_market_summary) <> DATE '1970-01-02' THEN RAISE EXCEPTION 'UTC day grouping failed'; END IF;
 IF (SELECT sales_count FROM daily_punk_summary WHERE token_id=1) <> 2 THEN RAISE EXCEPTION 'per-Punk summary failed'; END IF;
END $$;
DELETE FROM sales WHERE id='s3';
DO $$ BEGIN
 IF (SELECT volume_eth FROM daily_market_summary) <> 0.000000000000000001 THEN RAISE EXCEPTION 'aggregate deletion failed'; END IF;
END $$;
ROLLBACK;
