.PHONY: build
build:
	cargo build --target wasm32-unknown-unknown --release

.PHONY: stream
stream: build
	substreams run -e mainnet.eth.streamingfast.io:443 substreams.yaml map_transfers -s 12292922 -t +10

.PHONY: graph
graph: 
	substreams graph substreams.yaml
.PHONY: codegen
codegen:
	substreams protogen ./substreams.yaml --exclude-paths="sf/substreams,google"

.PHONY: test_transfers
test_transfers:
	substreams run -e mainnet.eth.streamingfast.io:443 substreams.yaml  map_transfers --start-block 10914494 --stop-block +500

.PHONY: test-assign
test-assign:
	substreams run -e mainnet.eth.streamingfast.io:443 substreams.yaml  map_assigns --start-block 3919494 --stop-block +300 -o json

.PHONY: test-assign-store
test-assign-store:
	substreams run -e mainnet.eth.streamingfast.io:443 substreams.yaml  store_assigns --start-block 3919494 --stop-block +300 -o json

.PHONY: test-assignees
test-assignees:
	substreams run -e mainnet.eth.streamingfast.io:443 substreams.yaml  punks_assignees --start-block 3917494 --stop-block +2000 -o json


.PHONY: test-punks
test-punks:
	substreams run -e mainnet.eth.streamingfast.io:443 substreams.yaml  store_all_punks --start-block 3918997 --stop-block +1000 -o json

.PHONY: test-sales
test-sales:
	substreams run -e mainnet.eth.streamingfast.io:443 substreams.yaml  map_sales --start-block 10919494 --stop-block +300 -o json

.PHONY: test-sales-store
test-sales-store:
	substreams run -e mainnet.eth.streamingfast.io:443 substreams.yaml  store_punk_sales --start-block 13922900 --stop-block +500 -o json

.PHONY: test-bids-store
test-bids-store:
	substreams run -e mainnet.eth.streamingfast.io:443 substreams.yaml  store_bids --start-block 13922900 --stop-block +500 -o json


.PHONY: test-sales-volume
test-sales-volume:
	substreams run -e mainnet.eth.streamingfast.io:443 substreams.yaml  store_total_volume --start-block 13922900 --stop-block +5000 -o json

.PHONY: test-punk-volume
test-punk-volume:
	substreams run -e mainnet.eth.streamingfast.io:443 substreams.yaml  store_punk_volume --start-block 13924900 --stop-block +5000 -o json


.PHONY: test-punk-state
test-punk-state:
	substreams run -e mainnet.eth.streamingfast.io:443 substreams.yaml  punk_state --start-block 10919494 --stop-block +2000 -o json


.PHONY: test-bids-state
test-bids-state:
	substreams run -e mainnet.eth.streamingfast.io:443 substreams.yaml  bids_state --start-block 13922900 --stop-block +2000 -o json

.PHONY: test-asks-state
test-asks-state:
	substreams run -e mainnet.eth.streamingfast.io:443 substreams.yaml  asks_state --start-block 13922900 --stop-block +10000 -o json


PHONY: test-contract
test-contract:
	substreams run -e mainnet.eth.streamingfast.io:443 substreams.yaml  contract_metadata --start-block 3919681 --stop-block +300 -o json


PHONY: test-punk-metadata
test-punk-metadata:
	substreams run -e mainnet.eth.streamingfast.io:443 substreams.yaml  map_metadata --start-block 13047091 --stop-block +2


PHONY: test-store-metadata
test-store-metadata:
	substreams run -e mainnet.eth.streamingfast.io:443 substreams.yaml  store_metadata --start-block 13047091 --stop-block +2

	