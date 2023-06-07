# FunctionalRust
![My image](img/rust.jpg)
![My image](img/lambda.jpg)

Author Pablo Picouto Garcia

Is this repo useful? Please ⭑Star this repository and share the love.

Examples of functional programing in Rust

## Features

* **[Functions](src/features/functions.rs)**
* **[Type Classes](src/features/type_classes.rs)**

* **[Effect system](src/features/effect_system.rs)**
* **[Extension method](src/features/extension_method.rs)**
* **[Memory management](src/features/memory_management.rs)**
* **[Collection](src/features/collection.rs)**
* **[async-await](src/features/async_programming.rs)**
* **[channels](src/features/channels_feature.rs)**
* **[promise](src/features/promise.rs)**
* **[pattern matching](src/features/pattern_matching.rs)**
* **[smart pointer](src/features/smart_pointer.rs)**
* **[NewTypes](src/features/new_types.rs)**
* **[Do notation](src/features/do_notation_style.rs)**
* **[Union type](src/features/union_type.rs)**
* **[Lens](src/features/lens.rs)**


## Monads

Implementation of some of the most famous Monads you can find in ```Haskel``` or ```Scala```

* **[IO Monad](src/features/rust_io.rs)**
* **[Monad](src/features/monad.rs)**
* **[Try Monad](src/features/try_monad.rs)**
* **[Either Monad](src/features/either_monad.rs)**

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

## Red Panda

![My image](img/red_panda.png)

Code example of **Red Panda** ```consumer/producer``` and benchmark result ```producing/consuming``` 1000 records with 100kb size.

* **[Consumer](red_panda/src/red_panda_consumer.rs)**
* **[Producer](red_panda/src/red_panda_producer.rs)**

## Performance

### Goose

Goose is a Rust load testing tool inspired by Locust. 

* **[Simulation](goose/src/goose_load_test.rs)**
* **[Mock server](goose/src/mock_http_server.rs)**

#### Benchmark 

```
 === PER REQUEST METRICS ===
 ------------------------------------------------------------------------------
 Name                     |        # reqs |        # fails |    req/s |  fail/s
 ------------------------------------------------------------------------------
 GET /hello               |       435,482 |         0 (0%) |     3629 |    0.00
 ------------------------------------------------------------------------------
 Name                     |    Avg (ms) |        Min |         Max |     Median
 ------------------------------------------------------------------------------
 GET /hello               |        3.90 |          1 |         103 |          4
 ------------------------------------------------------------------------------
 Slowest page load within specified percentile of requests (in ms):
 ------------------------------------------------------------------------------
 Name                     |    50% |    75% |    98% |    99% |  99.9% | 99.99%
 ------------------------------------------------------------------------------
 GET /hello               |      4 |      5 |      8 |     10 |     16 |     23
 ------------------------------------------------------------------------------
 Name                     |                                        Status codes 
 ------------------------------------------------------------------------------
 GET /hello               |                                       435,482 [200]
 -------------------------+----------------------------------------------------
 Aggregated               |                                       435,482 [200] 

 === OVERVIEW ===
 ------------------------------------------------------------------------------

```
