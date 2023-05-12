use tonic::{transport::Server, Request, Response, Status};

use grpc_service::greeter_server::{Greeter, GreeterServer};
use grpc_service::{HelloReply, HelloRequest};

/**
For every change in grpc_service.proto you need to run the command to re-build the code with protoc [cargo run --bin tokio-grpc-server]
Using [include_proto] we specify the [package] name we use in proto file
*/
pub mod grpc_service {
    tonic::include_proto!("grpcservice"); // The string specified here must match the proto package name
}

#[derive(Debug, Default)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>, // Accept request of type HelloRequest
    ) -> Result<Response<HelloReply>, Status> { // Return an instance of type HelloReply
        println!("Got a request: {:?}", request);

        let reply = grpc_service::HelloReply {
            message: format!("Hello {}!", request.into_inner().name).into(), // We must use .into_inner() as the fields of gRPC requests and responses are private
        };

        Ok(Response::new(reply)) // Send back our formatted greeting
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let greeter = MyGreeter::default();

    Server::builder()
        .add_service(GreeterServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}
