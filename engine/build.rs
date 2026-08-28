fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use the bundled protoc binary so users don't need to install protobuf-compiler.
    let protoc = protoc_bin_vendored::protoc_bin_path()
        .expect("protoc-bin-vendored: could not find bundled protoc");
    std::env::set_var("PROTOC", &protoc);

    // Compile the Mouser gRPC proto schema into Rust server + client stubs.
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(&["proto/mouser.proto"], &["proto"])?;
    Ok(())
}
