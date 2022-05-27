build_cpp:
	cargo build --package extra-cpp-bindings --release
	cd demo && git submodule update --init --recursive && make build

cpp: build_cpp
# 1. In order to use crypto pay api, you need to Generate Keys in
# https://merchant.crypto.com/developers/api_keys first
#
# 2. Copy the `Publishable Key` or `Secret Key` as `PAY_API_KEY`'s value in `.env`
	cd demo && . ./.env && make run

cpp_ci_test: build_cpp
	cd demo && make run # we load the envs in test.yml

webhook:
# 1. Install ngrok for crypto pay api testing: https://ngrok.com/download
#
# 2. Run `ngrok http 4567` in a seperate terminal first, then add the `payload url` into
# https://merchant.crypto.com/developers/webhooks
#
# 3. Find the `SIGNATURE SECRET` in merchant dashboard, and copy it as
# `PAY_WEBHOOK_SIGNATURE_SECRET`'s value in `.env`
	cd demo && . ./.env && npm install && node server.js

cppx86_64:
	rustup target add x86_64-apple-darwin
	cargo build --package extra-cpp-bindings --release --target x86_64-apple-darwin
	cd demo && make x86_64_build
