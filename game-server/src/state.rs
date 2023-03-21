use std::collections::{HashMap, HashSet};

use rand::{seq::SliceRandom, thread_rng};
use serde_json::{json, Value};

#[derive(Debug)]
pub enum State {
    Lobby(Lobby),
    Game(Game),
    GameOver(PlayerMapping),
}

impl State {
    pub(crate) fn new() -> State {
        State::Lobby(Lobby::new())
    }

    pub fn join(&mut self, name: String) -> Result<String, StateError> {
        match self {
            State::Lobby(l) => Ok(l.join(name)),
            State::Game(_) => todo!("Handle error"),
            State::GameOver(_) => todo!("Handle error"),
        }
    }

    pub fn start_game(&mut self) -> Result<(), StateError> {
        match self {
            State::Lobby(l) => {
                *self = State::Game(l.start_game().expect("TODO: handle error"));
                Ok(())
            }
            State::Game(_) => todo!("Handle error"),
            State::GameOver(_) => todo!("Handle error"),
        }
    }

    pub fn play_card(&mut self, user_id: &str, card: u8) -> Result<(), StateError> {
        match self {
            State::Lobby(_) => todo!("Handle error"),
            State::Game(g) => {
                if g.play_card(user_id, card).expect("TODO: handle error") {
                    *self = State::GameOver(g.player_mapping().clone())
                }
            }
            State::GameOver(_) => todo!("Handle error"),
        }
        Ok(())
    }

    pub fn select_pile(&mut self, user_id: &str, pile_index: usize) -> Result<(), StateError> {
        match self {
            State::Lobby(_) => todo!("Handle error"),
            State::Game(g) => {
                if g.select_pile(user_id, pile_index)
                    .expect("TODO: handle error")
                {
                    *self = State::GameOver(g.player_mapping().clone())
                }
            }
            State::GameOver(_) => todo!("Handle error"),
        }
        Ok(())
    }

    pub fn restart(&mut self) {
        match self {
            State::Lobby(_) => {}
            State::Game(g) => {
                let mut table = Table::new();
                let mut players = g.player_mapping().clone();
                players.reset(table.deck_mut());
                *self = State::Lobby(Lobby::new_from_parts(table, players))
            }
            State::GameOver(p) => {
                let mut table = Table::new();
                let mut players = p.clone();
                players.reset(table.deck_mut());
                *self = State::Lobby(Lobby::new_from_parts(table, players));
            }
        }
    }

    pub fn serialize_for_user(&self, user_id: &str) -> Value {
        match self {
            State::Lobby(l) => {
                let players = l.players();
                json!({
                    "state": "lobby",
                    "players": players,
                })
            }
            State::Game(g) => {
                let hand = g.hand_for(user_id).unwrap();
                let points = g.points_for(user_id).unwrap();
                let turn_state = match g.turn() {
                    Turn::CardPlay(_) => "play",
                    Turn::PileSelection(i, _) if i == user_id => "select_pile",
                    Turn::PileSelection(_, _) => "other_select_pile",
                };
                let waiting_for = g.waiting_for();

                json!({
                    "state": "game",
                    "players": g.player_scores(),
                    "round": {
                        "number": g.round(),
                        "state": turn_state,
                        "waiting_for": waiting_for,
                        "played": g.played_card_for(user_id),
                    },
                    "piles": g.piles(),
                    "hand": hand,
                    "points": points
                })
            }
            State::GameOver(p) => {
                json!({
                    "state": "game_over",
                    "players": p.player_scores()
                })
            }
        }
    }

    pub fn get_player(&self, user_id: &str) -> Option<&Player> {
        match self {
            State::Lobby(l) => l.get_player(user_id),
            State::Game(g) => g.get_player(user_id),
            State::GameOver(p) => p.get(user_id),
        }
    }
}

#[derive(Debug)]
pub enum StateError {}

#[derive(Debug)]
pub struct Lobby {
    table: Table,
    players: PlayerMapping,
}

impl Lobby {
    fn new() -> Self {
        Self::new_from_parts(Table::new(), PlayerMapping::new())
    }

    fn new_from_parts(table: Table, players: PlayerMapping) -> Lobby {
        Self { table, players }
    }

    fn join(&mut self, name: String) -> String {
        self.players.join(name, &mut self.table.deck)
    }

    fn start_game(&mut self) -> Result<Game, GameStartError> {
        if self.players.num() < MIN_PLAYERS {
            return Err(GameStartError::NotEnoughPlayers(self.players.num()));
        }
        let table = std::mem::replace(&mut self.table, Table::new());
        let players = std::mem::replace(&mut self.players, PlayerMapping::new());
        Ok(Game::new(table, players))
    }

    fn players(&self) -> Vec<String> {
        self.players.players()
    }

    fn get_player(&self, id: &str) -> Option<&Player> {
        self.players.get(id)
    }
}

const MIN_PLAYERS: usize = 2;

#[derive(Debug)]
enum GameStartError {
    NotEnoughPlayers(usize),
}

#[derive(Debug)]
struct Table {
    deck: Deck,
    piles: Piles,
}

impl Table {
    fn new() -> Self {
        let mut deck = Deck::new();
        Self {
            piles: Piles::new(&mut deck),
            deck,
        }
    }

    fn deck_mut(&mut self) -> &mut Deck {
        &mut self.deck
    }
}

/// A mapping of IDs to player names
#[derive(Debug, Clone)]
pub struct PlayerMapping(HashMap<String, Player>);

impl PlayerMapping {
    fn new() -> PlayerMapping {
        PlayerMapping(HashMap::new())
    }

    fn reset(&mut self, deck: &mut Deck) {
        for player in self.0.values_mut() {
            *player = Player::new(std::mem::replace(&mut player.name, String::new()), deck)
        }
    }

    fn players(&self) -> Vec<String> {
        self.0.values().map(|p| p.name.clone()).collect()
    }

    fn join(&mut self, name: String, deck: &mut Deck) -> String {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        name.hash(&mut hasher);
        let key = hasher.finish();

        // TODO: handle if the player was already added
        self.0.insert(key.to_string(), Player::new(name, deck));
        key.to_string()
    }

    fn num(&self) -> usize {
        self.0.len()
    }

    fn get(&self, user_id: &str) -> Option<&Player> {
        self.0.get(user_id)
    }

    fn get_mut(&mut self, user_id: &str) -> Option<&mut Player> {
        self.0.get_mut(user_id)
    }

    fn player_name(&self, user_id: &str) -> Option<&str> {
        self.get(user_id).map(|p| p.name.as_str())
    }

    fn ids(&self) -> impl Iterator<Item = &str> {
        self.0.keys().map(|k| k.as_str())
    }

    fn player_scores(&self) -> HashMap<String, u16> {
        self.0
            .values()
            .map(|p| (p.name.clone(), p.points))
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct Player {
    name: String,
    points: u16,
    hand: Vec<u8>,
}

impl Player {
    fn new(name: String, deck: &mut Deck) -> Self {
        let mut hand: Vec<_> = (0..10).into_iter().map(|_| deck.deal()).collect();
        hand.sort();
        Self {
            name,
            points: 0,
            hand,
        }
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
        self.cards.pop().expect("Deck should never be fully dealt")
    }
}

/// The card piles
///
/// The piles are always kept in sorted order
#[derive(serde::Serialize, Debug)]
struct Piles([Pile; 4]);

impl Piles {
    fn new(deck: &mut Deck) -> Self {
        let piles = [
            Pile::new(deck.deal()),
            Pile::new(deck.deal()),
            Pile::new(deck.deal()),
            Pile::new(deck.deal()),
        ];
        let mut s = Self(piles);
        s.sort();
        s
    }

    fn place(&mut self, card: u8) -> Option<Option<u16>> {
        let pile = self.pile_for_card_mut(card)?;
        let points = pile.place(card);
        self.sort();
        Some(points)
    }

    fn can_place(&self, card: u8) -> bool {
        self.pile_for_card(card).is_some()
    }

    fn replace_pile(&mut self, pile_index: usize, card: u8) -> u16 {
        let old = std::mem::replace(self.0.get_mut(pile_index).unwrap(), Pile::new(card));
        self.sort();
        old.points()
    }

    fn sort(&mut self) {
        self.0.sort_by(|p1, p2| p1.top_card().cmp(&p2.top_card()));
    }

    fn pile_for_card(&self, card: u8) -> Option<&Pile> {
        self.0
            .iter()
            .map(|p| (p.top_card() as i8 - card as i8, p))
            .take_while(|(diff, _)| *diff < 0)
            .map(|(_, p)| p)
            .last()
    }

    fn pile_for_card_mut(&mut self, card: u8) -> Option<&mut Pile> {
        self.0
            .iter_mut()
            .map(|p| (p.top_card() as i8 - card as i8, p))
            .take_while(|(diff, _)| *diff < 0)
            .map(|(_, p)| p)
            .last()
    }
}

#[derive(serde::Serialize, Debug)]
struct Pile(Vec<u8>);

impl Pile {
    fn new(card: u8) -> Self {
        Self(vec![card])
    }

    fn top_card(&self) -> u8 {
        *self.0.last().unwrap()
    }

    // Places the card in the pile
    //
    // Returns `Some` if pile was converted to points
    fn place(&mut self, card: u8) -> Option<u16> {
        if self.0.len() == 5 {
            let old = std::mem::replace(self, Pile::new(card));
            Some(old.points())
        } else {
            self.0.push(card);
            None
        }
    }

    /// Given a pile calculate how much that pile's points are
    fn points(&self) -> u16 {
        self.0
            .iter()
            .map(|card| match (card % 11, card % 5) {
                (0, _) if *card == 55 => 6,
                (0, _) => 5,
                (_, 0) if card % 10 == 0 => 3,
                (_, 0) => 2,
                _ => 1,
            })
            .sum()
    }

    #[cfg(test)]
    fn num(&self) -> usize {
        self.0.len()
    }
}

#[derive(Debug)]
pub struct Game {
    table: Table,
    players: PlayerMapping,
    turn: Turn,
    round: Round,
}

impl Game {
    fn new(table: Table, players: PlayerMapping) -> Game {
        Game {
            table,
            players,
            turn: Turn::CardPlay(CardPlay::new()),
            round: Round(1),
        }
    }

    /// Play a card as a user
    ///
    /// Returns `Ok(true)` if the game is over
    fn play_card(&mut self, user_id: &str, card: u8) -> Result<bool, PlacementError> {
        enum NextStep {
            PileSelection(String, CardPlay),
            ApplyPlay(CardPlay),
        }
        let next_step = match &mut self.turn {
            Turn::CardPlay(p) => {
                match self.players.get(user_id) {
                    Some(player) if player.hand.contains(&card) => {
                        p.play_card(user_id.to_owned(), card)?;
                    }
                    Some(_) => return Err(PlacementError::CardNotInHand),
                    None => return Err(PlacementError::NoUser),
                }

                // Just return if the round is still in progress
                if p.num() != self.players.num() {
                    return Ok(false);
                }

                let player_must_select_pile = p
                    .plays()
                    .find(|(_, card)| !self.table.piles.can_place(*card))
                    .map(|(player_id, _)| player_id.to_owned());
                let cp = std::mem::replace(p, CardPlay::new());
                if let Some(player_id) = player_must_select_pile {
                    NextStep::PileSelection(player_id, cp)
                } else {
                    NextStep::ApplyPlay(cp)
                }
            }
            _ => return Err(PlacementError::PlacementOutOfTurn),
        };
        match next_step {
            NextStep::PileSelection(p, cp) => self.turn = Turn::PileSelection(p, cp),
            NextStep::ApplyPlay(cp) => {
                self.apply_card_play(&cp);
                self.turn = Turn::CardPlay(CardPlay::new());
                return Ok(self.round.inc());
            }
        }
        Ok(false)
    }

    /// Selects a pile for a player (turning that pile into points)
    ///
    /// Returns `Ok(true)` if the game is over
    fn select_pile(&mut self, user_id: &str, pile_index: usize) -> Result<bool, PlacementError> {
        let (cp, card, points) = match &mut self.turn {
            Turn::PileSelection(i, cp) if i == user_id => {
                let card = cp.remove_card(i).unwrap();
                let points = self.table.piles.replace_pile(pile_index, card);
                (cp.clone(), card, points)
            }
            Turn::PileSelection(_, _) => todo!("Handle error"),
            Turn::CardPlay(_) => todo!("Handle error"),
        };
        self.apply_play_to_user(user_id, card, Some(points));
        self.apply_card_play(&cp);
        self.turn = Turn::CardPlay(CardPlay::new());
        Ok(self.round.inc())
    }

    fn apply_card_play(&mut self, cp: &CardPlay) {
        for (user_id, card) in cp.plays() {
            let points = self.table.piles.place(card).unwrap();
            self.apply_play_to_user(user_id, card, points);
        }
    }

    fn apply_play_to_user(&mut self, user_id: &str, card: u8, points: Option<u16>) {
        let player = self.players.get_mut(user_id).unwrap();
        player.hand.retain(|c| *c != card);
        player.points += points.unwrap_or_default();
    }

    fn player_mapping(&self) -> &PlayerMapping {
        &self.players
    }

    fn round(&self) -> Round {
        self.round
    }

    fn turn(&self) -> &Turn {
        &self.turn
    }

    fn piles(&self) -> &Piles {
        &self.table.piles
    }

    fn hand_for(&self, user_id: &str) -> Option<&[u8]> {
        self.players.get(user_id).map(|p| p.hand.as_slice())
    }

    fn waiting_for(&self) -> Vec<String> {
        let player_name = |user_id| self.players.player_name(user_id).unwrap().to_owned();
        match &self.turn {
            Turn::CardPlay(p) => {
                let all_players = self.players.ids();
                let already_played_players = p.user_ids().collect::<HashSet<&str>>();
                all_players
                    .filter(|p| !already_played_players.contains(p))
                    .map(player_name)
                    .collect()
            }
            Turn::PileSelection(user_id, _) => vec![player_name(user_id)],
        }
    }

    fn get_player(&self, user_id: &str) -> Option<&Player> {
        self.players.get(user_id)
    }

    fn points_for(&self, user_id: &str) -> Option<u16> {
        self.players.get(user_id).map(|p| p.points)
    }

    fn player_scores(&self) -> HashMap<String, u16> {
        self.players.player_scores()
    }

    /// Get the card played in this round by this user if they've played
    fn played_card_for(&self, user_id: &str) -> Option<u8> {
        self.turn.played_card_for(user_id)
    }
}

#[derive(Debug)]
enum PlacementError {
    /// Tried to place a card when it was the time for card placement
    PlacementOutOfTurn,
    /// Played placed a card after already placing one
    RepeatedPlacement,
    CardNotInHand,
    NoUser,
}

#[derive(Debug)]
enum Turn {
    /// Cards are being played
    CardPlay(CardPlay),
    /// A user must select a pile
    PileSelection(String, CardPlay),
}
impl Turn {
    fn played_card_for(&self, user_id: &str) -> Option<u8> {
        match self {
            Turn::CardPlay(c) => c.played_card_for(user_id),
            Turn::PileSelection(_, c) => c.played_card_for(user_id),
        }
    }
}

/// The cards played in a round
///
/// Ordered from smallest card to largest
#[derive(Debug, Clone)]
struct CardPlay(Vec<(String, u8)>);

impl CardPlay {
    fn new() -> Self {
        Self(Vec::default())
    }

    fn play_card(&mut self, user_id: String, card: u8) -> Result<(), PlacementError> {
        if self.0.iter().any(|(uid, _)| &user_id == uid) {
            return Err(PlacementError::RepeatedPlacement);
        }

        self.0.push((user_id, card));
        self.0.sort_by(|(_, c1), (_, c2)| c1.cmp(c2));
        Ok(())
    }

    /// The number of cards placed
    fn num(&self) -> usize {
        self.0.len()
    }

    /// All the cards played in this round
    fn plays(&self) -> impl Iterator<Item = (&str, u8)> {
        self.0
            .iter()
            .map(|(player_id, card)| (player_id.as_str(), *card))
    }

    fn user_ids(&self) -> impl Iterator<Item = &str> {
        self.0.iter().map(|(user_id, _)| user_id.as_str())
    }

    /// Remove the user's card from the card play
    fn remove_card(&mut self, user_id: &str) -> Option<u8> {
        let i = self.0.iter().position(|(uid, _)| uid == user_id)?;
        Some(self.0.remove(i).1)
    }

    fn played_card_for(&self, user_id: &str) -> Option<u8> {
        self.0
            .iter()
            .find_map(|(uid, card)| (uid == user_id).then(|| *card))
    }
}

#[derive(serde::Serialize, Debug, PartialEq, Eq, Copy, Clone)]
struct Round(u8);

impl Round {
    // Increments round, returns true if game is over
    fn inc(&mut self) -> bool {
        *self = Self(self.0 + 1);
        self.0 > 10
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lobby() {
        let mut lobby = Lobby::new();
        lobby.join("Bill".to_owned());
        let ted_id = lobby.join("Ted".to_owned());

        let mut players = lobby.players();
        players.sort();
        assert_eq!(players, vec!["Bill", "Ted"]);

        let ted = lobby.get_player(&ted_id).unwrap();
        assert_eq!(ted.name, "Ted");
        assert_eq!(ted.hand.len(), 10);

        assert!(lobby.start_game().is_ok());
    }

    #[test]
    fn test_game() {
        let table = Table::new();
        let mut deck = Deck::new();
        let mut players = PlayerMapping::new();
        let bill_id = players.join("Bill".to_owned(), &mut deck);
        let ted_id = players.join("Ted".to_owned(), &mut deck);
        let mut game = Game::new(table, players);
        let bill = game.player_mapping().get(&bill_id).unwrap();
        let ted = game.player_mapping().get(&ted_id).unwrap();

        let ted_last = *ted.hand.last().unwrap();
        let bill_first = *bill.hand.first().unwrap();
        let bill_last = *bill.hand.last().unwrap();

        // Can't play another player's card
        assert!(game.play_card(&bill_id, ted_last).is_err());
        // Can play player's own card
        assert!(game.play_card(&bill_id, bill_last).is_ok());
        // Can't play twice
        assert!(game.play_card(&bill_id, bill_first).is_err());

        assert!(game.play_card(&ted_id, ted_last).is_ok());

        assert_eq!(game.round(), Round(2));

        let cards_in_piles: usize = game.table.piles.0.iter().map(|p| p.num()).sum();
        assert_eq!(cards_in_piles, 6);
    }
}
