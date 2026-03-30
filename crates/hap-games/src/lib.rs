mod chess;
mod connect_four;
mod gomoku;
mod liars_bar;
mod othello;
mod xiangqi;

use anyhow::{Result, anyhow};
use hap_models::{GameCatalogItem, SessionEventKind};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateAudience<'a> {
    Public,
    Seat(&'a str),
}

#[derive(Debug, Clone)]
pub struct GameTransition {
    pub next_state: Value,
    pub event_kind: SessionEventKind,
    pub event_summary: String,
    pub event_details: HashMap<String, Value>,
}

pub trait GameAdapter: Send + Sync {
    fn game(&self) -> &GameCatalogItem;
    fn create_initial_state(&self) -> Result<Value>;
    fn normalize_state(&self, state: Value) -> Result<Value>;
    fn list_legal_moves(&self, state: &Value, query: Option<&Value>) -> Result<Vec<Value>>;
    fn play_move(&self, state: &Value, input: &Value) -> Result<GameTransition>;
    fn project_state(&self, state: &Value, _audience: StateAudience<'_>) -> Result<Value> {
        Ok(state.clone())
    }
}

pub fn standard_move_transition(next_state: Value) -> GameTransition {
    GameTransition {
        next_state,
        event_kind: SessionEventKind::MovePlayed,
        event_summary: String::new(),
        event_details: HashMap::new(),
    }
}

pub fn list_game_catalog() -> Vec<GameCatalogItem> {
    adapters()
        .iter()
        .map(|adapter| adapter.game().clone())
        .collect()
}

pub fn get_game_adapter(game_id: &str) -> Result<&'static dyn GameAdapter> {
    adapters()
        .into_iter()
        .find(|adapter| adapter.game().id == game_id)
        .ok_or_else(|| anyhow!("Unsupported game: {game_id}"))
}

fn adapters() -> Vec<&'static dyn GameAdapter> {
    vec![
        &xiangqi::ADAPTER,
        &chess::ADAPTER,
        &liars_bar::ADAPTER,
        &gomoku::ADAPTER,
        &connect_four::ADAPTER,
        &othello::ADAPTER,
    ]
}
