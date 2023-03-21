use std::collections::HashMap;

use rand::{seq::SliceRandom, thread_rng};
use serde_json::Value;

#[derive(Debug)]
pub enum State {
    Lobby(Lobby),
    Game(Game),
}

impl State {
    pub(crate) fn new() -> State {
        State::Lobby(Lobby::new())
    }

    pub fn join(&mut self, name: String) -> Result<String, StateError> {
        match self {
            State::Lobby(l) => Ok(l.join(name)),
            State::Game(_) => todo!(),
        }
    }

    pub fn serialize_for_user(&self, user_id: &str) -> Value {
        todo!()
    }
}

pub enum StateError {}

#[derive(Debug)]
pub struct Lobby {
    table: Table,
    players: PlayerMapping,
}

impl Lobby {
    fn new() -> Self {
        Self {
            table: Table::new(),
            players: PlayerMapping::new(),
        }
    }

    fn join(&mut self, name: String) -> String {
        self.players.join(name, &mut self.table.deck)
    }

    fn start_game(self) -> Result<Game, GameStartError> {
        if self.players.num() < MIN_PLAYERS {
            return Err(GameStartError::NotEnoughPlayers(self.players.num()));
        }
        Ok(Game::new(self.table, self.players))
    }

    fn players(&self) -> Vec<String> {
        self.players.players()
    }

    fn get_player(&self, id: &str) -> Option<&Player> {
        self.players.get(id)
    }
}

const MIN_PLAYERS: usize = 2;
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
}

/// A mapping of IDs to player names
#[derive(Debug)]
struct PlayerMapping(HashMap<String, Player>);

impl PlayerMapping {
    fn new() -> PlayerMapping {
        PlayerMapping(HashMap::new())
    }

    fn players(&self) -> Vec<String> {
        self.0.values().map(|p| p.name.clone()).collect()
    }

    fn join(&mut self, name: String, deck: &mut Deck) -> String {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        name.hash(&mut hasher);
        let key = hasher.finish();

        let mut hand: Vec<_> = (0..10).into_iter().map(|_| deck.deal()).collect();
        hand.sort();

        // TODO: handle if the player was already added
        self.0.insert(
            key.to_string(),
            Player {
                name,
                points: 0,
                hand,
            },
        );
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
}

#[derive(Debug)]
struct Player {
    name: String,
    points: u16,
    hand: Vec<u8>,
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

#[derive(serde::Serialize, Debug)]
struct Piles([Pile; 4]);

impl Piles {
    fn new(deck: &mut Deck) -> Self {
        let mut piles = [
            Pile::new(deck.deal()),
            Pile::new(deck.deal()),
            Pile::new(deck.deal()),
            Pile::new(deck.deal()),
        ];
        piles.sort_by(|p1, p2| p1.top_card().cmp(&p2.top_card()));
        Self(piles)
    }

    fn play(&mut self, card: u8) -> Option<Option<u16>> {
        let pile = self.pile_for_card_mut(card);
        if let Some(pile) = pile {
            Some(pile.place(card))
        } else {
            None
        }
    }

    fn pile_for_card_mut(&mut self, card: u8) -> Option<&mut Pile> {
        self.0
            .iter_mut()
            .map(|p| (p.top_card() as i8 - card as i8, p))
            .take_while(|(diff, _)| *diff < 0)
            .map(|(_, p)| p)
            .last()
    }

    fn pile_for_card(&self, card: u8) -> Option<(i8, &Pile)> {
        self.0
            .iter()
            .map(|p| (p.top_card() as i8 - card as i8, p))
            .take_while(|(diff, _)| *diff < 0)
            .last()
    }

    fn can_place(&self, card: u8) -> bool {
        self.pile_for_card(card).is_some()
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

    fn place(&mut self, card: u8) -> Option<u16> {
        self.0.push(card);
        if self.0.len() == 6 {
            let old = std::mem::replace(&mut self.0, vec![card]);
            println!("TODO");
            Some(0)
        } else {
            None
        }
    }

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
    fn play(&mut self, user_id: &str, card: u8) -> Result<(), PlacementError> {
        enum NextStep {
            PileSelection(String),
            ApplyPlay(CardPlay),
        }
        let next_step = match &mut self.turn {
            Turn::CardPlay(p) => {
                match self.players.get(user_id) {
                    Some(player) if player.hand.contains(&card) => {
                        p.play(user_id.to_owned(), card)?;
                    }
                    Some(_) => return Err(PlacementError::CardNotInHand),
                    None => return Err(PlacementError::NoUser),
                }

                if p.num() != self.players.num() {
                    return Ok(());
                }
                let c = p
                    .plays()
                    .find(|(_, card)| !self.table.piles.can_place(*card));
                if let Some((player_id, _)) = c {
                    NextStep::PileSelection(player_id.to_owned())
                } else {
                    NextStep::ApplyPlay(p.clone())
                }
            }
            _ => return Err(PlacementError::PlacementOutOfTurn),
        };
        match next_step {
            NextStep::PileSelection(p) => self.turn = Turn::PileSelection(p),
            NextStep::ApplyPlay(cp) => {
                self.apply_card_play(cp);
                self.turn = Turn::CardPlay(CardPlay::new());
                self.round.inc();
            }
        }
        Ok(())
    }

    fn apply_card_play(&mut self, cp: CardPlay) {
        for (user_id, card) in cp.plays() {
            if let Some(points) = self.table.piles.play(card).unwrap() {
                let player = self.players.get_mut(user_id).unwrap();
                player.hand.retain(|c| *c != card);
                player.points += points;
            }
        }
    }

    fn players(&self) -> &PlayerMapping {
        &self.players
    }

    fn round(&self) -> Round {
        self.round
    }
}

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
    CardPlay(CardPlay),
    PileSelection(String),
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

    fn play(&mut self, user_id: String, card: u8) -> Result<(), PlacementError> {
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

    fn empty(&mut self) {
        self.0.clear();
    }

    /// All the cards played in this round
    fn plays(&self) -> impl Iterator<Item = (&str, u8)> {
        self.0
            .iter()
            .map(|(player_id, card)| (player_id.as_str(), *card))
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
struct Round(u8);

impl Round {
    fn inc(&mut self) {
        *self = Self(self.0 + 1)
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
        let bill = game.players().get(&bill_id).unwrap();
        let ted = game.players().get(&ted_id).unwrap();

        let ted_last = *ted.hand.last().unwrap();
        let bill_first = *bill.hand.first().unwrap();
        let bill_last = *bill.hand.last().unwrap();

        // Can't play another player's card
        assert!(game.play(&bill_id, ted_last).is_err());
        // Can play player's own card
        assert!(game.play(&bill_id, bill_last).is_ok());
        // Can't play twice
        assert!(game.play(&bill_id, bill_first).is_err());

        assert!(game.play(&ted_id, ted_last).is_ok());

        assert_eq!(game.round(), Round(2));

        let cards_in_piles: usize = game.table.piles.0.iter().map(|p| p.num()).sum();
        assert_eq!(cards_in_piles, 6);
    }
}
