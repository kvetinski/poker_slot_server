use serde::{Deserialize, Serialize};

/// Card, Suit, HandRank - simple and serializable
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Card {
    pub rank: u8, // 2..=14
    pub suit: Suit,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Suit {
    Hearts,
    Diamonds,
    Clubs,
    Spades,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HandRank {
    HighCard,
    Pair(u8),
    TwoPair,
    Trips,
    Straight,
    Flush,
    FullHouse,
    FourKind,
    StraightFlush,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub name: String,
    pub password: String,
    pub wallet: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RoundStatus {
    Active,
    Discarded,
    Revealed,
    Folded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Round {
    pub id: String,
    pub user_id: String,
    pub cards: Vec<Card>,
    pub ante: i64,
    pub status: RoundStatus,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Pools {
    pub win_pool: i64,
    pub house_profit: i64,
}

impl Default for Pools {
    fn default() -> Self {
        Pools {
            win_pool: 0,
            house_profit: 0,
        }
    }
}

// Request / Response DTOs

#[derive(Debug, Deserialize)]
pub struct SignInRequest {
    pub name: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct SignUpRequest {
    pub name: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub id: String,
    pub name: String,
    pub wallet: i64,
}

#[derive(Debug, Deserialize)]
pub struct StartRequest {
    pub user_id: String,
    pub ante: i64,
}

#[derive(Debug, Serialize)]
pub struct StartResponse {
    pub round_id: String,
    pub cards: Vec<Card>,
    pub wallet: i64,
    pub win_pool: i64,
}

#[derive(Debug, Deserialize)]
pub struct DiscardRequest {
    pub user_id: String,
    pub round_id: String,
    pub discard_indices: Vec<usize>,
}

#[derive(Debug, Serialize)]
pub struct DiscardResponse {
    pub cards: Vec<Card>,
    pub wallet: i64,
    pub total_bet: i64,
}

#[derive(Debug, Deserialize)]
pub struct RevealRequest {
    pub user_id: String,
    pub round_id: String,
}

#[derive(Debug, Serialize)]
pub struct RevealResponse {
    pub wallet: i64,
    pub win_pool: i64,
    pub house_profit: i64,
    pub hand_rank: String,
    pub multiplier: u32,
    pub payout: i64,
}

#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub wallet: i64,
    pub win_pool: i64,
    pub house_profit: i64,
}

#[derive(Debug, Deserialize)]
pub struct StatusRequest {
    pub user_id: String,
}
