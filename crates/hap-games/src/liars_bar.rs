use anyhow::{Result, anyhow};
use hap_models::{GameCatalogItem, SessionStatus};
use rand::prelude::IndexedRandom;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use serde_json::{Value, from_value, json, to_value};
use std::collections::{BTreeSet, HashMap};
use std::sync::LazyLock;

use crate::{GameAdapter, GameTransition, StateAudience};

const SIDES: [&str; 4] = ["seat-1", "seat-2", "seat-3", "seat-4"];
const INITIAL_HAND_SIZE: usize = 5;
const REVOLVER_SIZE: u8 = 6;
const LIAR_DECK_KINGS: usize = 6;
const LIAR_DECK_QUEENS: usize = 6;
const LIAR_DECK_ACES: usize = 6;
const LIAR_DECK_JOKERS: usize = 2;
const MAX_PLAY_COUNT: usize = 3;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
enum CardRank {
    King,
    Queen,
    Ace,
    Joker,
}

impl CardRank {
    fn label(self) -> &'static str {
        match self {
            Self::King => "King",
            Self::Queen => "Queen",
            Self::Ace => "Ace",
            Self::Joker => "Joker",
        }
    }

    fn plural_label(self) -> &'static str {
        match self {
            Self::King => "Kings",
            Self::Queen => "Queens",
            Self::Ace => "Aces",
            Self::Joker => "Jokers",
        }
    }

    fn is_innocent_for(self, table_card: CardRank) -> bool {
        self == table_card || self == Self::Joker
    }
}

const TABLE_CARD_OPTIONS: [CardRank; 3] = [CardRank::King, CardRank::Queen, CardRank::Ace];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct PlayerState {
    side: String,
    display_name: String,
    alive: bool,
    shots_fired: u8,
    lethal_chamber: u8,
    hand: Vec<CardRank>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct PublicPlayerState {
    side: String,
    display_name: String,
    alive: bool,
    shots_fired: u8,
    hand_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct PlayRecord {
    side: String,
    display_name: String,
    declared_rank: CardRank,
    card_count: u8,
    cards: Vec<CardRank>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct PublicPlayRecord {
    side: String,
    display_name: String,
    declared_rank: CardRank,
    card_count: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct Resolution {
    caller_side: String,
    caller_display_name: String,
    accused_side: String,
    accused_display_name: String,
    punished_side: String,
    punished_display_name: String,
    revealed_cards: Vec<CardRank>,
    liar_found: bool,
    shots_fired: u8,
    survived: bool,
    next_round_number: Option<u32>,
    next_turn: Option<String>,
    winner: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct GameState {
    kind: String,
    turn: String,
    status: SessionStatus,
    winner: Option<String>,
    round_number: u32,
    phase: String,
    table_card: CardRank,
    previous_play: Option<PlayRecord>,
    last_resolution: Option<Resolution>,
    players: Vec<PlayerState>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct PublicGameState {
    kind: String,
    turn: String,
    status: SessionStatus,
    winner: Option<String>,
    round_number: u32,
    phase: String,
    table_card: CardRank,
    previous_play: Option<PublicPlayRecord>,
    last_resolution: Option<Resolution>,
    players: Vec<PublicPlayerState>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct SeatGameState {
    kind: String,
    turn: String,
    status: SessionStatus,
    winner: Option<String>,
    round_number: u32,
    phase: String,
    table_card: CardRank,
    previous_play: Option<PublicPlayRecord>,
    last_resolution: Option<Resolution>,
    players: Vec<PublicPlayerState>,
    self_side: String,
    private_hand: Vec<CardRank>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ActionInput {
    PlayCards { cards: Vec<CardRank> },
    CallLiar,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
enum LegalAction {
    PlayCards { cards: Vec<CardRank>, label: String },
    CallLiar { label: String },
}

pub(super) static ADAPTER: LiarsBarAdapter = LiarsBarAdapter;

pub(super) struct LiarsBarAdapter;

static GAME: LazyLock<GameCatalogItem> = LazyLock::new(|| {
    GameCatalogItem {
    id: "liars-bar".to_string(),
    title: "Liars Bar".to_string(),
    short_name: "Liars Bar".to_string(),
    description:
        "A text-first Liar's Deck table where four seats bluff with face-down cards, call LIAR, survive Russian Roulette, and fight to be the last player standing."
            .to_string(),
    sides: SIDES.iter().map(|side| side.to_string()).collect(),
}
});

impl GameAdapter for LiarsBarAdapter {
    fn game(&self) -> &GameCatalogItem {
        &GAME
    }

    fn create_initial_state(&self) -> Result<Value> {
        to_value(create_initial_state()).map_err(Into::into)
    }

    fn normalize_state(&self, state: Value) -> Result<Value> {
        let parsed: GameState = from_value(state)?;
        to_value(parsed).map_err(Into::into)
    }

    fn list_legal_moves(&self, state: &Value, _query: Option<&Value>) -> Result<Vec<Value>> {
        let parsed: GameState = from_value(state.clone())?;
        list_legal_actions(&parsed)?
            .into_iter()
            .map(|action| to_value(action).map_err(Into::into))
            .collect()
    }

    fn play_move(&self, state: &Value, input: &Value) -> Result<GameTransition> {
        let parsed: GameState = from_value(state.clone())?;
        let action: ActionInput = from_value(input.clone())?;
        play_move(&parsed, action)
    }

    fn project_state(&self, state: &Value, audience: StateAudience<'_>) -> Result<Value> {
        let parsed: GameState = from_value(state.clone())?;
        match audience {
            StateAudience::Public => to_value(project_public_state(&parsed)).map_err(Into::into),
            StateAudience::Seat(side) => {
                to_value(project_seat_state(&parsed, side)?).map_err(Into::into)
            }
        }
    }
}

fn create_initial_state() -> GameState {
    let mut state = GameState {
        kind: "liars-bar".to_string(),
        turn: SIDES[0].to_string(),
        status: SessionStatus::Active,
        winner: None,
        round_number: 0,
        phase: "playing".to_string(),
        table_card: CardRank::King,
        previous_play: None,
        last_resolution: None,
        players: SIDES
            .iter()
            .enumerate()
            .map(|(index, side)| PlayerState {
                side: (*side).to_string(),
                display_name: format!("P{}", index + 1),
                alive: true,
                shots_fired: 0,
                lethal_chamber: draw_lethal_chamber(),
                hand: Vec::new(),
            })
            .collect(),
    };

    state = start_next_round(state, SIDES[0], 1);
    state
}

fn list_legal_actions(state: &GameState) -> Result<Vec<LegalAction>> {
    if state.status == SessionStatus::Finished {
        return Ok(Vec::new());
    }

    let current_player = find_player(state, &state.turn)?;
    if !current_player.alive {
        return Err(anyhow!(
            "The current Liars Bar turn belongs to an eliminated seat"
        ));
    }

    let mut actions = Vec::new();
    if state.previous_play.is_some() {
        actions.push(LegalAction::CallLiar {
            label: "Call LIAR on the previous play".to_string(),
        });
    }

    if can_play_cards(state, current_player) {
        actions.extend(
            unique_play_combinations(&current_player.hand)
                .into_iter()
                .map(|cards| LegalAction::PlayCards {
                    label: format!(
                        "Play {} face-down as {}",
                        describe_card_count(cards.len()),
                        state.table_card.plural_label()
                    ),
                    cards,
                }),
        );
    }

    Ok(actions)
}

fn play_move(state: &GameState, action: ActionInput) -> Result<GameTransition> {
    if state.status == SessionStatus::Finished {
        return Err(anyhow!(
            "Cannot play an action after Liars Bar has finished"
        ));
    }

    let current_player = find_player(state, &state.turn)?;
    if !current_player.alive {
        return Err(anyhow!(
            "The current Liars Bar turn belongs to an eliminated seat"
        ));
    }

    let legal_actions = list_legal_actions(state)?;
    match action {
        ActionInput::PlayCards { cards } => {
            let requested = LegalAction::PlayCards {
                label: format!(
                    "Play {} face-down as {}",
                    describe_card_count(cards.len()),
                    state.table_card.plural_label()
                ),
                cards: normalize_cards(cards),
            };
            if !legal_actions.contains(&requested) {
                return Err(anyhow!(
                    "The selected card play is not legal in the current round"
                ));
            }

            let played_cards = requested_cards(&requested);
            let mut next_state = state.clone();
            {
                let player = find_player_mut(&mut next_state, &state.turn)?;
                remove_cards_from_hand(&mut player.hand, &played_cards)?;
            }

            next_state.previous_play = Some(PlayRecord {
                side: state.turn.clone(),
                display_name: current_player.display_name.clone(),
                declared_rank: state.table_card,
                card_count: played_cards.len() as u8,
                cards: played_cards.clone(),
            });
            next_state.last_resolution = None;
            next_state.phase = if count_alive_players_with_cards(&next_state) <= 1 {
                "forced_challenge".to_string()
            } else {
                "playing".to_string()
            };
            next_state.turn = next_turn_after_play(&next_state, &state.turn)?;
            let next_turn = next_state.turn.clone();

            Ok(GameTransition {
                next_state: to_value(next_state)?,
                event_kind: hap_models::SessionEventKind::ActionTaken,
                event_summary: format!(
                    "{} played {} face down as {}.",
                    current_player.display_name,
                    describe_card_count(played_cards.len()),
                    state.table_card.plural_label()
                ),
                event_details: map_from_pairs([
                    (
                        "headline",
                        json!(format!(
                            "{} plays {} as {}",
                            current_player.display_name,
                            describe_card_count(played_cards.len()),
                            state.table_card.plural_label()
                        )),
                    ),
                    ("actionType", json!("play_cards")),
                    ("side", json!(state.turn)),
                    ("displayName", json!(current_player.display_name)),
                    ("tableCard", json!(state.table_card)),
                    ("cardCount", json!(played_cards.len())),
                    ("nextTurn", json!(next_turn)),
                    ("roundNumber", json!(state.round_number)),
                ]),
            })
        }
        ActionInput::CallLiar => {
            if !legal_actions.contains(&LegalAction::CallLiar {
                label: "Call LIAR on the previous play".to_string(),
            }) {
                return Err(anyhow!("You cannot call LIAR before a play has been made"));
            }

            let previous_play = state
                .previous_play
                .clone()
                .ok_or_else(|| anyhow!("Cannot resolve LIAR without a previous play"))?;
            let liar_found = previous_play
                .cards
                .iter()
                .any(|card| !card.is_innocent_for(state.table_card));

            let punished_side = if liar_found {
                previous_play.side.clone()
            } else {
                state.turn.clone()
            };
            let punished_display_name = if liar_found {
                previous_play.display_name.clone()
            } else {
                current_player.display_name.clone()
            };

            let mut next_state = state.clone();
            let (punished_alive, shots_fired) = {
                let punished_player = find_player_mut(&mut next_state, &punished_side)?;
                punished_player.shots_fired = punished_player.shots_fired.saturating_add(1);
                if punished_player.shots_fired == punished_player.lethal_chamber {
                    punished_player.alive = false;
                    punished_player.hand.clear();
                }
                (punished_player.alive, punished_player.shots_fired)
            };

            let winner = remaining_alive_sides(&next_state)
                .into_iter()
                .next()
                .filter(|_| count_alive_players(&next_state) == 1);
            next_state.winner = winner.clone();
            next_state.status = if winner.is_some() {
                SessionStatus::Finished
            } else {
                SessionStatus::Active
            };

            let next_round_number = winner
                .as_ref()
                .map(|_| None)
                .unwrap_or(Some(state.round_number + 1));
            let next_turn = if winner.is_some() {
                None
            } else if punished_alive {
                Some(punished_side.clone())
            } else {
                next_alive_side(&next_state, &punished_side)
            };

            next_state.last_resolution = Some(Resolution {
                caller_side: state.turn.clone(),
                caller_display_name: current_player.display_name.clone(),
                accused_side: previous_play.side.clone(),
                accused_display_name: previous_play.display_name.clone(),
                punished_side: punished_side.clone(),
                punished_display_name: punished_display_name.clone(),
                revealed_cards: previous_play.cards.clone(),
                liar_found,
                shots_fired,
                survived: punished_alive,
                next_round_number,
                next_turn: next_turn.clone(),
                winner: winner.clone(),
            });

            if let (Some(round_number), Some(starter)) = (next_round_number, next_turn.as_deref()) {
                next_state = start_next_round(next_state, starter, round_number);
            } else {
                next_state.phase = "finished".to_string();
                next_state.previous_play = None;
                next_state.turn = winner.clone().unwrap_or_else(|| punished_side.clone());
            }

            let revealed_cards = describe_cards(&previous_play.cards);
            let roulette_result = if punished_alive {
                format!("{punished_display_name} survived chamber {shots_fired}/6.")
            } else {
                format!("{punished_display_name} pulled the lethal chamber and was eliminated.")
            };
            let outcome_tail = if let Some(winner_side) = winner.as_deref() {
                let winner_display = find_player(&next_state, winner_side)
                    .map(|player| player.display_name.clone())
                    .unwrap_or_else(|_| winner_side.to_string());
                format!(" {winner_display} wins the table.")
            } else if let Some(round_number) = next_round_number {
                let next_display = next_turn
                    .as_deref()
                    .and_then(|side| find_player(&next_state, side).ok())
                    .map(|player| player.display_name.clone())
                    .unwrap_or_else(|| "the next seat".to_string());
                format!(" Round {round_number} starts with {next_display}.")
            } else {
                String::new()
            };

            Ok(GameTransition {
                next_state: to_value(next_state)?,
                event_kind: hap_models::SessionEventKind::ActionTaken,
                event_summary: format!(
                    "{} called LIAR on {}. {} were revealed, so {} {}{}",
                    current_player.display_name,
                    previous_play.display_name,
                    revealed_cards,
                    punished_display_name,
                    roulette_result,
                    outcome_tail
                ),
                event_details: map_from_pairs([
                    (
                        "headline",
                        json!(format!(
                            "{} calls LIAR on {}",
                            current_player.display_name, previous_play.display_name
                        )),
                    ),
                    ("actionType", json!("call_liar")),
                    ("side", json!(state.turn)),
                    ("displayName", json!(current_player.display_name)),
                    ("tableCard", json!(state.table_card)),
                    ("callerSide", json!(state.turn)),
                    ("callerDisplayName", json!(current_player.display_name)),
                    ("accusedSide", json!(previous_play.side)),
                    ("accusedDisplayName", json!(previous_play.display_name)),
                    ("revealedCards", json!(previous_play.cards)),
                    ("liarFound", json!(liar_found)),
                    ("punishedSide", json!(punished_side)),
                    ("punishedDisplayName", json!(punished_display_name)),
                    ("shotsFired", json!(shots_fired)),
                    ("survived", json!(punished_alive)),
                    ("roundNumber", json!(state.round_number)),
                    ("nextRoundNumber", json!(next_round_number)),
                    ("nextTurn", json!(next_turn)),
                    ("winner", json!(winner)),
                ]),
            })
        }
    }
}

fn project_public_state(state: &GameState) -> PublicGameState {
    PublicGameState {
        kind: state.kind.clone(),
        turn: state.turn.clone(),
        status: state.status,
        winner: state.winner.clone(),
        round_number: state.round_number,
        phase: state.phase.clone(),
        table_card: state.table_card,
        previous_play: state.previous_play.as_ref().map(project_public_play),
        last_resolution: state.last_resolution.clone(),
        players: state
            .players
            .iter()
            .map(|player| PublicPlayerState {
                side: player.side.clone(),
                display_name: player.display_name.clone(),
                alive: player.alive,
                shots_fired: player.shots_fired,
                hand_size: player.hand.len(),
            })
            .collect(),
    }
}

fn project_seat_state(state: &GameState, side: &str) -> Result<SeatGameState> {
    let seat = find_player(state, side)?;
    let public = project_public_state(state);
    Ok(SeatGameState {
        kind: public.kind,
        turn: public.turn,
        status: public.status,
        winner: public.winner,
        round_number: public.round_number,
        phase: public.phase,
        table_card: public.table_card,
        previous_play: public.previous_play,
        last_resolution: public.last_resolution,
        players: public.players,
        self_side: seat.side.clone(),
        private_hand: seat.hand.clone(),
    })
}

fn project_public_play(play: &PlayRecord) -> PublicPlayRecord {
    PublicPlayRecord {
        side: play.side.clone(),
        display_name: play.display_name.clone(),
        declared_rank: play.declared_rank,
        card_count: play.card_count,
    }
}

fn start_next_round(mut state: GameState, starter_side: &str, round_number: u32) -> GameState {
    let mut liar_deck = build_liar_deck();
    liar_deck.shuffle(&mut rand::rng());

    state.table_card = *TABLE_CARD_OPTIONS
        .choose(&mut rand::rng())
        .unwrap_or(&CardRank::King);
    state.turn = starter_side.to_string();
    state.round_number = round_number;
    state.phase = "playing".to_string();
    state.previous_play = None;

    for player in &mut state.players {
        if !player.alive {
            player.hand.clear();
            continue;
        }

        player.hand = liar_deck.drain(..INITIAL_HAND_SIZE).collect();
    }

    state
}

fn build_liar_deck() -> Vec<CardRank> {
    let mut deck =
        Vec::with_capacity(LIAR_DECK_KINGS + LIAR_DECK_QUEENS + LIAR_DECK_ACES + LIAR_DECK_JOKERS);
    deck.extend(std::iter::repeat_n(CardRank::King, LIAR_DECK_KINGS));
    deck.extend(std::iter::repeat_n(CardRank::Queen, LIAR_DECK_QUEENS));
    deck.extend(std::iter::repeat_n(CardRank::Ace, LIAR_DECK_ACES));
    deck.extend(std::iter::repeat_n(CardRank::Joker, LIAR_DECK_JOKERS));
    deck
}

fn can_play_cards(state: &GameState, current_player: &PlayerState) -> bool {
    if current_player.hand.is_empty() {
        return false;
    }

    if state.previous_play.is_none() {
        return true;
    }

    count_alive_players_with_cards(state) > 1
}

fn count_alive_players(state: &GameState) -> usize {
    state.players.iter().filter(|player| player.alive).count()
}

fn count_alive_players_with_cards(state: &GameState) -> usize {
    state
        .players
        .iter()
        .filter(|player| player.alive && !player.hand.is_empty())
        .count()
}

fn remaining_alive_sides(state: &GameState) -> Vec<String> {
    state
        .players
        .iter()
        .filter(|player| player.alive)
        .map(|player| player.side.clone())
        .collect()
}

fn next_turn_after_play(state: &GameState, current_side: &str) -> Result<String> {
    if let Some(side) = next_alive_side_with_cards(state, current_side) {
        return Ok(side);
    }

    next_alive_side(state, current_side)
        .ok_or_else(|| anyhow!("No active Liars Bar seat is available to continue the round"))
}

fn next_alive_side(state: &GameState, current_side: &str) -> Option<String> {
    let current_index = state
        .players
        .iter()
        .position(|player| player.side == current_side)?;

    for offset in 1..=state.players.len() {
        let candidate = &state.players[(current_index + offset) % state.players.len()];
        if candidate.alive {
            return Some(candidate.side.clone());
        }
    }

    None
}

fn next_alive_side_with_cards(state: &GameState, current_side: &str) -> Option<String> {
    let current_index = state
        .players
        .iter()
        .position(|player| player.side == current_side)?;

    for offset in 1..=state.players.len() {
        let candidate = &state.players[(current_index + offset) % state.players.len()];
        if candidate.alive && !candidate.hand.is_empty() {
            return Some(candidate.side.clone());
        }
    }

    None
}

fn find_player<'a>(state: &'a GameState, side: &str) -> Result<&'a PlayerState> {
    state
        .players
        .iter()
        .find(|player| player.side == side)
        .ok_or_else(|| anyhow!("Unknown Liars Bar side: {side}"))
}

fn find_player_mut<'a>(state: &'a mut GameState, side: &str) -> Result<&'a mut PlayerState> {
    state
        .players
        .iter_mut()
        .find(|player| player.side == side)
        .ok_or_else(|| anyhow!("Unknown Liars Bar side: {side}"))
}

fn unique_play_combinations(hand: &[CardRank]) -> Vec<Vec<CardRank>> {
    let mut combinations = BTreeSet::new();

    for first in 0..hand.len() {
        let mut single = vec![hand[first]];
        single.sort();
        combinations.insert(single);

        for second in (first + 1)..hand.len() {
            let mut pair = vec![hand[first], hand[second]];
            pair.sort();
            combinations.insert(pair.clone());

            for third in (second + 1)..hand.len() {
                let mut trio = vec![hand[first], hand[second], hand[third]];
                trio.sort();
                combinations.insert(trio);
            }
        }
    }

    combinations
        .into_iter()
        .filter(|cards| !cards.is_empty() && cards.len() <= MAX_PLAY_COUNT)
        .collect()
}

fn normalize_cards(mut cards: Vec<CardRank>) -> Vec<CardRank> {
    cards.sort();
    cards
}

fn requested_cards(action: &LegalAction) -> Vec<CardRank> {
    match action {
        LegalAction::PlayCards { cards, .. } => cards.clone(),
        LegalAction::CallLiar { .. } => Vec::new(),
    }
}

fn remove_cards_from_hand(hand: &mut Vec<CardRank>, cards: &[CardRank]) -> Result<()> {
    for card in cards {
        let index = hand
            .iter()
            .position(|candidate| candidate == card)
            .ok_or_else(|| anyhow!("Selected card is not available in the current hand"))?;
        hand.remove(index);
    }
    Ok(())
}

fn describe_card_count(count: usize) -> String {
    match count {
        1 => "1 card".to_string(),
        _ => format!("{count} cards"),
    }
}

fn describe_cards(cards: &[CardRank]) -> String {
    cards
        .iter()
        .map(|card| card.label())
        .collect::<Vec<_>>()
        .join(" + ")
}

fn draw_lethal_chamber() -> u8 {
    rand::random_range(1..=REVOLVER_SIZE)
}

fn map_from_pairs<const N: usize>(pairs: [(&str, Value); N]) -> HashMap<String, Value> {
    pairs
        .into_iter()
        .filter(|(_, value)| !value.is_null())
        .map(|(key, value)| (key.to_string(), value))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_player(
        side: &str,
        display_name: &str,
        shots_fired: u8,
        lethal_chamber: u8,
        hand: Vec<CardRank>,
        alive: bool,
    ) -> PlayerState {
        PlayerState {
            side: side.to_string(),
            display_name: display_name.to_string(),
            alive,
            shots_fired,
            lethal_chamber,
            hand,
        }
    }

    fn build_state(
        turn: &str,
        table_card: CardRank,
        previous_play: Option<PlayRecord>,
        players: Vec<PlayerState>,
    ) -> GameState {
        GameState {
            kind: "liars-bar".to_string(),
            turn: turn.to_string(),
            status: SessionStatus::Active,
            winner: None,
            round_number: 2,
            phase: "playing".to_string(),
            table_card,
            previous_play,
            last_resolution: None,
            players,
        }
    }

    #[test]
    fn creates_initial_round_with_hidden_hands_and_table_card() {
        let state = create_initial_state();

        assert_eq!(state.kind, "liars-bar");
        assert_eq!(state.round_number, 1);
        assert!(TABLE_CARD_OPTIONS.contains(&state.table_card));
        assert!(state.players.iter().all(|player| player.alive));
        assert!(
            state
                .players
                .iter()
                .all(|player| player.hand.len() == INITIAL_HAND_SIZE)
        );
        assert!(state.players.iter().all(|player| player.shots_fired == 0));
    }

    #[test]
    fn lists_unique_card_plays_and_call_liar_after_a_previous_play() {
        let state = build_state(
            "seat-2",
            CardRank::King,
            Some(PlayRecord {
                side: "seat-1".to_string(),
                display_name: "P1".to_string(),
                declared_rank: CardRank::King,
                card_count: 2,
                cards: vec![CardRank::King, CardRank::Queen],
            }),
            vec![
                build_player(
                    "seat-1",
                    "P1",
                    0,
                    6,
                    vec![CardRank::King, CardRank::Queen],
                    true,
                ),
                build_player(
                    "seat-2",
                    "P2",
                    0,
                    6,
                    vec![CardRank::King, CardRank::King, CardRank::Joker],
                    true,
                ),
                build_player(
                    "seat-3",
                    "P3",
                    0,
                    6,
                    vec![CardRank::Ace, CardRank::Queen],
                    true,
                ),
                build_player("seat-4", "P4", 0, 6, vec![CardRank::Ace], true),
            ],
        );

        let actions = list_legal_actions(&state).unwrap();
        assert!(actions.contains(&LegalAction::CallLiar {
            label: "Call LIAR on the previous play".to_string(),
        }));
        assert!(actions.contains(&LegalAction::PlayCards {
            cards: vec![CardRank::King],
            label: "Play 1 card face-down as Kings".to_string(),
        }));
        assert!(actions.contains(&LegalAction::PlayCards {
            cards: vec![CardRank::King, CardRank::Joker],
            label: "Play 2 cards face-down as Kings".to_string(),
        }));
    }

    #[test]
    fn forces_a_liar_call_when_only_one_hand_remains() {
        let state = build_state(
            "seat-3",
            CardRank::Ace,
            Some(PlayRecord {
                side: "seat-2".to_string(),
                display_name: "P2".to_string(),
                declared_rank: CardRank::Ace,
                card_count: 1,
                cards: vec![CardRank::Queen],
            }),
            vec![
                build_player("seat-1", "P1", 0, 6, Vec::new(), true),
                build_player("seat-2", "P2", 0, 6, Vec::new(), true),
                build_player("seat-3", "P3", 0, 6, vec![CardRank::Ace], true),
                build_player("seat-4", "P4", 0, 6, Vec::new(), true),
            ],
        );

        let actions = list_legal_actions(&state).unwrap();
        assert_eq!(
            actions,
            vec![LegalAction::CallLiar {
                label: "Call LIAR on the previous play".to_string(),
            }]
        );
    }

    #[test]
    fn resolves_a_successful_liar_call_and_starts_the_next_round() {
        let state = build_state(
            "seat-2",
            CardRank::King,
            Some(PlayRecord {
                side: "seat-1".to_string(),
                display_name: "P1".to_string(),
                declared_rank: CardRank::King,
                card_count: 2,
                cards: vec![CardRank::King, CardRank::Queen],
            }),
            vec![
                build_player("seat-1", "P1", 0, 6, vec![CardRank::Ace], true),
                build_player("seat-2", "P2", 0, 6, vec![CardRank::King], true),
                build_player("seat-3", "P3", 0, 6, vec![CardRank::Queen], true),
                build_player("seat-4", "P4", 0, 6, vec![CardRank::Ace], true),
            ],
        );

        let transition = play_move(&state, ActionInput::CallLiar).unwrap();
        let next_state: GameState = from_value(transition.next_state).unwrap();

        assert_eq!(
            transition.event_kind,
            hap_models::SessionEventKind::ActionTaken
        );
        assert_eq!(find_player(&next_state, "seat-1").unwrap().shots_fired, 1);
        assert_eq!(next_state.round_number, 3);
        assert_eq!(next_state.turn, "seat-1");
        assert!(TABLE_CARD_OPTIONS.contains(&next_state.table_card));
        assert!(next_state.previous_play.is_none());
        assert!(next_state.last_resolution.as_ref().unwrap().liar_found);
    }

    #[test]
    fn eliminates_a_player_when_the_lethal_chamber_is_reached() {
        let state = build_state(
            "seat-2",
            CardRank::Queen,
            Some(PlayRecord {
                side: "seat-1".to_string(),
                display_name: "P1".to_string(),
                declared_rank: CardRank::Queen,
                card_count: 1,
                cards: vec![CardRank::King],
            }),
            vec![
                build_player("seat-1", "P1", 0, 1, vec![CardRank::Ace], true),
                build_player("seat-2", "P2", 0, 6, vec![CardRank::Queen], true),
                build_player("seat-3", "P3", 0, 6, Vec::new(), false),
                build_player("seat-4", "P4", 0, 6, Vec::new(), false),
            ],
        );

        let transition = play_move(&state, ActionInput::CallLiar).unwrap();
        let next_state: GameState = from_value(transition.next_state).unwrap();

        assert_eq!(next_state.status, SessionStatus::Finished);
        assert_eq!(next_state.winner.as_deref(), Some("seat-2"));
        assert!(!find_player(&next_state, "seat-1").unwrap().alive);
        assert!(!next_state.last_resolution.as_ref().unwrap().survived);
    }

    #[test]
    fn projects_public_and_private_views_without_leaking_hands() {
        let state = create_initial_state();
        let public_state = project_public_state(&state);
        let seat_state = project_seat_state(&state, "seat-3").unwrap();

        assert_eq!(public_state.players[0].hand_size, INITIAL_HAND_SIZE);
        let serialized_public = to_value(public_state).unwrap();
        assert!(serialized_public["players"][0].get("hand").is_none());

        assert_eq!(seat_state.self_side, "seat-3");
        assert_eq!(seat_state.private_hand.len(), INITIAL_HAND_SIZE);
        assert!(seat_state.previous_play.is_none());
    }
}
