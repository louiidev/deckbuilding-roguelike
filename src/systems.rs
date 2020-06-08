use crate::components::{
    Active, Button, Card, Enemy, HealthBar, Mousehandler, Player, Position, Rectangle, Sprite,
};
use crate::textures::Textures;
use crate::{GameScenes, GameState, WIDTH, HEIGHT};
use crate::cards::generate_intial_deck;

use rand::{thread_rng, Rng};
use raylib::consts::KeyboardKey::*;
use raylib::prelude::MouseButton::*;
use raylib::prelude::*;
use specs::prelude::*;
use specs::{Component, VecStorage};





pub struct MainMenuDraw {
    pub thread: RaylibThread,
    pub textures: Textures
}

impl<'a> System<'a> for MainMenuDraw {
    type SystemData = (
        WriteExpect<'a, RaylibHandle>,
        WriteExpect<'a, GameState>,
    );
    
    fn run(&mut self, (mut rl, mut state): Self::SystemData) {
        use std::ffi::CString;
        use raylib::prelude::Rectangle;
        let width = 200.;
        let height = 50.;
        let x = (WIDTH as f32 / 2.) - (width / 2.);
        let y = (HEIGHT as f32 / 2.) - (height / 2.);
        
        let start_btn = rgui::Button {
            bounds: Rectangle::new(x, y, width, height),
            text: CString::new("Start Battle").unwrap(),
        };
        let how_to_btn = rgui::Button {
            bounds: Rectangle::new(x, y + 60., width, height),
            text: CString::new("How to play").unwrap(),
        };
    
        let mut d = rl.begin_drawing(&self.thread);
        d.clear_background(crate::COLOUR);
        let texture = self.textures.get("title");
        let t_width = texture.width;

        d.draw_texture_ex(texture, Vector2 { x: x - (t_width as f32 / 2.), y: y - 200. }, 0., 2., Color::WHITE);
        if let rgui::DrawResult::Bool(b) = d.draw_gui(&start_btn) {
            if b {
                state.current_scene = GameScenes::Battle;
            }
        }

        if let rgui::DrawResult::Bool(b) = d.draw_gui(&how_to_btn) {
            if b {
                state.current_scene = GameScenes::Battle;
            }
        }
        
    }
}

// System is not thread safe
pub struct DrawSys {
    pub thread: RaylibThread,
    pub textures: Textures,
}
impl<'a> System<'a> for DrawSys {
    type SystemData = (
        WriteExpect<'a, RaylibHandle>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Sprite>,
        ReadStorage<'a, Active>,
        ReadStorage<'a, Rectangle>,
        ReadStorage<'a, HealthBar>,
        ReadStorage<'a, Button>,
    );

    fn run(&mut self, (mut rl, positions, sprites, active, rectangles, health_bars, buttons): Self::SystemData) {
        let mut d = rl.begin_drawing(&self.thread);
        d.clear_background(crate::COLOUR);
        let sprites_to_render: Vec<(&Sprite, &Position, &Active)> = (&sprites, &positions, &active)
            .join()
            .filter(|(_, _, active)| active.0)
            .collect();
        for (sprite, Position { x, y }, _) in sprites_to_render {
            d.draw_texture_ex(
                self.textures.get(&sprite.texture_path),
                Vector2 { x: *x, y: *y },
                0.,
                sprite.scale,
                Color::WHITE,
            );
        }

        for (health_bar, position, rect) in (&health_bars, &positions, &rectangles).join() {
            d.draw_rectangle_rec(raylib::prelude::Rectangle::new(position.x, position.y - 50., rect.width * (health_bar.current as f32 / health_bar.max as f32), 40.), Color::GREEN);
            d.draw_rectangle_lines_ex(raylib::prelude::Rectangle::new(position.x, position.y - 50., rect.width, 40.), 3, Color::BLACK);
            let text = format!("{}/{}", health_bar.current, health_bar.max);
            let size = measure_text(&text, 20);
            d.draw_text(&text, (position.x + (rect.width / 2.)) as i32 - (size / 2), position.y as i32 - 40, 20, Color::WHITE);
        }

        for (Button { color, text, hover, ..}, position, rect) in (&buttons, &positions, &rectangles).join() {
            let alpha = if *hover {
                100
            } else {
                255
            };
            let color = Color::new(color[0], color[1], color[2], alpha);
            d.draw_rectangle_rec(raylib::prelude::Rectangle::new(position.x, position.y, rect.width, rect.height), color);
            d.draw_rectangle_lines_ex(raylib::prelude::Rectangle::new(position.x, position.y, rect.width, rect.height), 3, Color::BLACK);
            let size = measure_text(&text, 20);
            d.draw_text(&text, (position.x + (rect.width / 2.)) as i32 - (size / 2), position.y as i32 + 15, 20, Color::WHITE);
        }
    }
}

pub fn collision_rect_point(rectangle: Rectangle, position: Position, point: Position) -> bool {
    let Rectangle { width, height } = rectangle;
    position.x <= point.x
        && point.x <= position.x + width
        && position.y <= point.y
        && point.y <= position.y + height
}

pub struct ButtonHandler;
impl<'a> System<'a> for ButtonHandler {
    type SystemData = (
        ReadExpect<'a, RaylibHandle>,
        WriteExpect<'a, BattleState>,
        WriteStorage<'a, Button>,
        ReadStorage<'a, Rectangle>,
        ReadStorage<'a, Position>,
    );

    fn run(&mut self, (rl, mut battle_state, mut buttons, rectangles, positions): Self::SystemData) {
        let mouse_pos = rl.get_mouse_position();
        for (button, rect, position) in (&mut buttons, &rectangles, &positions).join() {
            button.hover = collision_rect_point(*rect, *position, mouse_pos.into());
            if button.hover && rl.is_mouse_button_pressed(MOUSE_LEFT_BUTTON) {
                match button.action {
                    "end_turn" => {
                        battle_state.end_of_turn = true;
                    },
                    _ => {}
                }
            }
        }
    }
}
use crate::CardID;
#[derive(Debug, Default)]
pub struct BattleState {
    card_queue: Vec<Card>,
    targets: Vec<Entity>,
    end_of_turn: bool,
    deck: Vec<CardID>,
    hand: Vec<CardID>,
    discard: Vec<CardID>
}

impl BattleState {
    pub fn new(cards: &crate::CardDB) -> Self {
        BattleState {
            card_queue: Vec::new(),
            targets: Vec::new(),
            end_of_turn: false,
            deck: generate_intial_deck(cards),
            hand: Vec::new(),
            discard: Vec::new()
        }
    }
    
}

pub struct EnemyAttacker;
impl<'a> System<'a> for EnemyAttacker {
    type SystemData = (
        WriteStorage<'a, Enemy>,
        WriteExpect<'a, BattleState>,
        ReadStorage<'a, Player>,
        WriteStorage<'a, HealthBar>,
    );

    fn run(&mut self, (enemies, mut battle_state, players, mut health_bars): Self::SystemData) {
        if battle_state.end_of_turn {
            for enemy in enemies.join() {
                if enemy.open {
                    for (_, health_bar) in (&players, &mut health_bars).join() {
                        health_bar.current = std::cmp::max(0, health_bar.current - enemy.attack);
                    }
                }
            }
            battle_state.end_of_turn = false;
        }
    }
}

pub struct DamageQueue;
impl<'a> System<'a> for DamageQueue {
    type SystemData = (
        WriteExpect<'a, BattleState>,
        WriteStorage<'a, Enemy>,
        WriteStorage<'a, HealthBar>,
        Entities<'a>,
    );

    fn run(
        &mut self,
        (mut battle_state, mut enemies, mut health_bars, entities): Self::SystemData,
    ) {
        for i in 0..battle_state.card_queue.len() {
            let card = battle_state.card_queue.get(i).unwrap().clone();
            let target_ent = battle_state.targets.get_mut(i).unwrap();
            let enemy = enemies.get_mut(*target_ent).unwrap();
            let health_bar = health_bars.get_mut(*target_ent).unwrap();
            enemy.health = std::cmp::max(0, enemy.health - card.value);
            if enemy.health == 0 {
                entities.delete(*target_ent);
            }
            health_bar.current = enemy.health;
        }
        battle_state.card_queue = vec![];
        battle_state.targets = vec![];
    }
}

pub struct CardSelector {
    pub selected: Option<Entity>,
    pub dragging: Option<Entity>,
}

impl<'a> System<'a> for CardSelector {
    type SystemData = (
        WriteExpect<'a, BattleState>,
        ReadStorage<'a, Mousehandler>,
        ReadExpect<'a, RaylibHandle>,
        ReadStorage<'a, Card>,
        WriteStorage<'a, Rectangle>,
        WriteStorage<'a, Sprite>,
        WriteStorage<'a, Enemy>,
        WriteStorage<'a, HealthBar>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Active>,
        Entities<'a>,
    );

    fn run(
        &mut self,
        (
            mut battle_state,
            mouse_handler,
            rl,
            cards,
            mut rectangles,
            mut sprites,
            mut enemies,
            mut health_bars,
            mut positions,
            mut actives,
            entities,
        ): Self::SystemData,
    ) {
        let mut selected: Option<Entity> = None;

        for (rect, position, ent, _) in (&mut rectangles, &mut positions, &entities, &cards).join() {
            if collision_rect_point(*rect, *position, rl.get_mouse_position().into()) {
                selected = Option::from(ent);
            }
        }

        if let Some(s) = selected {
            if selected != self.selected {
                if let Some(gs) = self.selected {
                    positions.get_mut(gs).unwrap().y += 50.;
                }
                positions.get_mut(s).unwrap().y -= 50.;
                self.selected = Option::from(s);
            }
        } else {
            if let Some(gs) = self.selected {
                positions.get_mut(gs).unwrap().y += 50.;
            }
            self.selected = None;
        }

        if rl.is_mouse_button_pressed(MOUSE_LEFT_BUTTON) && self.selected.is_some() {
            self.dragging = self.selected;
            let mouse_ent = (&*entities, &mouse_handler).join().next().unwrap();
            actives.get_mut(self.dragging.unwrap()).unwrap().0 = false;
            actives.get_mut(mouse_ent.0).unwrap().0 = true;
        }

        if rl.is_mouse_button_down(MOUSE_LEFT_BUTTON) {
            let mouse_ent = (&*entities, &mouse_handler).join().next();
            if let Some(mouse) = mouse_ent {
                let rectangle = rectangles.get_mut(mouse.0).unwrap();
                let mouse_pos: Position = rl.get_mouse_position().into();
                positions.get_mut(mouse.0).unwrap().x = mouse_pos.x - (rectangle.width / 2.);
                positions.get_mut(mouse.0).unwrap().y = mouse_pos.y - (rectangle.height / 2.);
            }
           
        }

        if rl.is_mouse_button_up(MOUSE_LEFT_BUTTON) && self.dragging.is_some() {
            let mouse_ent = (&*entities, &mouse_handler).join().next().unwrap();
            actives.get_mut(mouse_ent.0).unwrap().0 = false;
            let mut removed = false;
            for (rectangle, position, enemy, ent) in (&mut rectangles, &mut positions, &mut enemies, &*entities).join() {
                if collision_rect_point(*rectangle, *position, rl.get_mouse_position().into()) {
                    if !enemy.open {
                        let sprite = sprites.get_mut(ent).unwrap();
                        let template_name = format!("h:{0}_a:{1}", enemy.health, enemy.attack);
                        sprite.texture_path = template_name;
                        enemy.open = true;
                        health_bars.insert(ent, HealthBar::new(enemy.health));
                    }
                    let _res = entities.delete(self.dragging.unwrap());
                    removed = true;
                    battle_state
                        .card_queue
                        .push(cards.get(self.dragging.unwrap()).unwrap().clone());
                    battle_state.targets.push(ent);
                    break;
                }
            }
            if !removed {
                actives.get_mut(self.dragging.unwrap()).unwrap().0 = true;
            }
            self.dragging = None;
        }
    }
}


pub struct DrawCardSys;

impl<'a> System<'a> for DrawCardSys {
    type SystemData = (
        WriteExpect<'a, BattleState>,
    );
    fn run(&mut self, (mut battle_state,):Self::SystemData) {
        if battle_state.hand.is_empty() {
            if battle_state.deck.len() <= 2 {
                let mut discard_pile = battle_state.discard.clone();
                //@TODO:
                // shuffle discard into deck
                // for now: 
                battle_state.deck.append(&mut discard_pile);
                battle_state.discard = Vec::new();
            }

            for _ in 0..2 {
                if let Some(card_id) = battle_state.deck.pop() {
                    battle_state.hand.push(card_id);
                } else {
                    panic!("You tried to take a card from the deck without there being a value in the deck");
                }
            }
        }
    }
}