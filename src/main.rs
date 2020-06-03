use ron::de::from_reader;
use std::fs::File;

use raylib::prelude::*;
use specs::prelude::*;

use rand::Rng;

use crate::components::{
    Button, Card, Enemy, HealthBar, Mousehandler, Player, Position, Sprite, Rectangle, Active
};
use crate::systems::{BattleState, CardSelector, DamageQueue, DrawSys, EnemyAttacker, ButtonHandler};
use crate::textures::Textures;

pub mod components;
pub mod systems;
pub mod textures;

pub const COLOUR: Color = Color::new(34, 32, 52, 255);
const WIDTH: i32 = 1000;
const HEIGHT: i32 = 800;

const C_WIDTH: i32 = 60;
const C_HEIGHT: i32 = 80;

fn window_should_close(world: &World) -> bool {
    let rl = world.read_resource::<RaylibHandle>();
    rl.window_should_close()
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(WIDTH, HEIGHT)
        .title("Hello, World")
        .build();
    rl.set_target_fps(60);
    rl.set_mouse_scale(1., 1.);
    let mut textures = Textures::from_paths(vec!["card-back", "mouse-grab"], &mut rl, &thread);
    let mut world = World::new();
    world.register::<Position>();
    world.register::<Rectangle>();
    world.register::<Sprite>();
    world.register::<Card>();
    world.register::<Enemy>();
    world.register::<Mousehandler>();
    world.register::<HealthBar>();
    world.register::<Player>();
    world.register::<Button>();
    world.register::<Active>();
    world.insert(BattleState::default());
    let mut rng = rand::thread_rng();

    // let data = fs::read_to_string("assets/cards.ron").expect("Unable to read file");
    let f = File::open("assets/enemies.ron").expect("Failed opening file");
    let enemies: Vec<Enemy> = match from_reader(f) {
        Ok(x) => x,
        Err(e) => {
            println!("Failed to load config: {}", e);

            std::process::exit(1);
        }
    };

    for x in 0..4 {
        let i = rng.gen_range(0, enemies.len());
        let enemy = enemies.get(i).expect("couldnt get card");
        let template_name = format!("h:{0}_a:{1}", enemy.health, enemy.attack);
        if !textures.contains(&template_name) {
            // generate template
            let location = "assets/enemy-front.png";
            let mut i = Image::load_image(&location).expect("could not load image");
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
        world
            .create_entity()
            .with(Rectangle {
                width: C_WIDTH as f32 * 3.,
                height: C_HEIGHT as f32 * 3.
            })
            .with(Position {
                x: (100 + x * ((C_WIDTH + 10) * 3)) as f32,
                y: (C_HEIGHT * 3) as f32,
            })
            .with(Sprite {
                scale: 3.,
                texture_path: "card-back".to_string(),
            })
            .with(Active::default())
            .with(enemy.clone())
            .build();
    }
    // let data = fs::read_to_string("assets/cards.ron").expect("Unable to read file");
    let f = File::open("assets/cards.ron").expect("Failed opening file");
    let cards: Vec<Card> = match from_reader(f) {
        Ok(x) => x,
        Err(e) => {
            println!("Failed to load config: {}", e);

            std::process::exit(1);
        }
    };

    for card in &cards {
        let location = "assets/card.png";
        let mut i = Image::load_image(&location).expect("could not load image");
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

    for index in 0..3 {
        let mut rng = rand::thread_rng();
        let i = rng.gen_range(0, cards.len());
        let card = cards.get(i).expect("couldnt get card");
        world
            .create_entity()
            .with(Position {
                x: (100 + index * ((C_WIDTH + 10) * 3)) as f32,
                y: (420 + ((C_HEIGHT + 10) * 2)) as f32,
            })
            .with(Rectangle {
                width: C_WIDTH as f32 * 3.,
                height: C_HEIGHT as f32 * 3.,
            })
            .with(Sprite {
                scale: 1.5,
                texture_path: card.name.to_string(),
            })
            .with(Active::default())
            .with(card.clone())
            .build();
    }

    world
        .create_entity()
        .with(Mousehandler)
        .with(Position { x: 0.0, y: 0.0 })
        .with(Rectangle { width: 25., height: 25. })
        .with(Active(false))
        .with(Sprite {
            scale: 1.,
            texture_path: "mouse-grab".to_string(),
        })
        .build();

    world
        .create_entity()
        .with(HealthBar::new(20))
        .with(Position { x: 50., y: 100. })
        .with(Rectangle {
            width: 200.,
            height: 50.,
        })
        .with(Player::default())
        .build();

    world
        .create_entity()
        .with(Position {
            x: (WIDTH - 200) as f32,
            y: (HEIGHT - 100) as f32,
        })
        .with(Rectangle {
            width: 150.,
            height: 50.,
        })
        .with(Button::new(
            "End Turn",
            "end_turn"
        ))
        .build();

    world.insert(rl);
    let mut dispatcher = specs::DispatcherBuilder::new()
        .with_thread_local(DrawSys { thread, textures })
        .with(
            CardSelector {
                selected: None,
                dragging: None,
            },
            "card_selector",
            &[],
        )
        .with(DamageQueue, "damage_queue", &["card_selector"])
        .with(EnemyAttacker, "enemy_attacker", &[])
        .with(ButtonHandler, "button_handler", &[])
        .build();
    dispatcher.setup(&mut world);

    loop {
        dispatcher.dispatch(&world);
        {
            if window_should_close(&world) {
                break;
            }
        }
        world.maintain();
    }
}
