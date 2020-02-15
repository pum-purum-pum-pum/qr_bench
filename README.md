# Cargo install
for installing cargo run (or read)
```bash install_deps.sh```

# Python bindings
Once you've got cargo on your machine
```
cargo build --release
cp target/release/libqr_searcher.so ./qr_searcher.so
python example.py
```

This will produce `.so` dynamic library you can easily use from `Python`. See `example.py` as an example.

# CLI
To make binary run:

```bash produce_binary.sh```

this will produce produce binary called `qr_searcher`

After that run `./qr_searcher -h`

for details