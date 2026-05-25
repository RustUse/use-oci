use use_oci::{annotation, descriptor, digest, media_type, platform, reference};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let parsed_digest: digest::OciDigest =
        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".parse()?;
    let descriptor = descriptor::OciDescriptor::new(
        media_type::OciMediaType::image_manifest(),
        parsed_digest,
        descriptor::DescriptorSize::new(7023),
    );
    let linux_arm64 =
        platform::OciPlatform::new(platform::OciOs::Linux, platform::OciArchitecture::Arm64);
    let title = annotation::Annotation::title("RustUse OCI example")?;
    let image_ref: reference::ImageReference =
        "ghcr.io/rustuse/app:0.1.0@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".parse()?;

    assert_eq!(descriptor.size().as_u64(), 7023);
    assert_eq!(linux_arm64.to_string(), "linux/arm64");
    assert_eq!(title.value().as_str(), "RustUse OCI example");
    assert_eq!(image_ref.repository().as_str(), "rustuse/app");
    Ok(())
}
