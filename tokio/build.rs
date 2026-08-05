/**
gRPC main class responsible to build the code defined in [proto] file, using [protoc] command
*/
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let protoc = protoc_bin_vendored::protoc_bin_path()?;
    std::env::set_var("PROTOC", protoc);

    tonic_build::compile_protos("proto/grpc_service.proto")?;
    Ok(())
}
