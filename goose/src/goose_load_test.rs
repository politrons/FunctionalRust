use goose::prelude::*;
use std::time::Duration;
use goose::config::GooseConfiguration;
use goose_eggs::{validate_and_load_static_assets, Validate};

pub async fn run() -> Result<(), GooseError> {
    GooseAttack::initialize()?
        .register_scenario(scenario!("Mock http server")
            .register_transaction(transaction!(hello_endpoint)))
        .set_default(GooseDefault::Host, "http://127.0.0.1:1981")?
        .set_default(GooseDefault::Users, 30)?
        .set_default(GooseDefault::StartupTime, 10)?
        .set_default(GooseDefault::RunningMetrics, 5)?
        .set_default(GooseDefault::RunTime, 120)?
        .execute()
        .await?;
    Ok(())
}

async fn hello_endpoint(user: &mut GooseUser) -> TransactionResult {
    let goose = user.get("/hello").await?;
    let validate = &Validate::builder()
        .status(200)
        .text("we will implement /world")
        .build();
    validate_and_load_static_assets(user, goose, &validate).await?;
    Ok(())
}
