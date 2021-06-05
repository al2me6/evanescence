build-web-debug:
    # Use the `dev` flag and force optimization and debug assertions to skip `wasm-opt`, which strips debug info.
    RUSTFLAGS="-Copt-level=3 -Cdebug-assertions -g" wasm-pack build --target web --out-name wasm --out-dir static evanescence_web --no-typescript --dev

build-web:
    wasm-pack build --target web --out-name wasm --out-dir static evanescence_web --no-typescript
