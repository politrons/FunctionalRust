use tonic::Request;
use grpc_service::my_grpc_client::MyGrpcClient;
use grpc_service::MyGrpcRequest;

/**
For every change in grpc_service.proto you need to run the command to re-build the code with protoc [cargo run --bin tokio-grpc--client]
Using [include_proto] we specify the [package] name we use in proto file
 */
pub mod grpc_service {
    tonic::include_proto!("grpcservice");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = MyGrpcClient::connect("http://[::1]:9100").await?;
    let request = create_request();
    let response = client.simple_request(request).await;
    println!("Response={:?}", response);
    Ok(())
}

fn create_request() -> Request<MyGrpcRequest> {
    Request::new(MyGrpcRequest {
        name: "Politrons".into(),
    })
}