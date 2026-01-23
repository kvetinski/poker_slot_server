use crate::models::{Card, HandRank, Suit};
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashMap;

pub fn new_deck() -> Vec<Card> {
    let mut deck = Vec::with_capacity(52);
    let suits = [Suit::Hearts, Suit::Diamonds, Suit::Clubs, Suit::Spades];
    for &s in &suits {
        for r in 2..=14 {
            deck.push(Card { rank: r, suit: s });
        }
    }
    deck
}

pub fn deal_hand(deck: &mut Vec<Card>, n: usize) -> Vec<Card> {
    deck.shuffle(&mut thread_rng());
    deck.drain(0..n).collect()
}

pub fn evaluate_hand(cards: &[Card]) -> HandRank {
    // same logic as before but tidy
    let mut ranks: Vec<u8> = cards.iter().map(|c| c.rank).collect();
    ranks.sort_unstable();
    let mut counts: HashMap<u8, usize> = HashMap::new();
    for &r in &ranks {
        *counts.entry(r).or_insert(0) += 1;
    }

    let is_flush = cards.iter().all(|c| c.suit == cards[0].suit);
    let is_straight = {
        let mut uniq: Vec<u8> = counts.keys().cloned().collect();
        uniq.sort_unstable();
        uniq.len() == 5 && (uniq[4] - uniq[0] == 4 || uniq == vec![2, 3, 4, 5, 14])
    };

    let mut freq: Vec<usize> = counts.values().cloned().collect();
    freq.sort_unstable_by(|a, b| b.cmp(a));

    if is_straight && is_flush {
        return HandRank::StraightFlush;
    }
    if freq.as_slice() == [4, 1] {
        return HandRank::FourKind;
    }
    if freq.as_slice() == [3, 2] {
        return HandRank::FullHouse;
    }
    if is_flush {
        return HandRank::Flush;
    }
    if is_straight {
        return HandRank::Straight;
    }
    if freq.as_slice() == [3, 1, 1] {
        return HandRank::Trips;
    }
    if freq.as_slice() == [2, 2, 1] {
        return HandRank::TwoPair;
    }
    if freq.as_slice() == [2, 1, 1, 1] {
        for (&rank, &count) in &counts {
            if count == 2 {
                return HandRank::Pair(rank);
            }
        }
    }
    HandRank::HighCard
}

pub fn payout_multiplier(hr: &HandRank) -> u32 {
    match hr {
        HandRank::HighCard => 0,
        HandRank::Pair(rank) => {
            if *rank >= 11 {
                1
            } else {
                0
            }
        }
        HandRank::TwoPair => 2,
        HandRank::Trips => 3,
        HandRank::Straight => 5,
        HandRank::Flush => 6,
        HandRank::FullHouse => 9,
        HandRank::FourKind => 25,
        HandRank::StraightFlush => 50,
    }
}
