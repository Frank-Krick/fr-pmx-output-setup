fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile(
        &["../fr-pmx-registry/proto/registry.proto"],
        &["../fr-pmx-registry/"],
    )?;
    tonic_build::configure().compile(
        &["../fr-pipewire-registry/proto/pipewire.proto"],
        &["../fr-pipewire-registry/"],
    )?;
    Ok(())
}
