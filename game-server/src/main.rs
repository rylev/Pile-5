use axum::extract::connect_info::ConnectInfo;
use axum::extract::ws::{self, WebSocket, WebSocketUpgrade};
use axum::extract::Query;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use futures::stream::{SplitSink, StreamExt};
use futures::SinkExt;
use once_cell::sync::OnceCell;
use tokio::sync::Mutex;
use tower_http::services::ServeDir;

use std::collections::HashMap;
use std::ops::ControlFlow;
use std::{net::SocketAddr, path::PathBuf};

mod state;

use state::{PileIndex, State};

#[tokio::main]
async fn main() {
    let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("dist");

    // build our application with some routes
    let app = Router::new()
        .fallback_service(ServeDir::new(assets_dir).append_index_html_on_directories(true))
        .route("/join", post(join))
        .route("/version", get(version))
        .route("/ws", get(ws_handler));

    // run it with hyper
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}

#[derive(serde::Deserialize)]
struct Name {
    name: String,
}

async fn version() -> impl IntoResponse {
    format!("{}", get_version())
}

async fn join(
    ConnectInfo(who): ConnectInfo<SocketAddr>,
    Json(Name { name }): Json<Name>,
) -> impl IntoResponse {
    println!("{who} attempting to join...");
    let user_id = state().lock().await.join(name);
    match user_id {
        Ok(user_id) => {
            println!("{who} joined lobby with user_id: {user_id}");
            broadcast_state().await;
            Json(serde_json::json! {
                {
                    "user_id": user_id,
                    "version": get_version()
                }
            })
        }
        Err(_) => todo!("Game already begun"),
    }
}

#[derive(serde::Deserialize)]
struct UserId {
    user_id: String,
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(UserId { user_id }): Query<UserId>,
    ConnectInfo(who): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    println!("{who} connected to WebSocket with user_id '{user_id}'.");
    // finalize the upgrade process by returning upgrade callback.
    // we can customize the callback by sending additional info such as address.
    ws.on_upgrade(move |socket| handle_socket(socket, who, user_id))
}

/// Actual websocket statemachine (one will be spawned per connection)
async fn handle_socket(socket: WebSocket, who: SocketAddr, user_id: String) {
    let (sender, mut receiver) = socket.split();
    {
        let mut senders = senders().lock().await;
        senders.authenticated.insert(user_id.clone(), sender);
    }
    send_state(&user_id).await;
    let recv = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if process_message(msg, who, &user_id).await.is_break() {
                return;
            }
        }
    });
    recv.await.unwrap();
}

fn state() -> &'static Mutex<State> {
    static STATE: OnceCell<Mutex<State>> = OnceCell::new();
    STATE.get_or_init(|| Mutex::new(State::new()))
}

/// helper to print contents of messages to stdout. Has special treatment for Close.
async fn process_message(msg: ws::Message, who: SocketAddr, user_id: &str) -> ControlFlow<(), ()> {
    match msg {
        ws::Message::Text(t) => {
            println!(">>> {} sent str: {:?}", who, t);
            let msg: Message = match serde_json::from_str(&t) {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("Error parsing '{t}': {e}");
                    return ControlFlow::Continue(());
                }
            };

            match handle_message(msg, user_id).await {
                Ok(()) => broadcast_state().await,
                Err(e) => send_message(user_id, e).await,
            }
        }
        ws::Message::Close(Some(cf)) => {
            println!(
                ">>> {} sent close with code {} and reason `{}`",
                who, cf.code, cf.reason
            );
            remove_sender(user_id).await;
            return ControlFlow::Break(());
        }
        ws::Message::Close(None) => {
            println!(">>> {} sent close message without CloseFrame", who);
            remove_sender(user_id).await;
            return ControlFlow::Break(());
        }

        m => {
            println!(">>> {} sent unrecognized message {:?}", who, m);
        }
    }
    ControlFlow::Continue(())
}

fn senders() -> &'static Mutex<Senders> {
    static SENDERS: OnceCell<Mutex<Senders>> = OnceCell::new();
    SENDERS.get_or_init(|| Mutex::new(Senders::new()))
}

struct Senders {
    authenticated: HashMap<String, SplitSink<WebSocket, ws::Message>>,
}

impl Senders {
    fn new() -> Self {
        Self {
            authenticated: HashMap::new(),
        }
    }
}

async fn remove_sender(user_id: &str) {
    senders().lock().await.authenticated.remove(user_id);
}

async fn send_message(user_id: &str, msg: String) {
    senders()
        .lock()
        .await
        .authenticated
        .get_mut(user_id)
        .unwrap()
        .send(ws::Message::Text(msg))
        .await
        .unwrap();
}

async fn broadcast_state() {
    let state = state().lock().await;
    for (user_id, sender) in senders().lock().await.authenticated.iter_mut() {
        _send_state(&state, user_id, sender).await;
    }
}

async fn send_state(user_id: &str) {
    let state = state().lock().await;
    if let Some(sender) = senders().lock().await.authenticated.get_mut(user_id) {
        _send_state(&state, user_id, sender).await;
    }
}

async fn _send_state(state: &State, user_id: &str, sender: &mut SplitSink<WebSocket, ws::Message>) {
    if state.get_player(user_id).is_none() {
        panic!("player with id '{user_id}' does not exist");
    }
    let response = state.serialize_for_user(user_id);
    if let Err(e) = sender
        .send(ws::Message::Text(serde_json::to_string(&response).unwrap()))
        .await
    {
        eprintln!("Error sending broadcast: {e}");
    }
}

async fn handle_message(msg: Message, user_id: &str) -> Result<(), String> {
    let mut state = state().lock().await;
    if state.get_player(user_id).is_none() {
        return Err(format!("Player with id '{user_id}' does not exist"));
    }
    match msg {
        Message::Debug => {
            println!("{state:#?}")
        }
        Message::StartGame => {
            state.start_game().unwrap();
        }
        Message::PlayCard { card } => {
            state.play_card(user_id, card).unwrap();
        }
        Message::SelectPile { pile_index } => {
            let pile_index = PileIndex::try_from(pile_index).expect("TODO: handle error");
            state.select_pile(user_id, pile_index).unwrap();
        }
        Message::RestartGame => state.restart(),
    }
    Ok(())
}

#[derive(serde::Deserialize)]
#[serde(tag = "event")]
#[serde(rename_all = "snake_case")]
enum Message {
    Debug,
    StartGame,
    PlayCard { card: u8 },
    SelectPile { pile_index: usize },
    RestartGame,
}

fn get_version() -> u64 {
    static STARTUP_TIME: OnceCell<u64> = OnceCell::new();
    *STARTUP_TIME.get_or_init(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    })
}
