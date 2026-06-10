fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Tell Cargo to rerun this build script if the proto file changes
    println!("cargo:rerun-if-changed=../proto/intelligence/v1/intelligence.proto");

    tonic_prost_build::configure()
        .type_attribute("forkfit.intelligence.v1.IntentResponse", "#[derive(serde::Serialize, serde::Deserialize)]")
        .compile_protos(
            &["../proto/intelligence/v1/intelligence.proto"],
            &["../proto"],
        )?;
    Ok(())
}
