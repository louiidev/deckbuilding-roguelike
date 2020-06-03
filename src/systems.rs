use crate::components::{
    Active, Button, Card, Enemy, HealthBar, Mousehandler, Player, Position, Rectangle, Sprite,
};
use crate::textures::Textures;
use rand::{thread_rng, Rng};
use raylib::consts::KeyboardKey::*;
use raylib::prelude::MouseButton::*;
use raylib::prelude::*;
use specs::prelude::*;
use specs::{Component, VecStorage};

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
    );

    fn run(&mut self, (mut rl, positions, sprites, active): Self::SystemData) {
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
        WriteStorage<'a, Button>,
        ReadStorage<'a, Rectangle>,
        ReadStorage<'a, Position>,
    );

    fn run(&mut self, (rl, mut buttons, rectangles, positions): Self::SystemData) {
        let mouse_pos = rl.get_mouse_position();
        for (button, rect, position) in (&mut buttons, &rectangles, &positions).join() {
            button.hover = collision_rect_point(*rect, *position, mouse_pos.into());
            if button.hover && rl.is_mouse_button_down(MOUSE_LEFT_BUTTON) {}
        }
    }
}

#[derive(Debug, Default)]
pub struct BattleState {
    card_queue: Vec<Card>,
    targets: Vec<Entity>,
    end_of_turn: bool,
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
            let mouse_ent = (&*entities, &mouse_handler).join().next().unwrap();
            let rectangle = rectangles.get_mut(mouse_ent.0).unwrap();
            let mouse_pos: Position = rl.get_mouse_position().into();
            positions.get_mut(mouse_ent.0).unwrap().x = mouse_pos.x - (rectangle.width / 2.);
            positions.get_mut(mouse_ent.0).unwrap().y = mouse_pos.y - (rectangle.height / 2.);
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
