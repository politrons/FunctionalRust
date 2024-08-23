use testcontainers::{clients, core::WaitFor, Image, images::postgres::Postgres};
use tokio_postgres::{Client, Row};
use uuid::Uuid;

#[tokio::main]
async fn main() {
    // Create a Docker client to manage containers
    let docker = clients::Cli::default();

    // Define a PostgreSQL container image using Testcontainers
    let postgres_image = Postgres::default();

    // Start the PostgreSQL container
    let pg_container = docker.run(postgres_image);
    println!("Docker info. Container_id:{:?} image: {:?}", pg_container.id(), pg_container.image().name());

    // Start the container explicitly (even though run should already start it)
    pg_container.start();

    // Wait for a moment to ensure that PostgreSQL is fully up and running
    WaitFor::seconds(60);

    // Get the port on which PostgreSQL is listening
    let pg_port = pg_container.get_host_port_ipv4(5432);

    // Create a PostgreSQL client that we'll use to interact with the database
    let client = create_postgres_client(pg_port).await;

    // Create the `app_user` table in the PostgreSQL database
    create_user_table(&client).await;

    // Insert users into the `app_user` table
    for _ in 0..5{
        insert_user_table(&client).await;
    }
    
    // Retrieve the inserted users from the database
    let result = get_query_result(client).await;

    // Transform the raw database rows into a vector of `User` structs
    let users: Vec<User> = transform_rows_in_users(result);

    // Print out the users we've retrieved from the database
    for user in users {
        println!("User {:?}", user);
    }
}

// Helper function to convert database rows into a vector of `User` structs
fn transform_rows_in_users(result: Vec<Row>) -> Vec<User> {
    result.into_iter()
        .map(|row| User::from(row))
        .collect()
}

// Asynchronously query the database for all users in the `app_user` table
async fn get_query_result(client: Client) -> Vec<Row> {
    client
        .query("SELECT id, username, password, email FROM app_user", &[])
        .await.unwrap_or_else(|e| {
        println!("No record found. Caused by {}", e);
        Vec::new()
    })
}

// Create a PostgreSQL client connected to the running Docker container
async fn create_postgres_client(pg_port: u16) -> Client {
    let (client, connection) = tokio_postgres::Config::new()
        .user("postgres")
        .password("postgres")
        .host("localhost")
        .port(pg_port)
        .dbname("postgres")
        .connect(tokio_postgres::NoTls)
        .await
        .unwrap();

    // Spawn a task to manage the connection; handle any connection errors that might arise
    tokio::spawn(async move {
        if let Err(error) = connection.await {
            eprintln!("Connection error: {}", error);
        }
    });
    client
}

// Insert a new user into the `app_user` table with a generated UUID for the username and email
async fn insert_user_table(client: &Client) {
    // Generate unique UUIDs for the username and email
    let user_uuid = Uuid::new_v4();
    let email_uuid = Uuid::new_v4();

    // Create the username and email strings using the UUIDs
    let username = format!("user-{}", user_uuid);
    let email = format!("user-{}@test.com", email_uuid);

    // Insert the new user into the `app_user` table
    match client
        .execute(
            "INSERT INTO app_user (username, password, email) VALUES ($1, $2, $3)",
            &[&username, &"mypass", &email],
        )
        .await {
        Ok(code) => println!("Insert in User table with code {:?} successfully", code),
        Err(e) => println!("Error Adding record in User table. Caused by {}", e),
    }
}

// Create the `app_user` table if it doesn't already exist
async fn create_user_table(client: &Client) {
    match client
        .batch_execute(
            "
        CREATE TABLE IF NOT EXISTS app_user (
            id              SERIAL PRIMARY KEY,
            username        VARCHAR UNIQUE NOT NULL,
            password        VARCHAR NOT NULL,
            email           VARCHAR UNIQUE NOT NULL
            )
    ",
        )
        .await {
        Ok(_) => println!("User Table created successfully"),
        Err(e) => println!("Error creating Table. Caused by {}", e),
    }
}

// Define a `User` struct that represents a row in the `app_user` table
#[derive(Debug)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub email: String,
}

// Implement a conversion from `Row` to `User`
impl From<Row> for User {
    fn from(row: Row) -> Self {
        Self {
            id: row.get("id"),
            username: row.get("username"),
            password: row.get("password"),
            email: row.get("email"),
        }
    }
}
