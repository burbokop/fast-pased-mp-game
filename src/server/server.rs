use std::{
    net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use rand::rng;

use crate::common::{
    BroadcastPackage, ClientToServerPackage, Collide as _, Complex, EntityCreateInfo, EntityRole,
    GameState, InitPackage, KillPackage, PacketReader, PacketWriter, PlayerState, Point,
    Segments as _, ServerToClientPackage,
};

pub(crate) fn exec_server(port: u16) {
    let game_state = Arc::new(Mutex::new(GameState::new()));
    let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), port)).unwrap();
    println!("listening started, ready to accept");

    {
        let mut last_proceed_instant = Instant::now();
        let game_state = game_state.clone();
        thread::spawn(move || loop {
            let now = Instant::now();
            {
                game_state
                    .lock()
                    .unwrap()
                    .proceed(now - last_proceed_instant);
            }
            let proceed_duration = Instant::now() - now;

            last_proceed_instant = now;
            std::thread::sleep(Duration::from_millis(1000 / 30) - proceed_duration);
        });
    }

    for stream in listener.incoming() {
        let game_state = game_state.clone();
        thread::spawn(move || -> std::io::Result<()> {
            let player_id = std::thread::current().id().as_u64();
            let mut player_state: PlayerState = Default::default();
            let mut left_mouse_pressed: bool = false;

            let mut stream = stream.unwrap();
            stream.set_nonblocking(true).unwrap();
            let mut reader: PacketReader = Default::default();
            let mut rng = rng();

            let write_package =
                |stream: &mut TcpStream, p: ServerToClientPackage| -> std::io::Result<()> {
                    PacketWriter::write(stream, &serde_json::to_vec(&p).unwrap())
                };

            write_package(
                &mut stream,
                ServerToClientPackage::Init(InitPackage { player_id }),
            )?;

            loop {
                if let Some(data) = reader.read(&mut stream)?.next() {
                    let package: ClientToServerPackage = serde_json::from_slice(&data).unwrap();
                    match package {
                        ClientToServerPackage::PlayerConnected(player_connected_package) => {
                            println!("Player connected: {}", player_id);
                            player_state.color = player_connected_package.color;

                            let mut game_state = game_state.lock().unwrap();
                            let pos = game_state.random_point_inside_bounds(&mut rng);
                            game_state.create(
                                EntityCreateInfo {
                                    pos,
                                    rot: Complex { r: 1., i: 0. },
                                    color: player_state.color.clone(),
                                    role: EntityRole::Character,
                                },
                                player_id,
                            );
                        }
                        ClientToServerPackage::RespawnRequest(_) => {
                            panic!("First package must be init package")
                        }
                        ClientToServerPackage::PlayerInput(_) => {
                            panic!("First package must be init package")
                        }
                    }
                    break;
                }
                std::thread::sleep(Duration::from_millis(1));
            }

            let mut last_broadcust_instant = Instant::now();
            let mut last_sequence_number = 0;
            let mut last_projectile_instant = Instant::now();

            loop {
                for data in reader.read(&mut stream)? {
                    let package: ClientToServerPackage = serde_json::from_slice(&data).unwrap();
                    match package {
                        ClientToServerPackage::PlayerConnected(_) => {
                            panic!("Double init")
                        }
                        ClientToServerPackage::PlayerInput(package) => {
                            let game_state = game_state.lock().unwrap();
                            if let Some(mut entity) =
                                game_state.find_character_by_player_id_mut(player_id)
                            {
                                entity.pos.x = entity.pos.x + package.movement.x;
                                entity.pos.y = entity.pos.y + package.movement.y;
                                entity.rot = package.rotation;

                                for bound in game_state.world_bounds().edges() {
                                    if let Some(exit_vec) =
                                        entity.vertices().segments().collide(&[bound])
                                    {
                                        entity.pos += exit_vec;
                                    }
                                }

                                last_sequence_number = package.sequence_number;
                            };
                            left_mouse_pressed = package.left_mouse_pressed
                        }
                        ClientToServerPackage::RespawnRequest(_) => {
                            if player_state.killed {
                                player_state.killed = false;
                                let mut game_state = game_state.lock().unwrap();
                                let create_info = EntityCreateInfo {
                                    pos: game_state.random_point_inside_bounds(&mut rng),
                                    rot: Complex { r: 1., i: 0. },
                                    color: player_state.color.clone(),
                                    role: EntityRole::Character,
                                };

                                game_state.create(create_info, player_id);
                            }
                        }
                    }
                }

                let now = Instant::now();

                if left_mouse_pressed
                    && now - last_projectile_instant > Duration::from_millis(1000 / 10)
                {
                    let mut game_state = game_state.lock().unwrap();

                    if let Some(character) = game_state
                        .find_character_by_player_id_mut(player_id)
                        .map(|x| x.clone())
                    {
                        game_state.create(
                            EntityCreateInfo {
                                pos: character.pos,
                                rot: character.rot,
                                color: character.color.clone(),
                                role: EntityRole::Projectile,
                            },
                            player_id,
                        );
                    }
                    last_projectile_instant = now;
                }

                {
                    let mut game_state = game_state.lock().unwrap();
                    if game_state.account_kill(player_id) {
                        drop(game_state);
                        player_state.killed = true;
                        write_package(&mut stream, ServerToClientPackage::Kill(KillPackage {}))?;
                    }
                }

                if now - last_broadcust_instant > Duration::from_millis(1000 / 30) {
                    write_package(
                        &mut stream,
                        ServerToClientPackage::Broadcast(BroadcastPackage {
                            game_state: game_state.lock().unwrap().clone(),
                            sequence_number: last_sequence_number,
                            player_state: player_state.clone(),
                        }),
                    )?;
                    last_broadcust_instant = now;
                }

                std::thread::sleep(Duration::from_millis(1));
            }
        });
    }
}
