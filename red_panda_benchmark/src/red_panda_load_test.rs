use std::env;
use goose::prelude::*;
use goose_eggs::{Validate, validate_and_load_static_assets};


#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let result = match args.get(0) {
        Some(x) if x == "produce" => produce().await,
        Some(x) if x == "produce_consume" => produce_and_consume().await,
        _ => produce().await
    };
    println!("{:?}", result)
}

/// Simulation of Goose where we can define the [transactions] that we want to run and what configuration
/// we want to have for the transactions.
/// We use [GooseAttack::initialize()] to start the builder, then we can register transactions,
/// where we can pass multiple [TransactionResult]
/// Once we have all transactions configured, we can configure the simulation overriding
/// [GooseDefault] using [set_default] operator.
pub async fn produce() -> Result<(), GooseError> {
    println!("Running produce red panda records....");
    GooseAttack::initialize()?
        .register_scenario(scenario!("Produce Red panda records")
            .register_transaction(transaction!(produce_request)))
        .set_default(GooseDefault::Host, "http://127.0.0.1:1981")?
        .set_default(GooseDefault::Users, 2)?
        .set_default(GooseDefault::StartupTime, 10)?
        .set_default(GooseDefault::RunningMetrics, 5)?
        .set_default(GooseDefault::RunTime, 120)?
        .execute()
        .await?;
    Ok(())
}

pub async fn produce_and_consume() -> Result<(), GooseError> {
    println!("Running produce and consume red panda records....");
    GooseAttack::initialize()?
        .register_scenario(scenario!("Produce and consume Red panda records")
            .register_transaction(transaction!(produce_and_consume_request)))
        .set_default(GooseDefault::Host, "http://127.0.0.1:1981")?
        .set_default(GooseDefault::Users, 2)?
        .set_default(GooseDefault::StartupTime, 10)?
        .set_default(GooseDefault::RunningMetrics, 5)?
        .set_default(GooseDefault::RunTime, 120)?
        .execute()
        .await?;
    Ok(())
}

/// [TransactionResult] is the definition of how we make the call to the service endpoint, and
/// how we validate the response.
/// Using from [goose-eggs] dependency [Validate] we can create a validate instance, to be used
/// to compare with [GooseResponse] from the call.
/// We can check what is the [status], [text] from the body is what we expect.
///
async fn produce_request(user: &mut GooseUser) -> TransactionResult {
    let goose = user.get("/panda/produce").await?;
    let validate = &Validate::builder()
        .status(200)
        .build();
    validate_and_load_static_assets(user, goose, &validate).await?;
    Ok(())
}

async fn produce_and_consume_request(user: &mut GooseUser) -> TransactionResult {
    let goose = user.get("/panda/produce_consume").await?;
    let validate = &Validate::builder()
        .status(200)
        .build();
    validate_and_load_static_assets(user, goose, &validate).await?;
    Ok(())
}
