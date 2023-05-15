# FunctionalRust
![My image](img/rust.jpg)
![My image](img/lambda.jpg)

Author Pablo Perez Garcia

Is this repo useful? Please ⭑Star this repository and share the love.

Examples of functional programing in Rust

## Features

* **[Functions](src/features/functions.rs)**
* **[Type Classes](src/features/type_classes.rs)**
* **[Monad](src/features/monad.rs)**
* **[Try Monad](src/features/try_monad.rs)**
* **[Either Monad](src/features/either_monad.rs)**
* **[Effect system](src/features/effect_system.rs)**
* **[Extension method](src/features/extension_method.rs)**
* **[Memory management](src/features/memory_management.rs)**
* **[Collection](src/features/collection.rs)**
* **[async-await](src/features/async_programming.rs)**
* **[channels](src/features/channels_feature.rs)**
* **[pattern matching](src/features/pattern_matching.rs)**
* **[smart pointer](src/features/smart_pointer.rs)**
* **[NewTypes](src/features/new_types.rs)**

## Tokio

#### Features

* **[select](tokio/src/tokio_select.rs)**

#### Hyper

Fast and safe HTTP for the Rust language.

* **[Client/Server](tokio/src/tokio_http_hyper.rs)**

#### Tonic

A gRPC over HTTP/2 implementation focused on high performance, interoperability, and flexibility.

* **[Protobuf](tokio/proto/grpc_service.proto)**
* **[Client](tokio/src/tokio_grpc_client.rs)**
* **[Server](tokio/src/tokio_grpc_server.rs)**

## Actix

![My image](img/actix-web.png)

Actix Web is a powerful, pragmatic, and extremely fast web framework for Rust.

* **[Server](actix/src/actix_server.rs)**
* **[Web-Server](actix/src/actix_web_server.rs)**

## Kafka

![My image](img/kafka.png)

Code example of **Kafka** ```consumer/producer``` and benchmark result ```producing/consuming``` 1000 records with 100kb size.

* **[Consumer](kafka/src/kafka_consumer.rs)**
* **[Producer](kafka/src/kafka_producer.rs)**
