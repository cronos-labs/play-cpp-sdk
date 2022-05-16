build_cpp:
	cargo build --package extra-cpp-bindings --release
	cd demo && make build

cpp: build_cpp
	cd demo && . ./.env && make run

cppx86_64:
	rustup target add x86_64-apple-darwin
	cargo build --package extra-cpp-bindings --release --target x86_64-apple-darwin
	cd demo && make x86_64_build
