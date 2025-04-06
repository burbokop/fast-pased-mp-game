use std::{
    net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use crate::common::{
    BroadcastPackage, ClientToServerPackage, EntityCreateInfo, EntityRole, GameState, InitPackage,
    PacketReader, PacketWriter, ServerToClientPackage,
};

pub(crate) fn exec_server(port: u16) {
    let game_state = Arc::new(Mutex::new(GameState::new()));
    let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), port)).unwrap();
    println!("listening started, ready to accept");
    for stream in listener.incoming() {
        let game_state = game_state.clone();
        thread::spawn(move || -> std::io::Result<()> {
            let player_id = std::thread::current().id().as_u64();
            let mut left_mouse_pressed: bool = false;

            let mut stream = stream.unwrap();
            stream.set_nonblocking(true).unwrap();
            let mut reader: PacketReader = Default::default();

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
                    println!("got package: {:?}", package);
                    match package {
                        ClientToServerPackage::PlayerConnected(player_connected_package) => {
                            game_state
                                .lock()
                                .unwrap()
                                .create(player_connected_package.entity_create_info, player_id);
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

            let mut last_prcceed_instant = Instant::now();

            loop {
                for data in reader.read(&mut stream).unwrap() {
                    let package: ClientToServerPackage = serde_json::from_slice(&data).unwrap();
                    match package {
                        ClientToServerPackage::PlayerConnected(_) => {
                            panic!("Double init")
                        }
                        ClientToServerPackage::PlayerInput(package) => {
                            let game_state = game_state.lock().unwrap();

                            let mut entity = game_state
                                .find_character_by_player_id_mut(player_id)
                                .unwrap();
                            entity.pos.x = entity.pos.x + package.movement.x;
                            entity.pos.y = entity.pos.y + package.movement.y;
                            entity.rot = package.rotation;

                            last_sequence_number = package.sequence_number;
                            left_mouse_pressed = package.left_mouse_pressed
                        }
                    }
                }

                let now = Instant::now();

                if left_mouse_pressed
                    && now - last_projectile_instant > Duration::from_millis(1000 / 10)
                {
                    let mut game_state = game_state.lock().unwrap();

                    let character = game_state
                        .find_character_by_player_id_mut(player_id)
                        .unwrap()
                        .clone();

                    game_state.create(
                        EntityCreateInfo {
                            pos: character.pos,
                            rot: character.rot,
                            color: character.color.clone(),
                            role: EntityRole::Projectile,
                        },
                        player_id,
                    );
                    last_projectile_instant = now;
                }

                game_state
                    .lock()
                    .unwrap()
                    .proceed(now - last_prcceed_instant);
                last_prcceed_instant = now;

                if now - last_broadcust_instant > Duration::from_millis(1000 / 30) {
                    write_package(
                        &mut stream,
                        ServerToClientPackage::Broadcast(BroadcastPackage {
                            game_state: game_state.lock().unwrap().clone(),
                            sequence_number: last_sequence_number,
                        }),
                    )?;
                    last_broadcust_instant = now;
                }

                std::thread::sleep(Duration::from_millis(1));
            }
        });
    }
}
