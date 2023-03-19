use axum::extract::connect_info::ConnectInfo;
use axum::{
    extract::{
        ws::{self, WebSocket, WebSocketUpgrade},
        TypedHeader,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::stream::{SplitSink, StreamExt};
use futures::SinkExt;
use once_cell::sync::OnceCell;
use rand::seq::SliceRandom;
use rand::thread_rng;
use tokio::sync::Mutex;
use tower_http::services::ServeDir;

use std::collections::HashMap;
use std::ops::ControlFlow;
use std::sync::Arc;
use std::{net::SocketAddr, path::PathBuf};

#[tokio::main]
async fn main() {
    let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");

    // build our application with some routes
    let app = Router::new()
        .fallback_service(ServeDir::new(assets_dir).append_index_html_on_directories(true))
        .route("/ws", get(ws_handler));

    // run it with hyper
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    println!("`{user_agent}` at {addr} connected.");
    // finalize the upgrade process by returning upgrade callback.
    // we can customize the callback by sending additional info such as address.
    ws.on_upgrade(move |socket| handle_socket(socket, addr))
}

static SENDERS: OnceCell<Mutex<Vec<Arc<Mutex<SplitSink<WebSocket, ws::Message>>>>>> =
    OnceCell::new();

/// Actual websocket statemachine (one will be spawned per connection)
async fn handle_socket(socket: WebSocket, who: SocketAddr) {
    let (sender, mut receiver) = socket.split();
    let sender = Arc::new(Mutex::new(sender));
    let senders = SENDERS.get_or_init(|| Mutex::new(Vec::new()));
    senders.lock().await.push(sender.clone());
    let recv = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if process_message(msg, who, &*sender).await.is_break() {
                return;
            }
        }
    });
    recv.await.unwrap();
}

/// helper to print contents of messages to stdout. Has special treatment for Close.
async fn process_message(
    msg: ws::Message,
    who: SocketAddr,
    sender: &Mutex<SplitSink<WebSocket, ws::Message>>,
) -> ControlFlow<(), ()> {
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
            let (response, broadcast) = handle_message(msg).await;

            if let Some(broadcast) = broadcast {
                for s in SENDERS.get().unwrap().lock().await.iter_mut() {
                    s.lock()
                        .await
                        .send(ws::Message::Text(
                            serde_json::to_string(&broadcast).unwrap(),
                        ))
                        .await
                        .unwrap();
                }
            }
            if let Some(response) = response {
                sender
                    .lock()
                    .await
                    .send(ws::Message::Text(serde_json::to_string(&response).unwrap()))
                    .await
                    .unwrap();
            }
        }
        ws::Message::Binary(d) => {
            println!(">>> {} sent {} bytes: {:?}", who, d.len(), d);
        }
        ws::Message::Close(Some(cf)) => {
            println!(
                ">>> {} sent close with code {} and reason `{}`",
                who, cf.code, cf.reason
            );
            return ControlFlow::Break(());
        }
        ws::Message::Close(None) => {
            println!(">>> {} sent close message without CloseFrame", who);
            return ControlFlow::Break(());
        }

        ws::Message::Pong(v) => {
            println!(">>> {} sent pong with {:?}", who, v);
        }
        ws::Message::Ping(v) => {
            println!(">>> {} sent ping with {:?}", who, v);
        }
    }
    ControlFlow::Continue(())
}

static STATE: OnceCell<Mutex<State>> = OnceCell::new();

async fn handle_message(msg: Message) -> (Option<Response>, Option<Response>) {
    let mut state = STATE.get_or_init(|| Mutex::new(State::new())).lock().await;
    match msg {
        Message::JoinGame { player_name } => {
            let ((id, hand), s) = match &mut *state {
                State::WaitingForPlayers(s) => {
                    let (id, state) = s.join(player_name.clone());
                    let hand = state.hand.clone().try_into().expect("");
                    ((id, hand), s)
                }
                _ => todo!("Handle bad message"),
            };
            (
                Some(Response::Joined(JoinedResponse {
                    id,
                    hand,
                    players: s.players.values().map(|p| p.name.clone()).collect(),
                })),
                Some(Response::OtherJoined { name: player_name }),
            )
        }
        Message::Debug => {
            println!("{state:#?}");
            (None, None)
        }
    }
}

#[derive(serde::Deserialize)]
#[serde(tag = "event")]
#[serde(rename_all = "snake_case")]
enum Message {
    // State: WaitingForPlayers
    JoinGame { player_name: String },
    // Other
    Debug,
}

#[derive(serde::Serialize)]
#[serde(tag = "event")]
#[serde(rename_all = "snake_case")]
enum Response {
    Joined(JoinedResponse),
    OtherJoined { name: String },
}

#[derive(serde::Serialize)]
struct JoinedResponse {
    id: u64,
    hand: [u8; 10],
    players: Vec<String>,
}

#[derive(Debug)]
enum State {
    WaitingForPlayers(WaitingForPlayersState),
    Game(GameState),
}

impl State {
    fn new() -> Self {
        Self::WaitingForPlayers(WaitingForPlayersState::new())
    }
}

#[derive(Debug)]
struct WaitingForPlayersState {
    deck: Deck,
    players: HashMap<u64, PlayerState>,
}

impl WaitingForPlayersState {
    fn new() -> Self {
        Self {
            deck: Deck::new(),
            players: HashMap::new(),
        }
    }

    fn join(&mut self, name: String) -> (u64, &PlayerState) {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        name.hash(&mut hasher);
        let key = hasher.finish();
        let mut hand: Vec<_> = (0..10).into_iter().map(|_| self.deck.deal()).collect();
        hand.sort();
        let state = self.players.entry(key.clone()).or_insert(PlayerState {
            name,
            points: 0,
            hand,
        });
        (key, state)
    }
}

#[derive(Debug)]
struct GameState {
    players: HashMap<u64, PlayerState>,
    piles: [Pile; 4],
    deck: Deck,
    round: Round,
}

impl GameState {
    pub fn new() -> Self {
        let mut deck = Deck::new();
        Self {
            players: HashMap::default(),
            piles: [
                Pile::new(deck.deal()),
                Pile::new(deck.deal()),
                Pile::new(deck.deal()),
                Pile::new(deck.deal()),
            ],
            deck,
            round: Round { number: 1 },
        }
    }
}

#[derive(Debug)]
struct PlayerState {
    name: String,
    points: u16,
    hand: Vec<u8>,
}

#[derive(Debug)]
struct Round {
    number: u8,
}

#[derive(Debug)]
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
