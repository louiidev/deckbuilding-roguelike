use std::collections::BTreeMap;

use raylib::prelude::*;
use specs::prelude::*;

use rand::Rng;

use crate::components::{
    Active, Button, Card, Enemy, HealthBar, Mousehandler, Player, Position, Rectangle, Sprite,
};
use crate::data_loading::{
    generate_card_textures, generate_enemy_textures, get_cards_from_file, get_enemies_from_file,
};
use crate::systems::{
    BattleState, ButtonHandler, CardSelector, DamageQueue, DrawSys, EnemyAttacker, MainMenuDraw, DrawCardSys
};
use crate::textures::Textures;

pub mod battle_scene;
pub mod components;
pub mod data_loading;
pub mod systems;
pub mod textures;
pub mod cards;

pub const COLOUR: Color = Color::new(34, 32, 52, 255);
pub const WIDTH: i32 = 1000;
pub const HEIGHT: i32 = 800;

const C_WIDTH: i32 = 60;
const C_HEIGHT: i32 = 80;

pub type CardID = u64;
pub type EnemyID = u64;
pub type CardDB = BTreeMap<CardID, Card>;
pub type EnemiesDB = BTreeMap<EnemyID, Enemy>;

#[derive(Copy, Clone)]
enum GameScenes {
    MainMenu,
    Battle,
}

pub struct BattleFlags {
    generated_deck: bool,

}

pub struct GameState {
    current_scene: GameScenes
}

fn window_should_close(world: &World) -> bool {
    let rl = world.read_resource::<RaylibHandle>();
    rl.window_should_close()
}

fn get_game_scene(world: &World) -> GameScenes {
    world.read_resource::<GameState>().current_scene
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(WIDTH, HEIGHT)
        .title("Hello, World")
        .build();
    rl.set_target_fps(60);
    rl.set_mouse_scale(1., 1.);

    let mut battle_textures = Textures::from_paths(vec!["card-back", "mouse-grab"], &mut rl, &thread);
    let menu_textures = Textures::from_paths(vec!["title"], &mut rl, &thread);
    let cards = get_cards_from_file();
    generate_card_textures(&mut rl, &thread, &mut battle_textures, &cards);
    let enemies = get_enemies_from_file();
    generate_enemy_textures(&mut rl, &thread, &mut battle_textures, &enemies);

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

    world.insert(BattleState::new(&cards));
    world.insert(enemies);
    world.insert(cards);
    world.insert(rl);
    world.insert(GameState {
        current_scene: GameScenes::Battle
    });


    let mut menu_dispatcher = specs::DispatcherBuilder::new()
        .with_thread_local(MainMenuDraw { thread: thread.clone(), textures: menu_textures })
        .with(ButtonHandler, "button_handler", &[])
        .build();

    let mut battle_dispatcher = specs::DispatcherBuilder::new()
        .with_thread_local(DrawSys { thread, textures: battle_textures })
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
        .with(DrawCardSys, "draw_cards", &[])
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

    battle_dispatcher.setup(&mut world);
    menu_dispatcher.setup(&mut world);

    loop {
        match get_game_scene(&world) {
            GameScenes::MainMenu => menu_dispatcher.dispatch(&world),
            GameScenes::Battle => battle_dispatcher.dispatch(&world),
        }
        if window_should_close(&world) {
            break;
        }
        world.maintain();
    }
}
