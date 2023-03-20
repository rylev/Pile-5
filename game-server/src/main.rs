use axum::extract::connect_info::ConnectInfo;
use axum::extract::ws::{self, WebSocket, WebSocketUpgrade};
use axum::extract::Query;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use futures::stream::{SplitSink, StreamExt};
use futures::SinkExt;
use once_cell::sync::OnceCell;
use rand::seq::SliceRandom;
use rand::thread_rng;
use tokio::sync::Mutex;
use tower_http::services::ServeDir;

use std::collections::HashMap;
use std::ops::ControlFlow;
use std::{net::SocketAddr, path::PathBuf};

#[tokio::main]
async fn main() {
    let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("dist");

    // build our application with some routes
    let app = Router::new()
        .fallback_service(ServeDir::new(assets_dir).append_index_html_on_directories(true))
        .route("/join", post(join))
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

async fn join(
    ConnectInfo(who): ConnectInfo<SocketAddr>,
    Json(Name { name }): Json<Name>,
) -> impl IntoResponse {
    println!("{who} joining...");
    let user_id = state().lock().await.join(name);
    match user_id {
        Some(user_id) => {
            println!("{who} joined with user_id: {user_id}");
            broadcast_state().await;
            Json(serde_json::json! {
                {
                    "user_id": user_id
                }
            })
        }
        None => todo!("Game already begun"),
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
    println!("{who} connected with user_id '{user_id}'.");
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
    broadcast_state().await;
    let recv = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if process_message(msg, who, &user_id).await.is_break() {
                return;
            }
        }
    });
    recv.await.unwrap();
}

fn state() -> &'static Mutex<GameState> {
    static STATE: OnceCell<Mutex<GameState>> = OnceCell::new();
    STATE.get_or_init(|| Mutex::new(GameState::new()))
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

            match handle_message(msg).await {
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
        let player_state = state.table.players.get(user_id).unwrap();
        let response = serde_json::json! {{
            "me": player_state,
            "round": state.round,
            "piles": state.table.piles,
            "players": state.table.players()
        }};
        if let Err(e) = sender
            .send(ws::Message::Text(serde_json::to_string(&response).unwrap()))
            .await
        {
            eprintln!("Error sending broadcast: {e}");
        }
    }
}

async fn handle_message(msg: Message) -> Result<(), String> {
    match msg {
        Message::QueryState => Ok(()),
    }
}

#[derive(serde::Deserialize)]
#[serde(tag = "event")]
#[serde(rename_all = "snake_case")]
enum Message {
    QueryState,
}

#[derive(serde::Serialize, Debug)]
struct TableState {
    #[serde(skip)]
    deck: Deck,
    #[serde(serialize_with = "serialize_players")]
    players: HashMap<String, PlayerState>,
    piles: [Pile; 4],
}

fn serialize_players<S>(
    players: &HashMap<String, PlayerState>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let players = players.values().map(|p| &p.name);
    serializer.collect_seq(players)
}

impl TableState {
    fn new() -> Self {
        let mut deck = Deck::new();
        Self {
            players: HashMap::new(),
            piles: [
                Pile::new(deck.deal()),
                Pile::new(deck.deal()),
                Pile::new(deck.deal()),
                Pile::new(deck.deal()),
            ],
            deck,
        }
    }

    fn players(&self) -> Vec<String> {
        self.players.values().map(|p| p.name.clone()).collect()
    }

    fn join(&mut self, name: String) -> String {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        name.hash(&mut hasher);
        let key = hasher.finish();
        let mut hand: Vec<_> = (0..10).into_iter().map(|_| self.deck.deal()).collect();
        hand.sort();
        // TODO: handle if the player was already added
        self.players.insert(
            key.to_string(),
            PlayerState {
                name,
                points: 0,
                hand,
            },
        );
        key.to_string()
    }
}

#[derive(serde::Serialize, Debug)]
struct GameState {
    table: TableState,
    round: Round,
}

impl GameState {
    pub fn new() -> Self {
        let table = TableState::new();
        Self {
            table,
            round: Round(0),
        }
    }

    /// Returns `None` when the game has already begun
    fn join(&mut self, name: String) -> Option<String> {
        if self.round == Round(0) {
            Some(self.table.join(name))
        } else {
            None
        }
    }
}

#[derive(serde::Serialize, Debug)]
struct PlayerState {
    name: String,
    points: u16,
    hand: Vec<u8>,
}

#[derive(serde::Serialize, Debug, PartialEq, Eq)]
struct Round(u8);

#[derive(serde::Serialize, Debug)]
struct Pile(Vec<u8>);

impl Pile {
    fn new(card: u8) -> Self {
        Self(vec![card])
    }
}

struct Deck {
    cards: Vec<u8>,
}

impl std::fmt::Debug for Deck {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Deck")
            .field("cards", &self.cards.len())
            .finish()
    }
}

impl Deck {
    fn new() -> Self {
        let mut cards: Vec<u8> = (1..=104u8).collect();
        cards.shuffle(&mut thread_rng());
        Self { cards }
    }

    fn deal(&mut self) -> u8 {
        self.cards
            .pop()
            .expect("Deck should never be fulling dealt")
    }
}
