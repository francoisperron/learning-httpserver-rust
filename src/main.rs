use axum::Router;
use tokio::net;

use crate::users_endpoints::add_users_endpoints;

mod users_repo;
mod user;
mod users_endpoints;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let listener = net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app()).await.unwrap();
}

fn app() -> Router {
    let router = Router::new();
    
    add_users_endpoints(router)
}