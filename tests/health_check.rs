use std::net::TcpListener;

use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;

use zero2prod::configuration::{get_configuration, DatabaseSettings};
use zero2prod::startup::run;

pub struct TestApp {
    pub address: String,
    pub connection_pool: PgPool,
}

#[tokio::test]
async fn health_check_works() {
    let test_app = spawn_app().await;

    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", &test_app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_200_when_valid() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    let body = "name=Joren%20vanGoethem&email=jorenvangoethem%40hotmail.com";

    let response = client
        .post(format!("{}/subscriptions", &test_app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&test_app.connection_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "jorenvangoethem@hotmail.com");
    assert_eq!(saved.name, "Joren vanGoethem");
}

#[tokio::test]
async fn subscribe_returns_400_when_invalid() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("", "missing mail and name"),
        ("name=Joren%20vanGoethem", "missing email"),
        ("email=jorenvangoethem%40hotmail.com", "missing name"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(format!("{}/subscriptions", &test_app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect(&config.connection_string_without_db())
        .await
        .expect("Failed to connect to Postgres.");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    let connection_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to connect to Postgres.");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database.");

    connection_pool
}

// since there is no await call here there is no need for this to be async
// without this the tests can't run so propagating the errors would be useless
// let the app panic with an error message instead by using expect or unwrap without error handling
async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random port.");

    let port = listener.local_addr().unwrap().port();

    let address = format!("http://127.0.0.1:{}", port);

    let mut configuration = get_configuration().expect("Failed to read configuration.");
    configuration.database.database_name = Uuid::new_v4().to_string();

    let connection_pool = configure_database(&configuration.database).await;

    let server = run(listener, connection_pool.clone()).expect("Failed to bind to address.");

    // tokio::spawn launches the server as a background task
    // and returns a handle to the spawned future
    // we have no use for it here so use a non-binding let _
    let _ = tokio::spawn(server);

    TestApp {
        address,
        connection_pool,
    }
}
