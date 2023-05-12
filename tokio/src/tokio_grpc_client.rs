use hello_world::greeter_client::GreeterClient;
use hello_world::HelloRequest;

/**
For every change in grpc_service.proto you need to run the command to re-build the code with protoc [cargo run --bin tokio-grpc--client]
Using [include_proto] we specify the [package] name we use in proto file
 */
pub mod hello_world {
    tonic::include_proto!("helloworld");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = GreeterClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(HelloRequest {
        name: "Tonic".into(),
    });

    let response = client.say_hello(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}