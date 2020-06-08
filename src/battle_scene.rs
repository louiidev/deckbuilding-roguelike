// use crate::components::{
//     Active, Button, HealthBar, Mousehandler, Player, Position, Rectangle, Sprite,
// };
// use crate::systems::{
//     BattleState, ButtonHandler, CardSelector, DamageQueue, DrawSys, EnemyAttacker,
// };
// use crate::textures::Textures;
// use raylib::prelude::*;
// use specs::prelude::*;

// use crate::{HEIGHT, WIDTH};

// struct BattleScene<'a, 'b> {
//     world: World,
//     dispatcher: Dispatcher<'a, 'b>,
// }

// impl<'a, 'b> BattleScene<'a, 'b> {
//     pub fn new(rl: RaylibHandle, thread: RaylibThread) -> Self {
//         let mut world = World::new();
//         world.register::<Position>();
//         world.register::<Rectangle>();
//         world.register::<Sprite>();
//         world.register::<Mousehandler>();
//         world.register::<HealthBar>();
//         world.register::<Player>();
//         world.register::<Button>();
//         world.register::<Active>();
//         world.insert(rl);
//         let mut textures = Textures::from_paths(vec!["card-back", "mouse-grab"], &mut rl, &thread);
//         let mut dispatcher = specs::DispatcherBuilder::new()
//             .with_thread_local(DrawSys { thread, textures })
//             .with(
//                 CardSelector {
//                     selected: None,
//                     dragging: None,
//                 },
//                 "card_selector",
//                 &[],
//             )
//             .with(DamageQueue, "damage_queue", &["card_selector"])
//             .with(EnemyAttacker, "enemy_attacker", &[])
//             .with(ButtonHandler, "button_handler", &[])
//             .build();
//         dispatcher.setup(&mut world);
//         BattleScene { world, dispatcher }
//     }
//     pub fn setup(&mut self) {
//         self.world
//             .create_entity()
//             .with(Mousehandler)
//             .with(Position { x: 0.0, y: 0.0 })
//             .with(Rectangle {
//                 width: 25.,
//                 height: 25.,
//             })
//             .with(Active(false))
//             .with(Sprite {
//                 scale: 1.,
//                 texture_path: "mouse-grab".to_string(),
//             })
//             .build();

//         self.world
//             .create_entity()
//             .with(HealthBar::new(20))
//             .with(Position { x: 50., y: 100. })
//             .with(Rectangle {
//                 width: 200.,
//                 height: 50.,
//             })
//             .with(Player::default())
//             .build();

//         self.world
//             .create_entity()
//             .with(Position {
//                 x: (WIDTH - 200) as f32,
//                 y: (HEIGHT - 100) as f32,
//             })
//             .with(Rectangle {
//                 width: 150.,
//                 height: 50.,
//             })
//             .with(Button::new("End Turn", "end_turn"))
//             .build();
//     }
// }
