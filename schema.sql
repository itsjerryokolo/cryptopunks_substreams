-- PostgreSQL, original CryptoPunks marketplace only; amounts are exact ETH.
CREATE TABLE ownership_history (
  id text PRIMARY KEY, token_id bigint NOT NULL, asset text NOT NULL,
  kind text NOT NULL, from_address text NOT NULL, to_address text NOT NULL,
  tx_hash text NOT NULL, block_number bigint NOT NULL, block_hash text NOT NULL,
  timestamp bigint NOT NULL, ordinal numeric(20,0) NOT NULL, log_index bigint NOT NULL
);
CREATE INDEX ownership_token_order ON ownership_history (token_id, asset, block_number DESC, ordinal DESC);
CREATE TABLE sales (
  id text PRIMARY KEY, token_id bigint NOT NULL, seller text NOT NULL, buyer text NOT NULL,
  amount_eth numeric NOT NULL, bid_accepted boolean NOT NULL,
  tx_hash text NOT NULL, block_number bigint NOT NULL, block_hash text NOT NULL,
  timestamp bigint NOT NULL, ordinal numeric(20,0) NOT NULL, log_index bigint NOT NULL
);
CREATE INDEX sales_token_order ON sales (token_id, block_number, ordinal);
CREATE TABLE bid_changes (
  id text PRIMARY KEY, token_id bigint NOT NULL, bidder text NOT NULL,
  amount_eth numeric NOT NULL, is_open boolean NOT NULL,
  tx_hash text NOT NULL, block_number bigint NOT NULL, block_hash text NOT NULL,
  timestamp bigint NOT NULL, ordinal numeric(20,0) NOT NULL, log_index bigint NOT NULL
);
CREATE INDEX bids_token_order ON bid_changes (token_id, block_number DESC, ordinal DESC);

-- State is current only through the sink cursor. A partial backfill is not head state.
CREATE VIEW current_ownership AS
WITH latest AS (
  SELECT DISTINCT ON (token_id, asset) * FROM ownership_history
  ORDER BY token_id, asset, block_number DESC, ordinal DESC
), owners AS (
  SELECT n.token_id, n.to_address AS native_owner,
    CASE WHEN n.to_address = '0xb7f7f6c52f2e2fdb1963eab30438024864c313f6'
      AND w.to_address <> '0x0000000000000000000000000000000000000000'
      THEN w.to_address END AS wrapped_holder,
    GREATEST(n.block_number, w.block_number) AS last_change_block
  FROM latest n LEFT JOIN latest w ON w.token_id = n.token_id AND w.asset = 'wrapped'
  WHERE n.asset = 'native'
)
SELECT *, CASE WHEN native_owner = '0xb7f7f6c52f2e2fdb1963eab30438024864c313f6'
  THEN wrapped_holder ELSE native_owner END AS effective_holder FROM owners;

CREATE VIEW current_bids AS
SELECT DISTINCT ON (token_id) * FROM bid_changes
ORDER BY token_id, block_number DESC, ordinal DESC;

-- Ordinary views recalculate from facts, including after sink rollback deletes.
-- Zero-price and self-sales remain included; this is not a wash-trade filter.
CREATE VIEW daily_market_summary AS
SELECT (to_timestamp(timestamp) AT TIME ZONE 'UTC')::date AS day,
  count(*) AS sales_count, sum(amount_eth) AS volume_eth,
  min(amount_eth) AS min_price_eth, max(amount_eth) AS max_price_eth,
  avg(amount_eth) AS average_price_eth,
  count(DISTINCT buyer) AS unique_buyers, count(DISTINCT seller) AS unique_sellers
FROM sales GROUP BY 1;
CREATE VIEW daily_punk_summary AS
SELECT (to_timestamp(timestamp) AT TIME ZONE 'UTC')::date AS day, token_id,
  count(*) AS sales_count, sum(amount_eth) AS volume_eth,
  min(amount_eth) AS min_price_eth, max(amount_eth) AS max_price_eth
FROM sales GROUP BY 1, 2;
