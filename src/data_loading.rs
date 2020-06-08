use crate::components::{Card, Enemy};
use crate::textures::Textures;

use raylib::prelude::*;
use ron::de::from_reader;
use std::fs::File;
use std::collections::BTreeMap;


use crate::{EnemyID, CardID, CardDB, EnemiesDB};

const C_WIDTH: i32 = 60;
const C_HEIGHT: i32 = 80;

pub fn get_enemies_from_file() -> EnemiesDB {
    let f = File::open("assets/enemies.ron").expect("Failed opening file");
    let loaded_enemies: Vec<Enemy> = match from_reader(f) {
        Ok(x) => x,
        Err(e) => {
            println!("Failed to load enemies: {}", e);

            std::process::exit(1);
        }
    };

    let mut enemies: EnemiesDB = BTreeMap::new();
    let mut id: EnemyID = 0;
    for enemy in loaded_enemies {
        enemies.insert(id, enemy);
        id+= 1;
    }
    
    enemies
}

pub fn get_cards_from_file() -> CardDB {
    let f = File::open("assets/cards.ron").expect("Failed opening file");
    let loaded_cards: Vec<Card> = match from_reader(f) {
        Ok(x) => x,
        Err(e) => {
            println!("Failed to load cards: {}", e);

            std::process::exit(1);
        }
    };

    let mut cards: CardDB = BTreeMap::new();
    let mut id: EnemyID = 0;

    for card in loaded_cards {
        cards.insert(id, card);
        id+= 1;
    }

    cards
}


pub fn generate_card_textures(rl: &mut RaylibHandle, thread: &RaylibThread, textures: &mut Textures, cards: &CardDB) {
    for (id, card) in cards {
        let template_name = format!("id_{}_card", id);
        if !textures.contains(&template_name) {
            let base_image_path = "assets/enemy-front.png";
            let mut i = Image::load_image(&base_image_path).expect("could not load image");
            i.image_resize_nn(i.width * 2, i.height * 2);
            i.image_draw_text_ex(
                Vector2 { x: 5., y: 2. },
                &rl.get_font_default(),
                &card.name,
                10.,
                1.,
                Color::WHITE,
            );
            i.image_draw_text_ex(
                Vector2 { x: 5., y: 15. },
                &rl.get_font_default(),
                &format!("attack: {}", &card.value.to_string()),
                10.,
                1.,
                Color::WHITE,
            );
            use std::str;
            let string = format!("effect: {}", &card.effect_description);
            let descriptions: Vec<&str> = string.split('\n').collect();
            for (index, d) in descriptions.iter().enumerate() {
                i.image_draw_text_ex(
                    Vector2 {
                        x: 5.,
                        y: 30. + (index * 10) as f32,
                    },
                    &rl.get_font_default(),
                    &d,
                    10.,
                    1.,
                    Color::WHITE,
                );
            }
            let texture = rl
                .load_texture_from_image(&thread, &i)
                .expect("could not load texture from image");
            textures.add(&card.name, texture);
        }
    }
}


pub fn generate_enemy_textures(rl: &mut RaylibHandle, thread: &RaylibThread, textures: &mut Textures, enemies: &EnemiesDB) {
    for (id, enemy) in enemies {
        let template_name = format!("id_{}_enemy", id);
        if !textures.contains(&template_name) {
            let base_image_path = "assets/enemy-front.png";
            let mut i = Image::load_image(&base_image_path).expect("could not load image");
            i.image_draw_text_ex(
                Vector2 { x: 5., y: 2. },
                &rl.get_font_default(),
                &enemy.name,
                10.,
                1.,
                Color::WHITE,
            );
            i.image_draw_text_ex(
                Vector2 {
                    x: 5.,
                    y: C_HEIGHT as f32 - 13.,
                },
                &rl.get_font_default(),
                &format!("attack: {}", enemy.attack),
                10.,
                1.,
                Color::WHITE,
            );
            let t = rl.load_texture_from_image(&thread, &i).unwrap();
            textures.add(&template_name, t);
        }
    }
}