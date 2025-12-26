fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Compile the proto file
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir("src/grpc")
        .compile_protos(&["proto/metering.proto"], &["proto"])?;

    // Tell Cargo to rerun if proto changes
    println!("cargo:rerun-if-changed=proto/metering.proto");

    Ok(())
}
