all: build_cpp

clone:
	git submodule update --init --recursive

build_play-cpp-sdk: clone
ifeq ($(shell uname -m), x86_64)
	./checkmac.sh && cargo build --package play-cpp-sdk --release
endif
ifeq ($(shell uname -m), arm64)
	rustup target add x86_64-apple-darwin
	./checkmac.sh && cargo build --package play-cpp-sdk --release --target x86_64-apple-darwin
endif

build_extra-cpp-bindings:
	cargo build --package extra-cpp-bindings --release

build_cpp: build_play-cpp-sdk
	./checkmac.sh && cd demo && make build

cpp: build_cpp
# 1. In order to use crypto pay api, you need to Generate Keys in
# https://merchant.crypto.com/developers/api_keys first
#
# 2. Copy the `Publishable Key` or `Secret Key` as `PAY_API_KEY`'s value in `.env`
# cd demo && git submodule update --init --recursive && make build
	cd demo && make run

cpp-ci-tests:
	./integration_test.sh

webhook:
# 1. Install ngrok for crypto pay api testing: https://ngrok.com/download
#
# 2. Run `ngrok http 4567` in a seperate terminal first, then add the `payload url` into
# https://merchant.crypto.com/developers/webhooks
#
# 3. Find the `SIGNATURE SECRET` in merchant dashboard, and copy it as
# `PAY_WEBHOOK_SIGNATURE_SECRET`'s value in `.env`
	cd demo && . ./.env && npm install && node server.js

install:
	. ./install.sh

uninstall:
	rm -rf build
