const BRIDGES: &[&str] = &["src/lib.rs"];

fn main() {
    cxx_build::bridges(BRIDGES)
        .file("src/pay.cc")
        .flag_if_supported("-std=c++14")
        .compile("game_sdk_bindings");

    for bridge in BRIDGES {
        println!("cargo:rerun-if-changed={}", bridge);
    }

    println!("cargo:rerun-if-changed=src/pay.cc");
    println!("cargo:rerun-if-changed=include/pay.h");
}
