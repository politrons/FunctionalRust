use testcontainers::{clients, core::WaitFor, Image, images::postgres::Postgres};
use tokio_postgres::{Client, Error, Row};

#[derive(Debug)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub email: String,
}

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

#[tokio::main]
async fn main() {
    let docker = clients::Cli::default();

    // Define a PostgreSQL container image
    let postgres_image = Postgres::default();

    let pg_container = docker.run(postgres_image);
    println!("Docker info. Container_id:{:?} image: {:?}", pg_container.id(), pg_container.image().name());

    pg_container.start();

    WaitFor::seconds(60);

    // Get the PostgreSQL port
    let pg_port = pg_container.get_host_port_ipv4(5432);

    // Define the connection to the Postgress client
    let (client, connection) = tokio_postgres::Config::new()
        .user("postgres")
        .password("postgres")
        .host("localhost")
        .port(pg_port)
        .dbname("postgres")
        .connect(tokio_postgres::NoTls)
        .await
        .unwrap();

    // Spawn connection
    tokio::spawn(async move {
        if let Err(error) = connection.await {
            eprintln!("Connection error: {}", error);
        }
    });

    create_user_table(&client).await;
    insert_user_table(&client).await;

    let result = match client
        .query("SELECT id, username, password, email FROM app_user", &[])
        .await {
        Ok(rows) => rows,
        Err(e) => {
            println!("No record found. Caused by {}", e);
            return;
        }
    };

    let users: Vec<User> = result.into_iter().map(|row| User::from(row)).collect();

    let user = users.first().unwrap();

    println!("User {:?}", user);

    assert_eq!(1, user.id);
    assert_eq!("user1", user.username);
    assert_eq!("mypass", user.password);
    assert_eq!("user@test.com", user.email);
}

async fn insert_user_table(client: &Client) {
    match client
        .execute(
            "INSERT INTO app_user (username, password, email) VALUES ($1, $2, $3)",
            &[&"user1", &"mypass", &"user@test.com"],
        )
        .await {
        Ok(code) => println!("Insert in User table with code {:?} successfully", code),
        Err(e) => println!("Error Adding record in User table. Caused by {}", e),
    }
}

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

