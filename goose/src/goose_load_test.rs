
use goose::prelude::*;
use std::time::Duration;

pub async fn run_load_test() -> Result<(), GooseError> {
    GooseAttack::initialize()?
        // In this example, we only create a single scenario, named "WebsiteUser".
        .register_scenario(
            scenario!("Mock http server")
                .set_host("http://127.0.0.1:1981")
                // After each transactions runs, sleep randomly from 500ms to 1 seconds.
                .set_wait_time(Duration::from_millis(500), Duration::from_secs(1))?
                .register_transaction(transaction!(hello_endpoint).set_on_start())
        )
        .execute()
        .await?;
    Ok(())
}

/// A very simple transaction that simply loads the front page.
async fn hello_endpoint(user: &mut GooseUser) -> TransactionResult {
    let _goose = user.get("/hello").await?;
    println!("Response {:?}",_goose.response);
    Ok(())
}
