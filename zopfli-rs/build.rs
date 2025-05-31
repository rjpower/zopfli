#[cfg(feature = "c-fallback")]
fn main() {
    use cc::Build;
    
    println!("cargo:rerun-if-changed=src/symbols_wrapper.c");
    println!("cargo:rerun-if-changed=../src/zopfli/symbols.h");
    println!("cargo:rerun-if-changed=../src/zopfli/util.c");
    println!("cargo:rerun-if-changed=../src/zopfli/util.h");
    println!("cargo:rerun-if-changed=../src/zopfli/tree.c");
    println!("cargo:rerun-if-changed=../src/zopfli/tree.h");
    println!("cargo:rerun-if-changed=../src/zopfli/katajainen.c");
    println!("cargo:rerun-if-changed=../src/zopfli/katajainen.h");
    println!("cargo:rerun-if-changed=../src/zopfli/zopfli.h");
    
    Build::new()
        .file("src/symbols_wrapper.c")
        .file("../src/zopfli/util.c")
        .file("../src/zopfli/tree.c")
        .file("../src/zopfli/katajainen.c")
        .include("../src/zopfli")
        .compile("zopfli_c");
}

#[cfg(not(feature = "c-fallback"))]
fn main() {
    // Nothing to do for pure Rust build
}