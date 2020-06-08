
use crate::{CardID, CardDB};
use crate::components::Card;
use rand::{thread_rng, Rng};


pub fn generate_intial_deck(cards_db: &CardDB) -> Vec<CardID> {
    let mut deck = Vec::new();

    let mut rng = rand::thread_rng();

    for _ in 0..10 {
        let index = rng.gen_range(0, cards_db.len());
        let card = cards_db.iter().nth(index).unwrap();
        deck.push(*card.0);
    }
    deck
}