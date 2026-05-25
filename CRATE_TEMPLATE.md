# RustUse OCI Crate Template

Use this checklist when adding a focused crate to the `use-oci` workspace.

- Keep package metadata inherited from the workspace wherever possible.
- Prefer no dependencies for primitive vocabulary crates.
- Use README-driven crate docs with `#![doc = include_str!("../README.md")]`.
- Use explicit error enums instead of stringly errors.
- Add unit tests for accepted and rejected values.
- Keep Docker-specific behavior, network access, runtime execution, registry clients, and image builder behavior out of scope.
