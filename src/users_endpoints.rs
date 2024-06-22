use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use axum::extract::{Path, State};
use axum::{Json, Router};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post, put};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use crate::user::{Id, User};
use crate::users_repo::{UsersRepo, UsersRepoInMemory};

pub fn add_users_endpoints(router: Router<UsersState>) -> Router {
    router
        .route("/users", post(create_user))
        .route("/users", get(get_users))
        .route("/users/:id", get(get_user))
        .route("/users/:id", put(update_user))
        .route("/users/:id", delete(delete_user))
        .with_state(UsersState::new())
}

pub async fn create_user(State(state): State<UsersState>, Json(request): Json<CreateUserApiRequest>) -> (StatusCode, Json<CreateUserApiResponse>) {
    let id = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_micros() as Id;
    let user = User { id, username: request.username };
    state.users_repo.save_user(&user);

    let response = CreateUserApiResponse { id: user.id };
    (StatusCode::CREATED, Json(response))
}

pub async fn update_user(State(state): State<UsersState>, Path(id): Path<u64>, Json(request): Json<UpdateUserApiRequest>) -> StatusCode {
    if state.users_repo.get_user(id).is_none() {
        StatusCode::NOT_FOUND
    } else {
        let mut user = state.users_repo.get_user(id).unwrap().clone();

        user.username = request.username;
        state.users_repo.save_user(&user);

        StatusCode::OK
    }
}

pub async fn get_users(State(state): State<UsersState>) -> (StatusCode, Json<GetUsersApiResponse>) {
    let users = state.users_repo
        .get_users()
        .into_iter()
        .map(|u| GetUserApiResponse { id: u.id, username: u.username })
        .collect();

    let response = GetUsersApiResponse { users };
    (StatusCode::OK, Json(response))
}

pub async fn get_user(State(state): State<UsersState>, Path(id): Path<u64>) -> Response {
    match state.users_repo.get_user(id) {
        Some(user) => {
            let response = GetUserApiResponse { id: user.id, username: user.username };
            Json(response).into_response()
        }
        None => StatusCode::NOT_FOUND.into_response()
    }
}

pub async fn delete_user(State(state): State<UsersState>, Path(id): Path<u64>) -> StatusCode {
    if state.users_repo.delete_user(id).is_some() {
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

#[derive(Clone)]
pub struct UsersState {
    pub users_repo: Arc<dyn UsersRepo>,
}

impl UsersState {
    pub fn new() -> UsersState {
        let users_repo = UsersRepoInMemory::default();
        UsersState { users_repo: Arc::new(users_repo) }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct CreateUserApiRequest {
    pub username: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct CreateUserApiResponse {
    pub id: u64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct UpdateUserApiRequest {
    pub username: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct GetUsersApiResponse {
    pub users: Vec<GetUserApiResponse>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct GetUserApiResponse {
    pub id: u64,
    pub username: String,
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http;
    use axum::http::Request;
    use http::StatusCode;
    use http_body_util::BodyExt;
    use tokio::net::TcpListener;
    use tower::{Service, ServiceExt};
    use crate::app;

    use crate::users_endpoints::{CreateUserApiRequest, CreateUserApiResponse, GetUserApiResponse};

    #[tokio::test]
    async fn add_and_get_user() {
        let mut app = app().into_service();

        let create_user_request_body = serde_json::to_string(&CreateUserApiRequest { username: "mario".to_string() }).unwrap();
        let create_user_request = Request::builder()
            .method(http::Method::POST)
            .uri("/users")
            .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .body(Body::from(create_user_request_body)).unwrap();

        let create_user_response = ServiceExt::<Request<Body>>::ready(&mut app)
            .await.unwrap().call(create_user_request).await.unwrap();

        assert_eq!(create_user_response.status(), StatusCode::CREATED);

        let create_user_response_body = create_user_response.into_body().collect().await.unwrap().to_bytes();
        let create_user_response_json: CreateUserApiResponse = serde_json::from_slice(&create_user_response_body).unwrap();

        let get_user_request = Request::builder()
            .method(http::Method::GET)
            .uri(format!("/users/{}", create_user_response_json.id))
            .body(Body::empty()).unwrap();
        let get_user_response = ServiceExt::<Request<Body>>::ready(&mut app)
            .await.unwrap().call(get_user_request).await.unwrap();

        assert_eq!(get_user_response.status(), StatusCode::OK);
        let get_user_response_body = get_user_response.into_body().collect().await.unwrap().to_bytes();
        let get_user_response_json: GetUserApiResponse = serde_json::from_slice(&get_user_response_body).unwrap();
        assert_eq!(get_user_response_json, GetUserApiResponse { id: create_user_response_json.id, username: "mario".to_string() });
    }

    #[tokio::test]
    async fn add_and_get_user_real_server() {
        let listener = TcpListener::bind("0.0.0.0:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(listener, app()).await.unwrap(); });

        let client = hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new()).build_http();

        let create_user_request_body = serde_json::to_string(&CreateUserApiRequest { username: "mario".to_string() }).unwrap();
        let create_user_request = Request::builder()
            .method(http::Method::POST)
            .uri(format!("http://{addr}/users"))
            .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .body(Body::from(create_user_request_body)).unwrap();

        let create_user_response = client.request(create_user_request).await.unwrap();
        assert_eq!(create_user_response.status(), StatusCode::CREATED);

        let create_user_response_body = create_user_response.into_body().collect().await.unwrap().to_bytes();
        let create_user_response_json: CreateUserApiResponse = serde_json::from_slice(&create_user_response_body).unwrap();

        let get_user_request = Request::builder()
            .method(http::Method::GET)
            .uri(format!("http://{addr}/users/{}", create_user_response_json.id))
            .body(Body::empty()).unwrap();

        let get_user_response = client.request(get_user_request).await.unwrap();
        assert_eq!(get_user_response.status(), StatusCode::OK);

        let get_user_response_body = get_user_response.into_body().collect().await.unwrap().to_bytes();
        let get_user_response_json: GetUserApiResponse = serde_json::from_slice(&get_user_response_body).unwrap();
        assert_eq!(get_user_response_json, GetUserApiResponse { id: create_user_response_json.id, username: "mario".to_string() });
    }
}