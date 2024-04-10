use sqlx::{Connection, PgConnection};
use std::net::TcpListener;

use zero2prod::configuration::get_configuration;

#[tokio::test]
async fn health_check_works() {
    let address = spawn_app();

    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_200_when_valid() {
    let address = spawn_app();
    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection_string = configuration.database.connection_string();
    let connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to connect to Postgres.");

    let client = reqwest::Client::new();

    let body = "name=Joren%20vanGoethem&email=jorenvangoethem%40hotmail.com";

    let response = client
        .post(format!("{}/subscriptions", &address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_returns_400_when_invalid() {
    let address = spawn_app();
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("", "missing mail and name"),
        ("name=Joren%20vanGoethem", "missing email"),
        ("email=jorenvangoethem%40hotmail.com", "missing name"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(format!("{}/subscriptions", &address))
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

// since there is no await call here there is no need for this to be async
// without this the tests can't run so propagating the errors would be useless
// let the app panic with an error message instead by using expect or unwrap without error handling
fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random port.");
    let port = listener.local_addr().unwrap().port();

    let server = zero2prod::run(listener).expect("Failed to bind to address.");

    // tokio::spawn launches the server as a background task
    // and returns a handle to the spawned future
    // we have no use for it here so use a non-binding let _
    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}
