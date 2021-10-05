# big-brother

### static build with `musl-libc`
```sh
rustup target add "x86_64-unknown-linux-musl"
# https://github.com/rust-lang/rust/issues/71651#issuecomment-864265118
RUSTC="$PWD/rustc.wrap" cargo build --release --target "x86_64-unknown-linux-musl"
strip target/x86_64-unknown-linux-musl/release/big-brother # optional
```
