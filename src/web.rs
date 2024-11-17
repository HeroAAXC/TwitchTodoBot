use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::Html,
    routing::{get, post},
    Json, Router,
};
use tokio::{
    sync::{mpsc::Sender, Mutex},
    task::JoinHandle,
};
use tower_http::cors::CorsLayer;
use twitch_irc::{login::LoginCredentials, transport::Transport};

use crate::{channel_joiner::ChannelJoiner, config::ModSet};

const ROOT_PAGE: &str = include_str!("./index.html"); // TODO

pub fn spawn_axum_worker<T: Transport, C: LoginCredentials>(
    joiner: Arc<Mutex<ChannelJoiner<T, C>>>,
    mod_set: Arc<Mutex<ModSet>>,
    stop_sender: Arc<Mutex<Sender<()>>>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let app = Router::new()
            .route("/", get(root))
            .route("/get_channels", get(get_channels))
            .with_state(joiner.clone())
            .route("/post_channels", post(post_channels))
            .with_state(joiner)
            .route("/get_mods", get(get_mods))
            .with_state(mod_set.clone())
            .route("/post_mods", post(post_mods))
            .with_state(mod_set)
            .route("/send_stop", post(send_stop))
            .with_state(stop_sender)
            .layer(CorsLayer::permissive());
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        axum::serve(listener, app).await.unwrap()
    })
}

pub async fn root() -> Html<&'static str> {
    Html(ROOT_PAGE)
}

pub async fn post_channels<T: Transport, C: LoginCredentials>(
    State(joiner): State<Arc<Mutex<ChannelJoiner<T, C>>>>,
    Json(payload): Json<Vec<String>>,
) -> StatusCode {
    match joiner.lock().await.update_channels(payload).await {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            log::error!("{e}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub async fn get_mods(State(mods): State<Arc<Mutex<ModSet>>>) -> Json<String> {
    Json(
        serde_json::to_string(
            &mods
                .lock()
                .await
                .set
                .clone()
                .into_iter()
                .collect::<Vec<String>>(),
        )
        .unwrap(),
    )
}

pub async fn post_mods(
    State(mods): State<Arc<Mutex<ModSet>>>,
    Json(payload): Json<Vec<String>>,
) -> StatusCode {
    mods.lock().await.update(payload);
    mods.lock().await.save().await.unwrap();

    StatusCode::OK
}

pub async fn get_channels<T: Transport, C: LoginCredentials>(
    State(joiner): State<Arc<Mutex<ChannelJoiner<T, C>>>>,
) -> Json<String> {
    Json(serde_json::to_string(&joiner.lock().await.channels()).unwrap())
}

pub async fn send_stop(
    State(stop_sender): State<Arc<Mutex<Sender<()>>>>,
    payload: String,
) -> StatusCode {
    if payload == "njitrbjnirebtnui4tb4u59ßb" {
        let _res = stop_sender.lock().await.send(()).await;
        return StatusCode::OK;
    }
    StatusCode::BAD_REQUEST
}
