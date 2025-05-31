#!/bin/bash
# Build script for hybrid C/Rust Zopfli implementation

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$( cd "$SCRIPT_DIR/.." && pwd )"

echo "Building Zopfli hybrid C/Rust implementation..."

# Build C library
echo "Building C library..."
cd "$PROJECT_ROOT"
make clean
make -j$(nproc 2>/dev/null || echo 4)

# Build Rust library with C fallback
echo "Building Rust library with C fallback..."
cd "$PROJECT_ROOT/zopfli-rs"
cargo build --release

# Build Rust library in pure-rust mode (if available)
echo "Building Rust library in pure-rust mode (may fail if not all functions ported)..."
cargo build --release --no-default-features --features pure-rust || echo "Pure Rust build not yet complete"

# Build fuzz targets
echo "Building fuzz targets..."
cd "$PROJECT_ROOT/zopfli-rs/fuzz"
cargo +nightly fuzz build

# Create a simple CLI wrapper for testing
echo "Creating test CLI..."
cd "$PROJECT_ROOT/zopfli-rs"
cat > src/bin/zopfli-rs.rs << 'EOF'
use std::env;
use std::fs;
use std::io::{self, Read, Write};
use zopfli::{ZopfliOptions, OutputType};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <file> [--gzip|--zlib|--deflate]", args[0]);
        return Ok(());
    }
    
    let filename = &args[1];
    let output_type = if args.len() > 2 {
        match args[2].as_str() {
            "--gzip" => OutputType::Gzip,
            "--zlib" => OutputType::Zlib,
            "--deflate" => OutputType::Deflate,
            _ => OutputType::Gzip,
        }
    } else {
        OutputType::Gzip
    };
    
    let data = fs::read(filename)?;
    let options = ZopfliOptions::default();
    let compressed = zopfli::compress(&options, output_type, &data);
    
    let out_filename = match output_type {
        OutputType::Gzip => format!("{}.gz", filename),
        OutputType::Zlib => format!("{}.zlib", filename),
        OutputType::Deflate => format!("{}.deflate", filename),
    };
    
    fs::write(&out_filename, compressed)?;
    println!("Compressed {} -> {}", filename, out_filename);
    
    Ok(())
}
EOF

# Add bin to Cargo.toml
grep -q "bin]" Cargo.toml || cat >> Cargo.toml << 'EOF'

[[bin]]
name = "zopfli-rs"
path = "src/bin/zopfli-rs.rs"
EOF

cargo build --release --bin zopfli-rs

echo "Build complete!"
echo ""
echo "Binaries available at:"
echo "  C version: $PROJECT_ROOT/zopfli"
echo "  Rust version: $PROJECT_ROOT/zopfli-rs/target/release/zopfli-rs"
echo ""
echo "To run fuzzers:"
echo "  cd $PROJECT_ROOT/zopfli-rs/fuzz"
echo "  cargo +nightly fuzz run fuzz_differential"