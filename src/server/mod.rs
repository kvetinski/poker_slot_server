use crate::game;
use crate::models::{
    DiscardRequest, DiscardResponse, LoginResponse, RevealRequest, RevealResponse, RoundStatus,
    SignInRequest, SignUpRequest, StartRequest, StartResponse, StatusResponse,
};
use crate::store::SharedStore;
use axum::{
    extract::Extension,
    extract::Path,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;

pub fn router(store: SharedStore) -> Router {
    Router::new()
        .route("/", get(root_health))
        .route("/api/signup", post(signup_handler))
        .route("/api/signin", post(signin_handler))
        .route("/api/status/{user_id}", get(status_handler))
        .route("/api/start", post(start_handler))
        .route("/api/discard", post(discard_handler))
        .route("/api/reveal", post(reveal_handler))
        .layer(Extension(store))
}

/// GET /
async fn root_health() -> Json<serde_json::Value> {
    Json(json!({"status":"ok","service":"poker-server","version":"0.1"}))
}

/// POST /api/signup
async fn signup_handler(
    Extension(store): Extension<SharedStore>,
    Json(req): Json<SignUpRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, String)> {
    let user = store
        .create_user_if_unique(&req.name, &req.password)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    Ok(Json(LoginResponse {
        id: user.id,
        name: user.name,
        wallet: user.wallet,
    }))
}

/// POST /api/signin
async fn signin_handler(
    Extension(store): Extension<SharedStore>,
    Json(req): Json<SignInRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, String)> {
    let user = store
        .login_user_if_exists(&req.name, &req.password)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    Ok(Json(LoginResponse {
        id: user.id,
        name: user.name,
        wallet: user.wallet,
    }))
}

/// POST /api/start
async fn start_handler(
    Extension(store): Extension<SharedStore>,
    Json(req): Json<StartRequest>,
) -> Result<Json<StartResponse>, (StatusCode, String)> {
    // ensure user exists
    let user = store
        .get_user(&req.user_id)
        .await
        .ok_or((StatusCode::BAD_REQUEST, "user not found".to_string()))?;

    if req.ante <= 0 {
        return Err((StatusCode::BAD_REQUEST, "invalid ante".to_string()));
    }
    if req.ante > user.wallet {
        return Err((StatusCode::BAD_REQUEST, "insufficient wallet".to_string()));
    }

    // check win_pool capacity: max_multiplier = 50
    let pools = store.get_pools().await;
    let max_possible = req.ante * 50;
    if pools.win_pool < max_possible {
        return Err((
            StatusCode::BAD_REQUEST,
            format!(
                "win pool too small, max ante allowed {}",
                pools.win_pool / 50
            ),
        ));
    }

    // deduct wallet (short critical section)
    store
        .update_user_wallet(&req.user_id, user.wallet - req.ante)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "wallet update failed".to_string(),
            )
        })?;

    // deal 5 cards (pure)
    let mut deck = game::new_deck();
    let hand = game::deal_hand(&mut deck, 5);

    // create round
    let round_id = store
        .create_round(req.user_id.clone(), req.ante, hand.clone())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let current_user = store.get_user(&req.user_id).await.unwrap();
    let pools_after = store.get_pools().await;
    Ok(Json(StartResponse {
        round_id,
        cards: hand,
        wallet: current_user.wallet,
        win_pool: pools_after.win_pool,
    }))
}

/// POST /api/discard
async fn discard_handler(
    Extension(store): Extension<SharedStore>,
    Json(req): Json<DiscardRequest>,
) -> Result<Json<DiscardResponse>, (StatusCode, String)> {
    // basic validations
    let round = store
        .get_round(&req.round_id)
        .await
        .ok_or((StatusCode::BAD_REQUEST, "round not found".to_string()))?;
    if round.user_id != req.user_id {
        return Err((StatusCode::BAD_REQUEST, "user mismatch".to_string()));
    }
    if round.status != RoundStatus::Active {
        return Err((StatusCode::BAD_REQUEST, "round not active".to_string()));
    }

    // cost: 50% ante per card
    let discard_count = req.discard_indices.len();
    let cost = ((round.ante as f64) * 0.5 * (discard_count as f64)) as i64;

    let user = store
        .get_user(&req.user_id)
        .await
        .ok_or((StatusCode::BAD_REQUEST, "user not found".to_string()))?;
    if user.wallet < cost {
        return Err((
            StatusCode::BAD_REQUEST,
            "insufficient wallet for discard".to_string(),
        ));
    }

    // deduct
    store
        .update_user_wallet(&req.user_id, user.wallet - cost)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "wallet update failed".to_string(),
            )
        })?;

    // replace cards
    let mut deck = game::new_deck();
    // remove existing cards from deck (naive approach - just shuffle and deal new ones)
    let mut newcards = round.cards.clone();
    // replace indices with new cards
    let dealt = game::deal_hand(&mut deck, discard_count);
    for (i, idx) in req.discard_indices.iter().enumerate() {
        if *idx < newcards.len() {
            newcards[*idx] = dealt[i].clone();
        }
    }

    store
        .update_round_cards(&req.round_id, newcards.clone())
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed update round".to_string(),
            )
        })?;

    // compute total bet (ante + raise) - here raise 0
    let total_bet = round.ante;

    let new_user = store.get_user(&req.user_id).await.unwrap();
    Ok(Json(DiscardResponse {
        cards: newcards,
        wallet: new_user.wallet,
        total_bet,
    }))
}

/// POST /api/reveal
async fn reveal_handler(
    Extension(store): Extension<SharedStore>,
    Json(req): Json<RevealRequest>,
) -> Result<Json<RevealResponse>, (StatusCode, String)> {
    let round = store
        .get_round(&req.round_id)
        .await
        .ok_or((StatusCode::BAD_REQUEST, "round not found".to_string()))?;
    if round.user_id != req.user_id {
        return Err((StatusCode::BAD_REQUEST, "user mismatch".to_string()));
    }
    if round.status != RoundStatus::Active {
        return Err((StatusCode::BAD_REQUEST, "round not active".to_string()));
    }

    let total_bet = round.ante;
    let hr = game::evaluate_hand(&round.cards);
    let mult = game::payout_multiplier(&hr);
    let payout = (total_bet as i64) * (mult as i64);

    if mult == 0 {
        // losing: split 25% house, 75% win_pool
        let house = (total_bet * 25) / 100;
        let win = total_bet - house;
        store.add_to_pools(win, house).await;
        store
            .set_round_status(&req.round_id, RoundStatus::Revealed)
            .await
            .ok();
        let user = store.get_user(&req.user_id).await.unwrap();
        let pools = store.get_pools().await;
        return Ok(Json(RevealResponse {
            wallet: user.wallet,
            win_pool: pools.win_pool,
            house_profit: pools.house_profit,
            hand_rank: format!("{:?}", hr),
            multiplier: mult,
            payout: 0,
        }));
    }

    // winning hand: check pool
    let pools = store.get_pools().await;
    if pools.win_pool < payout {
        // refund total_bet to user
        let user = store.get_user(&req.user_id).await.unwrap();
        let _ = store
            .update_user_wallet(&req.user_id, user.wallet + total_bet)
            .await;
        store
            .set_round_status(&req.round_id, RoundStatus::Revealed)
            .await
            .ok();
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "win_pool short, refunded".to_string(),
        ));
    }

    // pay out
    store
        .sub_from_win_pool(payout)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let user = store.get_user(&req.user_id).await.unwrap();
    let _ = store
        .update_user_wallet(&req.user_id, user.wallet + payout)
        .await;
    store
        .set_round_status(&req.round_id, RoundStatus::Revealed)
        .await
        .ok();
    let pools_after = store.get_pools().await;
    let new_user = store.get_user(&req.user_id).await.unwrap();

    Ok(Json(RevealResponse {
        wallet: new_user.wallet,
        win_pool: pools_after.win_pool,
        house_profit: pools_after.house_profit,
        hand_rank: format!("{:?}", hr),
        multiplier: mult,
        payout,
    }))
}

/// GET /api/status?user_id=...
async fn status_handler(
    Extension(store): Extension<SharedStore>,
    Path(user_id): Path<String>,
) -> Result<Json<StatusResponse>, (StatusCode, String)> {
    let user = store
        .get_user(&user_id)
        .await
        .ok_or((StatusCode::BAD_REQUEST, "user not found".to_string()))?;

    // optional redundant check removed because query already carries the user_id:
    // if user.id != req.user_id { ... }

    let pools = store.get_pools().await;

    Ok(Json(StatusResponse {
        wallet: user.wallet,
        win_pool: pools.win_pool,
        house_profit: pools.house_profit,
    }))
}
