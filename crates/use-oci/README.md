# RustUse/use-oci

`use-oci` is a RustUse facade crate for small, focused Open Container Initiative primitive crates. It re-exports focused OCI image, distribution, layout, and runtime-adjacent crates with explicit aliases.

It is not Docker, Kubernetes, a container runtime, a registry client, an image builder, an HTTP client, or a command runner.

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

## Facade aliases

The facade re-exports child crates with explicit aliases: `image`, `manifest`, `index`, `descriptor`, `layer`, `config`, `layout`, `annotation`, `media_type`, `platform`, `digest`, `distribution`, `reference`, `runtime`, `hook`, `namespace`, and `tag`.

## Non-goals

- No Docker-specific workflow behavior.
- No Kubernetes orchestration behavior.
- No runtime execution.
- No registry HTTP client.
- No image builder.

## License

Licensed under either of the following, at your option:

- Apache License, Version 2.0
- MIT license
