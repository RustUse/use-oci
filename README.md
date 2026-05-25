# RustUse/use-oci

`use-oci` is a RustUse facade workspace for small, focused Open Container Initiative primitive crates. It models OCI image, distribution, layout, and runtime-adjacent identifiers, descriptors, media types, digests, platforms, annotations, references, and metadata without becoming an implementation of an OCI system.

It is not Docker, Kubernetes, a container runtime, a registry client, an image builder, an HTTP client, or a command runner. The first release wave is intentionally small: parsers, validators, normalized types, builders, enums, identifiers, and rendering helpers that do not shell out or make network calls.

## Sibling relationships

- `use-oci` models standards-shaped OCI primitives.
- `use-docker` remains a separate sibling facade for Docker-specific developer workflow primitives.
- `use-kubernetes` should remain a separate sibling facade for orchestration primitives if it is added later.
- Docker-related RustUse crates may reuse OCI primitives where the underlying concept is technically OCI-shaped.

## Workspace crates

| Crate                  | Path                           | Purpose                                                          |
| ---------------------- | ------------------------------ | ---------------------------------------------------------------- |
| `use-oci`              | `crates/use-oci/`              | Feature-gated facade over focused OCI primitive crates           |
| `use-oci-image`        | `crates/use-oci-image/`        | Image name, reference, ID, kind, and metadata composition        |
| `use-oci-manifest`     | `crates/use-oci-manifest/`     | Image manifest descriptor lists and annotations                  |
| `use-oci-index`        | `crates/use-oci-index/`        | Multi-platform image index entries and metadata                  |
| `use-oci-descriptor`   | `crates/use-oci-descriptor/`   | Descriptor metadata shared by image documents                    |
| `use-oci-layer`        | `crates/use-oci-layer/`        | Layer media, compression, size, and diff ID metadata             |
| `use-oci-config`       | `crates/use-oci-config/`       | Image config command, env, port, label, and volume metadata      |
| `use-oci-layout`       | `crates/use-oci-layout/`       | Layout version markers and lexical path helpers                  |
| `use-oci-annotation`   | `crates/use-oci-annotation/`   | Annotation keys, values, pairs, and common names                 |
| `use-oci-media-type`   | `crates/use-oci-media-type/`   | Known and custom OCI media type rendering                        |
| `use-oci-platform`     | `crates/use-oci-platform/`     | OS, architecture, variant, version, and feature metadata         |
| `use-oci-digest`       | `crates/use-oci-digest/`       | Digest algorithm, value, and `algorithm:value` parsing           |
| `use-oci-distribution` | `crates/use-oci-distribution/` | Registry, repository, blob, manifest, tag, and route identifiers |
| `use-oci-reference`    | `crates/use-oci-reference/`    | Image reference syntax with tag and digest composition           |
| `use-oci-runtime`      | `crates/use-oci-runtime/`      | Runtime process, mount, namespace, hook, and rootfs metadata     |
| `use-oci-hook`         | `crates/use-oci-hook/`         | Runtime hook lifecycle metadata without execution                |
| `use-oci-namespace`    | `crates/use-oci-namespace/`    | Namespace kind and lexical namespace path metadata               |
| `use-oci-tag`          | `crates/use-oci-tag/`          | Tag validation and safe tag classification                       |

## Installation

Use the workspace directly or depend on a Git revision until the first crates.io release is published.

```toml
[dependencies]
use-oci = { git = "https://github.com/RustUse/use-oci", rev = "<commit>" }
```

After publication, choose the narrowest focused crate that matches your use case or use the facade when one dependency is more convenient.

```toml
[dependencies]
use-oci = "0.0.1"
```

## Basic usage

```rust
# #[cfg(feature = "full")]
# {
use use_oci::{annotation, descriptor, digest, index, manifest, media_type, platform, reference};

let parsed_digest: digest::OciDigest = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".parse()?;
let media_type = media_type::OciMediaType::image_manifest();
let descriptor = descriptor::OciDescriptor::new(
    media_type.clone(),
    parsed_digest.clone(),
    descriptor::DescriptorSize::new(7023),
);
let linux_arm64 = platform::OciPlatform::new(platform::OciOs::Linux, platform::OciArchitecture::Arm64);
let title = annotation::Annotation::title("RustUse OCI example")?;
let manifest = manifest::OciManifest::new(descriptor.clone()).with_layer(descriptor.clone());
let index_entry = index::IndexManifest::new(descriptor.clone()).with_platform(linux_arm64.clone());
let image_ref: reference::ImageReference = "ghcr.io/rustuse/app:0.1.0@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".parse()?;

assert_eq!(parsed_digest.algorithm().as_str(), "sha256");
assert_eq!(descriptor.media_type().to_string(), media_type.to_string());
assert_eq!(linux_arm64.to_string(), "linux/arm64");
assert_eq!(title.key().as_str(), annotation::OCI_TITLE);
assert_eq!(manifest.layers().len(), 1);
assert_eq!(index_entry.platform(), Some(&linux_arm64));
assert_eq!(media_type.to_string(), "application/vnd.oci.image.manifest.v1+json");
assert_eq!(image_ref.tag().map(reference::TagName::as_str), Some("0.1.0"));
# }
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Scope

`use-oci` keeps focused crates small, dependency-light, deterministic, and framework-free. The workspace favors validated newtypes, explicit error enums, stable display labels, small builders, and conservative parsing helpers.

## Non-goals

- No Docker CLI, Docker socket, or Docker engine behavior.
- No Kubernetes orchestration behavior.
- No container runtime execution.
- No image build, pull, push, unpack, mount, or container lifecycle behavior.
- No live registry HTTP API calls in v0.1.
- No full OCI JSON parser or serializer in v0.1.
- No tar, gzip, zstd, or filesystem extraction behavior.

## Experimental

`use-oci` is experimental while the workspace remains below `0.3.0`. Expect small API adjustments during the first release wave.

## Development

```sh
cargo fmt --all -- --check
cargo check --workspace --all-features
cargo check --workspace --all-features --examples
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo test --workspace --no-default-features
```

## License

Licensed under either of the following, at your option:

- Apache License, Version 2.0
- MIT license
