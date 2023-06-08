# keys pinched from https://docs.substrate.io/tutorials/v3/private-network/#add-keys-to-keystore
define KEYSTORE_POPULATE_PAYLOAD
{
	"jsonrpc": "2.0",
	"method": "author_insertKey",
	"params": ["bepa","clip organ olive upper oak void inject side suit toilet stick narrow","0x9effc1668ca381c242885516ec9fa2b19c67b6684c02a8a3237b6862e5c8cd7e"],
	"id": 1
}
endef
export KEYSTORE_POPULATE_PAYLOAD

export RUST_BACKTRACE=1

all: build test clippy

clean:
	cargo clean

clean-node:
	cargo clean -p node-template

enforce-nightly-2022-08-10:
	rustup target add wasm32-unknown-unknown --toolchain nightly-2022-08-10

build: enforce-nightly-2022-08-10
	cargo build

test:
	cargo test

clippy:
	cargo clippy

run-node: build
	target/debug/node-template --dev --tmp

run:
	cargo run -- --dev --tmp

populate-keys:
	curl --location --request POST 'http://localhost:9933' \
		--header 'Content-Type: application/json' \
		--data-raw "$$KEYSTORE_POPULATE_PAYLOAD"

.PHONY: all