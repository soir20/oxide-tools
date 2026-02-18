# oxide-string-ids
Brute‑force script to recover string IDs from 32‑bit Jenkins lookup2 hashes found in SOE .dat locale files.

# Prerequisites
Requires [Rust](https://rust-lang.org/tools/install/) to build (`cargo build`) and run (`cargo run`) the script.

# Usage
Run the script with your input file, output file, and optional ID range:
`cargo run --release -- <input.dat> <output.txt> --range-start 0 --range-end 5000000`
By default, the range is `0` to `u32::MAX`.