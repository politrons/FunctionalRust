use tonic::{transport::Server, Request, Response, Status};
use tonic::transport::Error;

use grpc_service::my_grpc_server::{MyGrpc, MyGrpcServer};
use grpc_service::{MyGrpcResponse, MyGrpcRequest};

/**
For every change in grpc_service.proto you need to run the command to re-build the code with protoc [cargo run --bin tokio-grpc-server]
Using [include_proto] we specify the [package] name we use in proto file
 */
pub mod grpc_service {
    tonic::include_proto!("grpcservice"); // The string specified here must match the proto package name
}

/**
Struct type to be used to implement the gRPC service.
*/
#[derive(Debug, Default)]
pub struct MyGrpcService {}

/**
gRPC [service] implementation defined in [protobuf]. We have to implemented using a struct type.

We use annotation [async_trait] to specify the gRPC communication is [asynchronous], so the implementation it can be
async, and the client it will receive a [Future].

As a request, we receive a [Request<MyGrpcRequest>] type where [MyGrpcRequest] is the message defined in the protobuf
We return a Result of [Response<MyGrpcResponse>] type, where also [MyGrpcResponse] is the type defined in the protobuf.

For the successful value we return [Response<MyGrpcResponse>] , and [Status] for Error channel.
 */
#[tonic::async_trait]
impl MyGrpc for MyGrpcService {
    async fn simple_request(&self, request: Request<MyGrpcRequest>) -> Result<Response<MyGrpcResponse>, Status> {
        println!("Message received: {:?}", request);
        let response = create_response(request);
        Ok(Response::new(response))
    }
}

/**
Creation of gRPC response type extracting [name] attribute from request message.
We must use [into_inner] as the fields of gRPC requests and responses are private
*/
fn create_response(request: Request<MyGrpcRequest>) -> MyGrpcResponse {
    MyGrpcResponse {
        message: format!("How you doing {}!", request.into_inner().name).into(),
    }
}


/**
gRPC Server implementation.
We use [Server] builder and we use [add_service] to specify the service implementation.
Using Service name [Greeter][Service] we pass the instance of the service [MyGrpcService].
We use [serve] to pass the address in one particular port.
Finally we use await to keep the server running until shutdown is received or process is over.
 */
#[tokio::main]
async fn main() -> Result<(), Error> {
    let addr = "[::1]:9100".parse().unwrap();
    let service = MyGrpcService::default();
    println!("gRPC server running on port 9100.....");
    return match Server::builder()
        .add_service(MyGrpcServer::new(service))
        .serve(addr)
        .await {
        Ok(_) => {
            println!("gRPC server shutdown");
            Ok(())
        }
        Err(e) => {
            println!("Error gRPC server");
            Err(e)
        }
    };
}
