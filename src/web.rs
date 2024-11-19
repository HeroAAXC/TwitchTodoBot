use std::{collections::HashMap, sync::Arc, time::Duration};

use axum::{
    extract::State,
    http::StatusCode,
    response::{sse::Event, Html, Sse},
    routing::{get, post},
    Json, Router,
};
use futures::{stream, Stream};
use serde::{Deserialize, Serialize};
use tokio::{
    sync::{
        mpsc::{self, Sender},
        Mutex,
    },
    task::JoinHandle,
};
use tokio_stream::StreamExt as _;
use tower_http::cors::CorsLayer;
use twitch_irc::{login::LoginCredentials, transport::Transport};

use crate::{
    bot::{hash_message, Data},
    channel_joiner::ChannelJoiner,
    communication::TodoUpdate,
    config::ModSet,
};

const ROOT_PAGE: &str = if cfg!(feature = "de") {
    include_str!("./index_de.html")
} else {
    include_str!("./index_en.html")
};

const TODOS_PAGE: &str = include_str!("./todos.html");

pub fn spawn_axum_worker<T: Transport, C: LoginCredentials>(
    joiner: Arc<Mutex<ChannelJoiner<T, C>>>,
    mod_set: Arc<Mutex<ModSet>>,
    stop_sender: Arc<Mutex<Sender<()>>>,
    todo_updates: Arc<Mutex<Vec<Sender<TodoUpdate>>>>,
    data: Data,
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
            .route("/todos", get(todos_index))
            .route("/get_todos", get(get_todos))
            .with_state(data)
            .route("/todos_sse", get(sse_handler))
            .with_state(todo_updates)
            .layer(CorsLayer::permissive());
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        axum::serve(listener, app).await.unwrap()
    })
}

pub async fn root() -> Html<&'static str> {
    Html(ROOT_PAGE)
}

pub async fn todos_index() -> Html<&'static str> {
    Html(TODOS_PAGE)
}

pub async fn sse_handler(
    State(todo_updates): State<Arc<Mutex<Vec<Sender<TodoUpdate>>>>>,
) -> Sse<impl Stream<Item = Result<Event, std::convert::Infallible>>> {
    let (send, mut recv) = mpsc::channel(1);

    todo_updates.lock().await.push(send);

    drop(todo_updates);

    let stream = stream::repeat_with(move || {
        let mut todo_data = TodoStatusMessage::default();

        while let Ok(r) = recv.try_recv() {
            match r {
                TodoUpdate::AddTodo {
                    user,
                    todo_message,
                    uuid,
                } => {
                    todo_data.new_todos.push((uuid, user, todo_message));
                }
                TodoUpdate::CheckTodo(uuid) => {
                    todo_data.checks.push(uuid);
                }
            }
        }
        let event_message = match todo_data.is_empty() {
            true => SSEUpdate::KeepAlive,
            false => SSEUpdate::StatusUpdate(todo_data),
        };
        Event::default().data(serde_json::to_string(&event_message).unwrap())
    })
    .map(Ok)
    .throttle(Duration::from_secs(1));

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(20))
            .text(SSEUpdate::KeepAlive.to_string()),
    )
}

pub async fn get_todos(State(data): State<Data>) -> Json<Vec<(String, Vec<(String, u64)>)>> {
    let data = data.lock().await.clone();
    Json(
        data.iter()
            .map(|(name, todos)| {
                (
                    name.clone(),
                    todos
                        .clone()
                        .into_iter()
                        .map(|v| (v.clone(), hash_message(&name, &v)))
                        .collect(),
                )
            })
            .collect::<Vec<(String, Vec<(String, u64)>)>>(),
    )
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

#[derive(Serialize, Deserialize)]
pub struct TodoStatusMessage {
    new_todos: Vec<(u64, String, String)>,
    checks: Vec<u64>,
}

impl Default for TodoStatusMessage {
    fn default() -> Self {
        Self {
            new_todos: vec![],
            checks: vec![],
        }
    }
}

impl TodoStatusMessage {
    pub fn is_empty(&self) -> bool {
        self.new_todos.is_empty() && self.checks.is_empty()
    }
}

#[derive(Serialize, Deserialize)]
pub enum SSEUpdate {
    StatusUpdate(TodoStatusMessage),
    KeepAlive,
    InitMessage(HashMap<String, String>),
}

impl ToString for SSEUpdate {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
