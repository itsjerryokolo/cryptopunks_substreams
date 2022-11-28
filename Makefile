.PHONY: build
build:
	cargo build --target wasm32-unknown-unknown --release

.PHONY: stream
stream: build
	substreams run -e mainnet.eth.streamingfast.io:443 substreams.yaml map_transfers -s 12292922 -t +10

.PHONY: codegen
codegen:
	substreams protogen ./substreams.yaml --exclude-paths="sf/substreams,google"

.PHONY: test_transfers
test_transfers:
	substreams run -e mainnet.eth.streamingfast.io:443 substreams.yaml  map_transfers --start-block 10914494 --stop-block +500

.PHONY: test-assign
test-assign:
	substreams run -e mainnet.eth.streamingfast.io:443 substreams.yaml  map_assigns --start-block 3919494 --stop-block +300

.PHONY: test-assign-store
test-assign-store:
	substreams run -e mainnet.eth.streamingfast.io:443 substreams.yaml  store_assigns --start-block 3917494 --stop-block +2000


.PHONY: test-punks
test-punks:
	substreams run -e mainnet.eth.streamingfast.io:443 substreams.yaml  store_all_punks --start-block 3919494 --stop-block +1000

.PHONY: test-sales
test-sales:
	substreams run -e mainnet.eth.streamingfast.io:443 substreams.yaml  map_sales --start-block 10919494 --stop-block +300

.PHONY: test-sales-store
test-sales-store:
	substreams run -e mainnet.eth.streamingfast.io:443 substreams.yaml  store_punk_sales --start-block 13922900 --stop-block +500