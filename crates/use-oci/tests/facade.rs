use use_oci::{
    annotation, config, descriptor, digest, distribution, hook, image, index, layer, layout,
    manifest, media_type, namespace, platform, reference, runtime, tag,
};

#[test]
fn facade_exposes_all_oci_namespaces() -> Result<(), Box<dyn std::error::Error>> {
    let parsed_digest: digest::OciDigest =
        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".parse()?;
    let descriptor = descriptor::OciDescriptor::new(
        media_type::OciMediaType::image_manifest(),
        parsed_digest.clone(),
        descriptor::DescriptorSize::new(7023),
    );
    let platform =
        platform::OciPlatform::new(platform::OciOs::Linux, platform::OciArchitecture::Amd64);
    let annotation = annotation::Annotation::title("RustUse")?;
    let manifest = manifest::OciManifest::new(descriptor.clone()).with_layer(descriptor.clone());
    let index_entry = index::IndexManifest::new(descriptor.clone()).with_platform(platform.clone());
    let repository = distribution::RepositoryName::new("rustuse/app")?;
    let manifest_reference = distribution::ManifestReference::Digest(parsed_digest.clone());
    let route = distribution::DistributionRoute::manifest(&repository, &manifest_reference);
    let reference: reference::ImageReference =
        "ghcr.io/rustuse/app:latest@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".parse()?;
    let runtime_spec = runtime::RuntimeSpec::new(runtime::RootFilesystem::new("rootfs")?)
        .with_namespace(namespace::NamespaceKind::Pid)
        .with_hook(hook::OciHook::new(
            hook::HookKind::Prestart,
            hook::HookPath::new("/bin/check")?,
        ));
    let layer = layer::OciLayer::new(
        layer::LayerMediaType::gzip_tar(),
        parsed_digest,
        layer::LayerSize::new(7023),
    );
    let config =
        config::OciImageConfig::new(platform::OciArchitecture::Amd64, platform::OciOs::Linux)
            .with_env(config::EnvVar::new("RUST_LOG", "info")?);
    let layout = layout::OciLayoutPaths::new("oci-layout");
    let image = image::ImageMetadata::new(image::ImageName::new("rustuse/app")?)
        .with_descriptor(descriptor);
    let tag = tag::OciTag::latest();

    assert_eq!(annotation.key().as_str(), annotation::OCI_TITLE);
    assert_eq!(manifest.layers().len(), 1);
    assert_eq!(index_entry.platform(), Some(&platform));
    assert_eq!(
        route.path(),
        "/v2/rustuse/app/manifests/sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    );
    assert!(reference.tag().is_some());
    assert_eq!(runtime_spec.namespaces(), &[namespace::NamespaceKind::Pid]);
    assert_eq!(layer.size().as_u64(), 7023);
    assert_eq!(config.env().len(), 1);
    assert!(layout.index_file().ends_with("index.json"));
    assert_eq!(image.descriptors().len(), 1);
    assert!(tag.is_latest());
    Ok(())
}
