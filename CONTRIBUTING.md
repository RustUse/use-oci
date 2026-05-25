# Contributing

Thanks for helping improve `use-oci`.

Keep contributions small, dependency-light, framework-free, and focused on OCI primitive types, validators, display helpers, and metadata models. Avoid network calls, runtime execution, Docker-specific workflow behavior, full registry clients, and image builder behavior.

Before opening a pull request, run:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```
