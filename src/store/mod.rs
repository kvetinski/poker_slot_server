use crate::models::{Card, Pools, Round, RoundStatus, User};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub type SharedStore = Arc<dyn Store + Send + Sync>;

#[derive(Clone)]
pub struct InMem {
    inner: Arc<Mutex<InMemState>>,
}

#[derive(Default)]
struct InMemState {
    users: HashMap<String, User>,
    rounds: HashMap<String, Round>,
    pools: Pools,
}

impl InMem {
    pub fn new_demo() -> Self {
        let mut state = InMemState::default();
        let u = User {
            id: "user1".to_string(),
            name: "user1".to_string(),
            password: "pass1".to_string(),
            wallet: 1000,
        };
        state.users.insert(u.id.clone(), u);
        state.pools.win_pool = 50_000;
        state.pools.house_profit = 0;
        InMem {
            inner: Arc::new(Mutex::new(state)),
        }
    }

    pub fn into_shared(self) -> SharedStore {
        Arc::new(self)
    }
}

#[async_trait::async_trait]
pub trait Store {
    async fn create_user_if_unique(&self, name: &str, password: &str) -> Result<User, String>;
    async fn login_user_if_exists(&self, name: &str, password: &str) -> Result<User, String>;
    async fn get_user(&self, user_id: &str) -> Option<User>;
    async fn update_user_wallet(&self, user_id: &str, new_wallet: i64) -> Result<(), String>;
    async fn create_round(
        &self,
        user_id: String,
        ante: i64,
        cards: Vec<Card>,
    ) -> Result<String, String>;
    async fn get_round(&self, round_id: &str) -> Option<Round>;
    async fn update_round_cards(&self, round_id: &str, cards: Vec<Card>) -> Result<(), String>;
    async fn set_round_status(&self, round_id: &str, status: RoundStatus) -> Result<(), String>;
    async fn get_pools(&self) -> Pools;
    async fn add_to_pools(&self, win: i64, house: i64);
    async fn sub_from_win_pool(&self, amount: i64) -> Result<(), String>;
}

/// In-memory implementation
#[async_trait::async_trait]
impl Store for InMem {
    async fn create_user_if_unique(&self, name: &str, password: &str) -> Result<User, String> {
        let mut s = self.inner.lock();
        if s.users.values().any(|u| u.name == name) {
            return Err("name already exists".into());
        }

        let id = Uuid::new_v4().to_string();
        let user = User {
            id: id.clone(),
            name: name.to_string(),
            password: password.to_string(),
            wallet: 1000,
        };

        s.users.insert(id.clone(), user.clone());
        Ok(user)
    }

    async fn login_user_if_exists(&self, name: &str, password: &str) -> Result<User, String> {
        let mut s = self.inner.lock();
        if !s
            .users
            .values()
            .any(|u| u.name == name && u.password == password)
        {
            return Err("invalid credentials".into());
        }

        let id = Uuid::new_v4().to_string();
        let user = User {
            id: id.clone(),
            name: name.to_string(),
            password: password.to_string(),
            wallet: 1000,
        };

        s.users.insert(id.clone(), user.clone());
        Ok(user)
    }

    async fn get_user(&self, user_id: &str) -> Option<User> {
        let s = self.inner.lock();
        s.users.get(user_id).cloned()
    }

    async fn update_user_wallet(&self, user_id: &str, new_wallet: i64) -> Result<(), String> {
        let mut s = self.inner.lock();
        match s.users.get_mut(user_id) {
            Some(u) => {
                u.wallet = new_wallet;
                Ok(())
            }
            None => Err("user not found".into()),
        }
    }

    async fn create_round(
        &self,
        user_id: String,
        ante: i64,
        cards: Vec<Card>,
    ) -> Result<String, String> {
        let mut s = self.inner.lock();
        let id = Uuid::new_v4().to_string();
        let r = Round {
            id: id.clone(),
            user_id,
            cards,
            ante,
            status: crate::models::RoundStatus::Active,
        };
        s.rounds.insert(id.clone(), r);
        Ok(id)
    }

    async fn get_round(&self, round_id: &str) -> Option<Round> {
        let s = self.inner.lock();
        s.rounds.get(round_id).cloned()
    }

    async fn update_round_cards(&self, round_id: &str, cards: Vec<Card>) -> Result<(), String> {
        let mut s = self.inner.lock();
        match s.rounds.get_mut(round_id) {
            Some(r) => {
                r.cards = cards;
                Ok(())
            }
            None => Err("round not found".into()),
        }
    }

    async fn set_round_status(&self, round_id: &str, status: RoundStatus) -> Result<(), String> {
        let mut s = self.inner.lock();
        match s.rounds.get_mut(round_id) {
            Some(r) => {
                r.status = status;
                Ok(())
            }
            None => Err("round not found".into()),
        }
    }

    async fn get_pools(&self) -> Pools {
        let s = self.inner.lock();
        s.pools.clone()
    }

    async fn add_to_pools(&self, win: i64, house: i64) {
        let mut s = self.inner.lock();
        s.pools.win_pool += win;
        s.pools.house_profit += house;
    }

    async fn sub_from_win_pool(&self, amount: i64) -> Result<(), String> {
        let mut s = self.inner.lock();
        if s.pools.win_pool < amount {
            return Err("win_pool short".into());
        }
        s.pools.win_pool -= amount;
        Ok(())
    }
}
