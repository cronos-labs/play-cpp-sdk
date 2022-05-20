UNAME := $(shell uname)

ifeq ($(UNAME), Darwin)
	FLAGS=-framework Security -framework CoreFoundation
endif

ifeq ($(UNAME), Linux)
	FLAGS=-lpthread -lssl -lcrypto -ldl
endif

all: build run
build: prepare static dynamic
run: run_static run_dynamic

prepare:
	python3 helper.py --target_dir ../target/release

prepare_x86_64:
	python3 helper.py --target_dir ../target/x86_64-apple-darwin/release

static:
	g++ -o cppexamplestatic main.cc lib/libextra_cpp_bindings.a -std=c++11 $(FLAGS)

dynamic:
	g++ -o cppexample main.cc include/*.cc include/extra-cpp-bindings/src/*.cc lib/libcxxbridge1.a -lextra_cpp_bindings -std=c++11 $(FLAGS)  -L lib

x86_64_build: prepare_x86_64
	arch -x86_64 g++ -o cppexample main.cc lib/libextra_cpp_bindings.a -std=c++11 $(FLAGS)

run_static:
	./cppexamplestatic

run_dynamic:
	export LD_LIBRARY_PATH=$(PWD)/lib && ./cppexample

clean:
	rm cppexample cppexamplestatic
