/**
gRPC main class responsible to build the code defined in [proto] file, using [protoc] command
*/
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/grpc_service.proto")?;
    Ok(())
}