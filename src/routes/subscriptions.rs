use actix_web::{web, HttpResponse};
use sqlx::types::chrono::Utc;
use sqlx::types::Uuid;
use sqlx::PgPool;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(
    form: web::Form<FormData>,
    connection_pool: web::Data<PgPool>,
) -> HttpResponse {
    let request_id = format!("request id {} -", Uuid::new_v4());
    let request_span = tracing::info_span!("Handling new subscription request.", %request_id, subscriber_email = %form.email, subscriber_name = %form.name);
    let _request_span_guard = request_span.enter();

    tracing::info!(
        "{} Adding '{}' '{}' as new subscriber.",
        request_id,
        form.email,
        form.name
    );
    match sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(connection_pool.get_ref())
    .await
    {
        Ok(_) => {
            tracing::info!("{} new subscriber has been saved.", request_id);
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            tracing::error!("{} Failed to execute query: {:?}", request_id, e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
