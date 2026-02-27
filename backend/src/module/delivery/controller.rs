use axum::{
    http::{HeaderValue, StatusCode, header},
    response::IntoResponse,
};

pub async fn openapi_yaml() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/yaml; charset=utf-8"),
        )],
        include_str!("../../../docs/openapi.yaml"),
    )
}

pub async fn postman_collection() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json; charset=utf-8"),
        )],
        include_str!("../../../docs/postman_collection.json"),
    )
}

pub async fn bruno_collection() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json; charset=utf-8"),
        )],
        include_str!("../../../docs/bruno_collection.json"),
    )
}
