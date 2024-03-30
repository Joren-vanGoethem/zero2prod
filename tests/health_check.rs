use std::net::TcpListener;

#[tokio::test]
async fn health_check_works() {
    let address = spawn_app();

    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
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
