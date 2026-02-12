# Bullshark & Sparse bullshark simulations

How to run:

```
export RAYON_NUM_THREADS=<how many threads to run simulations>
RUST_LOG=info cargo run --bin bullshark --package=dag-based --release
RUST_LOG=info cargo run --bin sparse_bullshark --package=dag-based --release
```

Need to wait few hours simulation to complete.
Can consume a lot of memory, up to 100Gb (depends on RAYON_NUM_THREADS)

Python scripts that build plots located in `./systems/dag-based/src/bin/versus/`
