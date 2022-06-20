#!/usr/bin/env python3
import fnmatch
import os
import shutil
from pathlib import Path

EXAMPLE_SOURCES = [
    "../defi-wallet-core-rs/example/cpp-example/chainmain.cc",
    "../defi-wallet-core-rs/example/cpp-example/cronos.cc",
    "../defi-wallet-core-rs/example/cpp-example/chainmain.h",
    "../defi-wallet-core-rs/example/cpp-example/cronos.h",
]

SOURCES = [
    "../extra-cpp-bindings/include/pay.h",
    "../extra-cpp-bindings/src/pay.cc",
    "../extra-cpp-bindings/include/walletconnectcallback.h",
    "../extra-cpp-bindings/src/walletconnectcallback.cc",
    "../defi-wallet-core-rs/bindings/cpp/src/nft.cc",
    "../defi-wallet-core-rs/bindings/cpp/include/nft.h",
]
INCLUDE_PATH = "./include"
LIB_PATH = "./lib"

INITIAL_INCLUDES = [
    '#include "extra-cpp-bindings/src/lib.rs.h"',
    '#include "extra-cpp-bindings/include/pay.h"',
    '#include "extra-cpp-bindings/include/walletconnectcallback.h"',
    '#include "defi-wallet-core-cpp/src/lib.rs.h"',
    '#include "defi-wallet-core-cpp/src/uint.rs.h"',
    '#include "defi-wallet-core-cpp/include/nft.h"',
]
FINAL_INCLUDES = [
    '#include "lib.rs.h"',
    '#include "../../pay.h"',
    '#include "../../walletconnectcallback.h"',
    '#include "lib.rs.h"',
    '#include "uint.rs.h"',
    '#include "../../nft.h"',
]

INITIAL_SOURCES_INCLUDES = [
    '#include "extra-cpp-bindings/include/pay.h"',
    '#include "extra-cpp-bindings/include/walletconnectcallback.h"',
    '#include "defi-wallet-core-cpp/include/nft.h"',
]
FINAL_SOURCES_INCLUDES = [
    '#include "pay.h"',
    '#include "walletconnectcallback.h"',
    '#include "nft.h"',
]


# the path of output target, defined by --target_dir
TARGET_DIR = None

# the path of cxxbridge artifacts
OUT_DIR = None


# copy the generated binding files: `*.cc` and `*.h` to `output_path`
def copy_cxxbridge(output_path):
    files = []
    files.extend(collect_files("*.h", OUT_DIR))
    files.extend(collect_files("*.cc", OUT_DIR))

    def has_include_string(s):
        for include in INITIAL_INCLUDES:
            if include in s:
                return True
        return False

    # replace string
    for filename in files:
        # Safely read the input filename using 'with'
        with open(filename) as f:
            s = f.read()
            if not has_include_string(s):
                continue

        # Safely write the changed content, if found in the file
        with open(filename, "w") as f:
            for i, include in enumerate(INITIAL_INCLUDES):
                s = s.replace(include, FINAL_INCLUDES[i])
            f.write(s)

    # copy the bindings, need python 3.8+
    shutil.copytree(OUT_DIR, output_path, dirs_exist_ok=True)

    # move lib.rs.cc of defi-wallet-core-cpp to core.cc to avoid name collision
    # (extra-cpp-bindings also has a lib.rs.cc)
    shutil.move(
        Path(__file__).parent / output_path / "defi-wallet-core-cpp/src/lib.rs.cc",
        Path(__file__).parent / output_path / "defi-wallet-core-cpp/src/core.cc",
    )


# copy library files: `*.a`, `*.dylib`, `*.lib` (windows), `*.dll` (windows) to `output_path`
def copy_lib_files(output_path):
    os.makedirs(output_path, exist_ok=True)
    files = []
    files.extend(collect_files("*.a", TARGET_DIR, recursive=False))
    files.extend(collect_files("*.dylib", TARGET_DIR, recursive=False))
    files.extend(collect_files("*.lib", TARGET_DIR, recursive=False))
    files.extend(collect_files("*.dll", TARGET_DIR, recursive=False))
    # workaround: search libcxxbridge1.a and push the first one
    files.append(collect_files("libcxxbridge1.a", TARGET_DIR)[0])

    # copy the libraries, need python 3.8+
    for f in files:
        shutil.copy(f, output_path)


# copy `EXAMPLE_SOURCES` to `output_path`
def copy_example_files(output_path):
    for f in EXAMPLE_SOURCES:
        shutil.copy(f, output_path)


# copy `SOURCES` to `output_path`
def copy_sources_files(output_path):
    for f in SOURCES:
        shutil.copy(f, output_path)
    files = []
    files.extend(collect_files("*.h", output_path, recursive=False))
    files.extend(collect_files("*.cc", output_path, recursive=False))

    def has_include_string(s):
        for include in INITIAL_SOURCES_INCLUDES:
            if include in s:
                return True
        return False

    # replace string
    for filename in files:
        # Safely read the input filename using 'with'
        with open(filename) as f:
            s = f.read()
            if not has_include_string(s):
                continue

        # Safely write the changed content, if found in the file
        with open(filename, "w") as f:
            for i, include in enumerate(INITIAL_SOURCES_INCLUDES):
                s = s.replace(include, FINAL_SOURCES_INCLUDES[i])
            f.write(s)


# collect files with `pattern` in `search path`, and return the matched files
def collect_files(pattern, search_path, recursive=True):
    result = []
    if recursive:
        for root, dirs, files in os.walk(search_path):
            for name in files:
                if fnmatch.fnmatch(name, pattern):
                    result.append(os.path.join(root, name))
    else:
        for f in os.listdir(search_path):
            # if os.path.isfile(os.path.join(search_path, f)):
            if fnmatch.fnmatch(f, pattern):
                result.append(os.path.join(search_path, f))

    return result


if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(
        description="Generate bindings for the C++ example."
    )
    parser.add_argument(
        "--target_dir",
        default="../target/release",
        metavar="path",
        help="path to target dir",
    )
    args = parser.parse_args()
    TARGET_DIR = args.target_dir

    OUT_DIR = Path(TARGET_DIR).parent / "cxxbridge"

    if os.path.exists(TARGET_DIR):
        print("TARGET_DIR= ", TARGET_DIR)
        print("OUT_DIR= ", OUT_DIR)
        copy_cxxbridge(INCLUDE_PATH)
        copy_lib_files(LIB_PATH)
        copy_sources_files(INCLUDE_PATH)

    copy_example_files(".")
