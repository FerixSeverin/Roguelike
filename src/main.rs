mod config;
mod turn;
mod world;
mod mobility;
mod character;
mod item;

use bevy::prelude::*;
use bevy::render::pass::ClearColor;
use bevy::input::keyboard::KeyboardInput;

//Events

//Materials

struct Materials {
    player: Handle<ColorMaterial>,
    wall: Handle<ColorMaterial>,
    pit: Handle<ColorMaterial>,
    evil: Handle<ColorMaterial>,
}

//Work in progress components, will be moved later to separate files

struct Pit;

struct Model {
    name: String,
    serial_number: String,
}

// Systems

fn setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.spawn((turn::Counter {}, turn::InQueue, turn::Head));
    commands.insert_resource(turn::Queue::new(commands.current_entity().unwrap().clone()));
    commands.insert_resource(world::World::new());

    commands.spawn(Camera2dComponents::default());
    commands.insert_resource(Materials {
        player: materials.add(Color::rgb_u8(0, 163, 204).into()),
        wall: materials.add(Color::rgb_u8(217, 217, 217).into()),
        pit: materials.add(Color::rgb_u8(64, 64, 64).into()),
        evil: materials.add(Color::rgb_u8(237, 76, 47).into()),
    });
}

fn world_setup(
    mut commands: Commands,
    mut turn_queue: ResMut<turn::Queue>,
    mut world: ResMut<world::World>,
    materials: Res<Materials>,
) {
    commands.spawn((item::Item::new("Knife", "Desc", 15), item::Melee::new(5), item::Equippable::new(item::EqiuppedInto::Weapon), item::Damage { base: 5 }));
    let knife_id: Entity = commands.current_entity().unwrap();

    commands.spawn((item::Item::new("Health Potion", "Desc", 5), item::Consumable));
    let potion_id: Entity = commands.current_entity().unwrap();

    let w: world::WorldFile = world::load(format!("{}/world.ron", env!("CARGO_MANIFEST_DIR")));
    let mut y: isize = (w.get().len() - 1) as isize;
    for row in w.get() {
        for (x, tile) in row.chars().enumerate() {
            //let mut sprite_component: SpriteComponents;
            match tile {
                '.' => {
                    world.grid.insert((x as i32, y as i32), world::Tile::empty());
                }
                '#' => {
                    commands
                        .spawn(SpriteComponents {
                            material: materials.wall.clone(),
                            sprite: Sprite::new(Vec2::new(20.0, 20.0)),
                            ..Default::default()
                        })
                        .with(mobility::Blocking)
                        .with(mobility::Position {
                            point: (x as i32, y as i32)
                        });
                    world.grid.insert((x as i32, y as i32), world::Tile::main(commands.current_entity().unwrap()));
                }
                '@' => {
                    commands
                        .spawn(SpriteComponents {
                            material: materials.player.clone(),
                            sprite: Sprite::new(Vec2::new(20.0, 20.0)),
                            ..Default::default()
                        })
                        .with(character::Player)
                        .with(mobility::Position {
                            point: (x as i32, y as i32)
                        })
                        .with(mobility::Walkable { step_size: 1 })
                        .with(character::Attributes::new(20, 5))
                        .with(turn::InQueue)
                        .with(character::Inventory::starting_inventory(vec![knife_id, potion_id]))
                        .with(character::Equipment::naked());
                    turn_queue.add_zero(commands.current_entity().unwrap());
                    world.grid.insert((x as i32, y as i32), world::Tile::main(commands.current_entity().unwrap()));
                }
                'P' => {
                    commands
                        .spawn(SpriteComponents {
                            material: materials.pit.clone(),
                            sprite: Sprite::new(Vec2::new(15.0, 15.0)),
                            ..Default::default()
                        })
                        .with(Pit)
                        .with(mobility::Position {
                            point: (x as i32, y as i32)
                        });
                    world.grid.insert((x as i32, y as i32), world::Tile::base(commands.current_entity().unwrap()));
                }
                'E' => {
                    commands
                        .spawn(SpriteComponents {
                            material: materials.evil.clone(),
                            sprite: Sprite::new(Vec2::new(18.0, 18.0)),
                            ..Default::default()
                        })
                        .with(character::Evil)
                        .with(mobility::Position {
                            point: (x as i32, y as i32)
                        })
                        .with(mobility::Walkable { step_size: 1 })
                        .with(character::Attributes::new(10, 0))
                        .with(Model {
                            name: "E1-L".to_string(),
                            serial_number: "XXXXXX-XX".to_string(),
                        })
                        .with(turn::InQueue);
                    turn_queue.add_zero(commands.current_entity().unwrap());
                    world.grid.insert((x as i32, y as i32), world::Tile::main(commands.current_entity().unwrap()));
                }
                _ => continue,
            };
        }
        y -= 1;
    }
}

fn player_attack(
    keyboard_input: Res<Input<KeyCode>>,
    mut events: ResMut<Events<turn::Done>>,
    mut turn_queue: ResMut<turn::Queue>,
    mut players: Query<(
        Entity,
        &mut character::Player,
        &mut mobility::Position,
        &mut turn::Head,
        &character::Equipment,
    )>,
    mut targets: Query<(
        Entity,
        &character::Evil,
        &mut character::Attributes,
        &mobility::Position,
    )>,
    weapons: Query<(
        &item::Item,
        &item::Equippable,
        &item::Damage,
    )>,
) {
    let mut attempted_to_attack = false;
    let mut attack_direction = (0, 0);
    if keyboard_input.just_pressed(KeyCode::W) {
        attack_direction = (0, 1);
        attempted_to_attack = true;
    } else if keyboard_input.just_pressed(KeyCode::D) {
        attack_direction = (1, 0);
        attempted_to_attack = true;
    } else if keyboard_input.just_pressed(KeyCode::S) {
        attack_direction = (0, -1);
        attempted_to_attack = true;
    } else if keyboard_input.just_pressed(KeyCode::A) {
        attack_direction = (-1, 0);
        attempted_to_attack = true;
    }

    if attempted_to_attack {
        for (_entity, _player, player_position, _head, _equipment) in players.iter_mut() {
            let attack_position = mobility::Position { point: (player_position.x() + attack_direction.0, player_position.y() + attack_direction.1) };
            for (_entity, _evil, mut attributes, target_position) in targets.iter_mut() {
                if target_position.eq(&attack_position) {
                    attributes.health.modifier -= _equipment.get_weapon_damage(&weapons);
                    println!("Attacked! Targets current health is: {}", attributes.health.current());
                    turn_queue.head_makes_action(100);
                    events.send(turn::Done);
                }
            }
        }
    }
}

fn player_movement(
    mut commands: Commands,
    mut turn_queue: ResMut<turn::Queue>,
    mut world: ResMut<world::World>,
    keyboard_input: Res<Input<KeyCode>>,
    mut events: ResMut<Events<turn::Done>>,
    mut players: Query<(
        Entity,
        &mut character::Player,
        &mobility::Walkable,
        &mut mobility::Position,
        &mut turn::Head,
    )>,

) {
    for (player_entity, mut _player, walkable, mut position, _head) in players.iter_mut() {
        let mut attempted_to_walk = false;
        let mut blocked = false;

        let step_size = walkable.step_size;
        let mut position_change = (0, 0);

        if keyboard_input.just_pressed(KeyCode::Up) {
            position_change = (0, step_size);
            attempted_to_walk = true;
        } else if keyboard_input.just_pressed(KeyCode::Right) {
            position_change = (step_size, 0);
            attempted_to_walk = true;
        } else if keyboard_input.just_pressed(KeyCode::Down) {
            position_change = (0, -step_size);
            attempted_to_walk = true;
        } else if keyboard_input.just_pressed(KeyCode::Left) {
            position_change = (-step_size, 0);
            attempted_to_walk = true;
        }

        if attempted_to_walk {
            match world.grid.get(&position.check(position_change)) {
                Some(tile) => match tile.main {
                    Some(_main_entity) => {
                        blocked = true;
                    },
                    None => ()
                },
                None => ()
            }

            if !blocked {
                world.move_main(&position.point, &(position.x() + position_change.0, position.y() + position_change.1));
                position.translate(position_change);
                if !turn_queue.head_makes_action(100) {
                    commands.remove_one::<turn::Head>(player_entity);
                    events.send(turn::Done);
                }
            }
        }
    }
}

fn position_translation(mut q: Query<(&mobility::Position, &mut Transform)>) {
    fn convert(p: i32, position_multiplier: i32) -> i32 {
        p * position_multiplier
    }
    for (pos, mut transform) in q.iter_mut() {
        transform.translation =
            Vec3::new(convert(pos.x(), 20) as f32, convert(pos.y(), 20) as f32, 0.0);
    }
}

fn pit_mechanic(
    mut commands: Commands,
    pits: Query<(Entity, &Pit, &mobility::Position)>,
    mut walkable_entities: Query<(Entity, &mut mobility::Walkable, &mobility::Position)>,
) {
    for (_pit_entity, _pit, pit_positon) in pits.iter() {
        for (walkable_entity, _walkable, walking_position) in walkable_entities.iter_mut() {
            if pit_positon.eq(walking_position) {
                println!("Fell into pit");
                commands.remove_one::<mobility::Walkable>(walkable_entity);
            }
        }
    }
}

fn get_legs(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    mut players: Query<(Entity, &character::Player)>,
) {
    for (player_entity, _player) in players.iter_mut() {
        if keyboard_input.just_pressed(KeyCode::X) {
            println!("Got legs");
            commands.insert_one(player_entity, mobility::Walkable { step_size: 1 });
        }
    }
}

fn turn_management(
    mut commands: Commands,
    mut turn_queue: ResMut<turn::Queue>,
    mut event_reader: Local<EventReader<turn::Done>>,
    events: Res<Events<turn::Done>>,
    entities: Query<(Entity, &turn::InQueue)>,
) {
    for _event in event_reader.iter(&events) {
        println!("Next Turn");
        let head_entity = turn_queue.peek().entity;
        match entities.get(head_entity) {
            Ok(head) => {commands.insert_one(head.0, turn::Head);}
            Err(e) => {
                turn_queue.remove_head();
                println!("{:?}", e);
            }
        }
    }
}

fn turn_tick(
    mut commands: Commands,
    mut turn_queue: ResMut<turn::Queue>,
    mut events: ResMut<Events<turn::Done>>,
    mut turns: Query<(Entity, &turn::Counter, &turn::InQueue, &turn::Head)>,
) {
    for (entity, _counter, _in_queue, _head) in turns.iter_mut() {
        turn_queue.head_makes_action(100);
        commands.remove_one::<turn::Head>(entity);
        events.send(turn::Done);
    }
}

fn inventory_management(
    keyboard_input: Res<Input<KeyCode>>,
    mut players: Query<(Entity, &character::Player, &mut character::Inventory, &mut character::Equipment)>,
    items: Query<&item::Item>,
    equippable_items: Query<(&item::Item, &item::Equippable)>,
) {
    if keyboard_input.just_pressed(KeyCode::I) {
        for (_entity, _player, _inventory, _equipment) in players.iter_mut() {
            _inventory.look(&items);
        }
    }

    if keyboard_input.just_pressed(KeyCode::K) {
        for (_entity, _player, mut _inventory, mut _equipment) in players.iter_mut() {
            _inventory.equip(&mut _equipment.weapon, 0, &equippable_items);
        }
    }
}

fn equipment_management(
    keyboard_input: Res<Input<KeyCode>>,
    players_with_equipment: Query<(Entity, &character::Player, &character::Equipment)>,
    equippable_items: Query<(&item::Item, &item::Equippable)>,
) {
    for (_entity, _player, _equipment) in players_with_equipment.iter() {
        if keyboard_input.just_pressed(KeyCode::O) {
            _equipment.look(&equippable_items);
        }
    }
}

fn death(
    mut commands: Commands,
    mut world: ResMut<world::World>,
    mut entities: Query<(Entity, &mut character::Attributes, &mobility::Position)>,
) {
    for (entity, attributes, position) in entities.iter_mut() {
        if attributes.health.current() <= 0 {
            println!("Entity died!");
            match world.grid.get_mut(&position.point) {
                Some(tile) => {
                    tile.clear_main();
                }
                None => (),
            }
            commands.despawn(entity);
        }
    }
}

fn evil_idle (
    mut commands: Commands,
    mut turn_queue: ResMut<turn::Queue>,
    mut events: ResMut<Events<turn::Done>>,
    entities: Query<(Entity, &turn::Head, &turn::InQueue, &character::Evil)>,
) {
    for (entity, _head, _in_queue, _evil) in entities.iter() {
        println!("Evil idles...");
        turn_queue.head_makes_action(50);
        commands.remove_one::<turn::Head>(entity);
        events.send(turn::Done);
    }
}

fn check_turn_order(
    turn_queue: ResMut<turn::Queue>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::U) {
        turn_queue.check_turn_order();
    }
}

/*fn keyboard_event(
    mut event_reader: Local<EventReader<KeyboardInput>>,
    keyboard_input_events: Res<Events<KeyboardInput>>,
) {
    for event in event_reader.iter(&keyboard_input_events) {
        match event {
            KeyboardInput { scan_code: 72, key_code: up, state: ElementState::Pressed } => {
                println!("Up")
            }
            KeyboardInput { scan_code: 77, key_code: right, state: ElementState::Pressed } => {
                println!("Right")
            }
            KeyboardInput { scan_code: 80, key_code: down, state: ElementState::Pressed } => {
                println!("Down")
            }
            KeyboardInput { scan_code: 75, key_code: left, state: ElementState::Pressed } => {
                println!("Left")
            }
            _ => (),
        }
    }
}*/

fn main() {
    let loaded_config = config::load_config(format!("{}/config.ron", env!("CARGO_MANIFEST_DIR")));
    let world_stage = "world_stage";

    App::build()
        .add_resource(ClearColor(Color::rgb(0.01, 0.01, 0.12)))
        .add_resource(WindowDescriptor {
            width: loaded_config.window_width(),
            height: loaded_config.window_height(),
            title: loaded_config.window_title().to_string(),
            ..Default::default()
        })
        .add_startup_system(setup.system())
        .add_startup_stage(world_stage)
        .add_startup_system_to_stage(world_stage, world_setup.system())
        .add_event::<turn::Done>()
        .add_system(player_movement.system())
        .add_system(position_translation.system())
        .add_system(pit_mechanic.system())
        .add_system(get_legs.system())
        .add_system(turn_management.system())
        .add_system(turn_tick.system())
        .add_system(inventory_management.system())
        .add_system(equipment_management.system())
        .add_system(player_attack.system())
        .add_system(death.system())
        .add_system(evil_idle.system())
        .add_system(check_turn_order.system())
        //.add_system(keyboard_event.system())
        .add_plugins(DefaultPlugins)
        .run();
}
