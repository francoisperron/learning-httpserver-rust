use std::sync::Arc;

use axum::{extract::{Path, State}, http, Json, response::{IntoResponse, Response}, Router, routing::{delete, get, post, put}};
use http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::users::id::Id;
use crate::users::user::User;
use crate::users::username::Username;
use crate::users::users_repo::{UsersRepo, UsersRepoInMemory};

pub fn add_users_endpoints(router: Router<UsersState>) -> Router {
    router
        .route("/users", post(create_user))
        .route("/users", get(get_users))
        .route("/users/:id", get(get_user))
        .route("/users/:id", put(update_user))
        .route("/users/:id", delete(delete_user))
        .with_state(UsersState::in_memory())
}

pub async fn create_user(State(state): State<UsersState>, Json(request): Json<CreateUserApiRequest>) -> Response {
    let Ok(user) = User::new(&request.username) else {
        return StatusCode::BAD_REQUEST.into_response();
    };

    state.users_repo.save_user(&user).await;

    (StatusCode::CREATED, Json(CreateUserApiResponse { id: user.id.into() })).into_response()
}

pub async fn update_user(State(state): State<UsersState>, Path(id): Path<u64>, Json(request): Json<UpdateUserApiRequest>) -> StatusCode {
    let Some(mut user) = state.users_repo.get_user(Id::from(id)).await else {
        return StatusCode::NOT_FOUND
    };

    let Ok(username) = Username::new(&request.username) else {
        return StatusCode::BAD_REQUEST;
    };

    user.username = username;
    state.users_repo.save_user(&user).await;

    StatusCode::OK
}

pub async fn get_users(State(state): State<UsersState>) -> (StatusCode, Json<GetUsersApiResponse>) {
    let users = state.users_repo
        .get_users()
        .await
        .into_iter()
        .map(|u| GetUserApiResponse { id: u.id.into(), username: u.username.into() })
        .collect();

    (StatusCode::OK, Json(GetUsersApiResponse { users }))
}

pub async fn get_user(State(state): State<UsersState>, Path(id): Path<u64>) -> Response {
    let Some(user) = state.users_repo.get_user(Id::from(id)).await else {
        return StatusCode::NOT_FOUND.into_response()
    };

    Json(GetUserApiResponse { id: user.id.into(), username: user.username.into() }).into_response()
}

pub async fn delete_user(State(state): State<UsersState>, Path(id): Path<u64>) -> StatusCode {
    let deleted = state.users_repo.delete_user(Id::from(id)).await;

    if deleted { StatusCode::OK } else { StatusCode::NOT_FOUND }
}

#[derive(Debug, Clone)]
pub struct UsersState {
    pub users_repo: Arc<UsersRepoInMemory>,
}


impl UsersState {
    pub fn in_memory() -> UsersState {
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GetUsersApiResponse {
    pub users: Vec<GetUserApiResponse>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GetUserApiResponse {
    pub id: u64,
    pub username: String,
}

#[cfg(test)]
mod tests {
    use axum::http;
    use http::StatusCode;

    use crate::users::users_endpoints::{CreateUserApiRequest, GetUserApiResponse, UpdateUserApiRequest};
    use crate::users::users_endpoints::test_support::Api;

    #[tokio::test]
    async fn creates_user() {
        let api = Api::new().await;

        let post_response = api.post_user(&CreateUserApiRequest { username: "mario".to_string() }).await;
        let user_id = post_response.body.unwrap().id;
        assert_eq!(post_response.status_code, StatusCode::CREATED);

        let get_response = api.get_user(user_id).await;
        assert_eq!(get_response.body, Some(GetUserApiResponse { id: user_id, username: "mario".to_string() }));
    }

    #[tokio::test]
    async fn updates_user() {
        let api = Api::new().await;

        let post_response = api.post_user(&CreateUserApiRequest { username: "mario".to_string() }).await;
        let user_id = post_response.body.unwrap().id;

        let put_response = api.put_user(user_id, &UpdateUserApiRequest { username: "luigi".to_string() }).await;
        assert_eq!(put_response.status_code, StatusCode::OK);

        let get_response = api.get_user(user_id).await;
        assert_eq!(get_response.body, Some(GetUserApiResponse { id: user_id, username: "luigi".to_string() }));
    }

    #[tokio::test]
    async fn deletes_user() {
        let api = Api::new().await;

        let post_response = api.post_user(&CreateUserApiRequest { username: "mario".to_string() }).await;
        let user_id = post_response.body.unwrap().id;
        
        let delete_response = api.delete_user(user_id).await;
        assert_eq!(delete_response.status_code, StatusCode::OK);

        let get_response = api.get_user(user_id).await;
        assert_eq!(get_response.status_code, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn gets_all_users() {
        let api = Api::new().await;

        let post_1_response = api.post_user(&CreateUserApiRequest { username: "mario".to_string() }).await;
        let post_1_id = post_1_response.body.unwrap().id;
        let post_2_response = api.post_user(&CreateUserApiRequest { username: "luigi".to_string() }).await;
        let post_2_id = post_2_response.body.unwrap().id;

        let get_response = api.get_users().await;
        assert_eq!(get_response.status_code, StatusCode::OK);
        
        let users = get_response.body.unwrap().users;
        assert_eq!(users.clone().into_iter().find(|u| u.id == post_1_id).unwrap().username, "mario".to_string());
        assert_eq!(users.into_iter().find(|u| u.id == post_2_id).unwrap().username, "luigi".to_string());
    }
}

#[cfg(test)]
mod test_support {
    use std::net::SocketAddr;

    use axum::body::Body;
    use http::{header, Method, Request, StatusCode};
    use http_body_util::BodyExt;
    use hyper_util::client::legacy::Client;
    use hyper_util::client::legacy::connect::HttpConnector;
    use hyper_util::rt::TokioExecutor;
    use tokio::net::TcpListener;

    use crate::app;
    use crate::users::users_endpoints::{CreateUserApiRequest, CreateUserApiResponse, GetUserApiResponse, GetUsersApiResponse, UpdateUserApiRequest};

    pub struct Api {
        server: SocketAddr,
        client: Client<HttpConnector, Body>,
    }

    #[cfg(test)]
    pub struct ApiResponse<R> {
        pub status_code: StatusCode,
        pub body: Option<R>,
    }

    #[cfg(test)]
    impl Api {
        pub async fn new() -> Api {
            let listener = TcpListener::bind("0.0.0.0:0").await.unwrap();
            let server = listener.local_addr().unwrap();
            tokio::spawn(async move { axum::serve(listener, app()).await.unwrap(); });
            let client = Client::builder(TokioExecutor::new()).build_http();
            Api { server, client }
        }

        pub async fn post_user(&self, request_body: &CreateUserApiRequest) -> ApiResponse<CreateUserApiResponse> {
            let request = Request::builder()
                .method(Method::POST)
                .uri(format!("http://{}/users", self.server))
                .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Body::from(serde_json::to_string(request_body).unwrap())).unwrap();

            let response = self.client.request(request).await.unwrap();

            let status_code = response.status();
            let body = response.into_body().collect().await.unwrap().to_bytes();
            ApiResponse { status_code, body: serde_json::from_slice(&body).unwrap() }
        }

        pub async fn put_user(&self, id: u64, request: &UpdateUserApiRequest) -> ApiResponse<()> {
            let r = Request::builder()
                .method(Method::PUT)
                .uri(format!("http://{}/users/{}", self.server, id))
                .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Body::from(serde_json::to_string(request).unwrap())).unwrap();

            let response = self.client.request(r).await.unwrap();

            let status_code = response.status();
            ApiResponse { status_code, body: None }
        }

        pub async fn delete_user(&self, id: u64) -> ApiResponse<()> {
            let r = Request::builder()
                .method(Method::DELETE)
                .uri(format!("http://{}/users/{}", self.server, id))
                .body(Body::empty()).unwrap();

            let response = self.client.request(r).await.unwrap();

            let status_code = response.status();
            ApiResponse { status_code, body: None }
        }

        pub async fn get_user(&self, id: u64) -> ApiResponse<GetUserApiResponse> {
            let request = Request::builder()
                .method(Method::GET)
                .uri(format!("http://{}/users/{}", self.server, id))
                .body(Body::empty()).unwrap();

            let response = self.client.request(request).await.unwrap();

            let status_code = response.status();
            let body = response.into_body().collect().await.unwrap().to_bytes();

            if status_code.is_success() {
                ApiResponse { status_code, body: Some(serde_json::from_slice(&body).unwrap()) }
            } else {
                ApiResponse { status_code, body: None }
            }
        }

        pub async fn get_users(&self) -> ApiResponse<GetUsersApiResponse> {
            let request = Request::builder()
                .method(Method::GET)
                .uri(format!("http://{}/users", self.server))
                .body(Body::empty()).unwrap();

            let response = self.client.request(request).await.unwrap();

            let status_code = response.status();
            let body = response.into_body().collect().await.unwrap().to_bytes();
            ApiResponse { status_code, body: serde_json::from_slice(&body).unwrap() }
        }
    }
}
