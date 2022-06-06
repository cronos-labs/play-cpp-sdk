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

easywsclient.o:
	g++ -c include/easywsclient/easywsclient.cpp

easywsclient.o.x86_64:
	arch -x86_64 g++ -c include/easywsclient/easywsclient.cpp

static: easywsclient.o
	g++ -o cppexamplestatic easywsclient.o main.cc lib/libextra_cpp_bindings.a -std=c++14 $(FLAGS)

dynamic: easywsclient.o
	g++ -o cppexample \
		easywsclient.o \
		main.cc \
		include/*.cc \
		include/extra-cpp-bindings/src/*.cc \
		lib/libcxxbridge1.a \
		-lextra_cpp_bindings \
		-std=c++14 \
		$(FLAGS) \
		-L lib \

x86_64_build: prepare_x86_64 easywsclient.o.x86_64
	arch -x86_64 g++ -o cppexample easywsclient.o main.cc lib/libextra_cpp_bindings.a -std=c++14 $(FLAGS)

run_static:
	./cppexamplestatic

run_dynamic:
	export LD_LIBRARY_PATH=$(PWD)/lib && ./cppexample

clean:
	rm cppexample cppexamplestatic
