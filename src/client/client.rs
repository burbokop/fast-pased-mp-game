use std::{
    cell::RefCell,
    net::{SocketAddrV4, TcpStream},
    num::NonZero,
    ops::Deref,
    time::{Duration, Instant},
};

use rand::{rng, Rng};
use sdl2::{event::Event, keyboard::Keycode, mouse::MouseButton};

use crate::{
    client::RenderModel,
    common::{
        ClientToServerPackage, Collide as _, Color, Complex, EntityCreateInfo, EntityRole,
        GameState, PacketReader, PacketWriter, PlayerConnectedPackage, PlayerInputPackage,
        PlayerState, Point, RespawnRequestPackage, Segments as _, ServerToClientPackage, Vector,
    },
};

pub(crate) struct GameStateQueue {
    pub(crate) prediction: GameState,
    pub(crate) last_received: GameState,
    pub(crate) penultimate_received: GameState,
}

impl GameStateQueue {
    pub(crate) fn new() -> Self {
        Self {
            prediction: GameState::new(),
            last_received: GameState::new(),
            penultimate_received: GameState::new(),
        }
    }
}

struct Networker {
    stream: RefCell<TcpStream>,
    reader: RefCell<PacketReader>,
    player_id: Option<NonZero<u64>>,
    last_broadcast_instant: Instant,
    last_broadcast_insterval: Duration,
}

impl Networker {
    pub fn connect(addr: SocketAddrV4) -> std::io::Result<Networker> {
        let stream = TcpStream::connect(addr)?;
        stream.set_nonblocking(true)?;
        Ok(Networker {
            stream: RefCell::new(stream),
            reader: Default::default(),
            player_id: None,
            last_broadcast_instant: Instant::now(),
            last_broadcast_insterval: Duration::from_millis(0),
        })
    }

    fn interpolation_value(&self) -> f64 {
        let now = Instant::now();
        (now - self.last_broadcast_instant).div_duration_f64(self.last_broadcast_insterval)
    }

    pub fn write_package(&self, p: ClientToServerPackage) -> std::io::Result<()> {
        PacketWriter::write(
            &mut self.stream.borrow_mut().deref(),
            &serde_json::to_vec(&p).unwrap(),
        )
    }

    pub fn proceed(
        &mut self,
        game_state_queue: &mut GameStateQueue,
        player_state: &mut PlayerState,
        last_sequence_number: u32,
    ) -> std::io::Result<()> {
        for data in self
            .reader
            .borrow_mut()
            .read_ref(self.stream.borrow_mut())?
        {
            let package: ServerToClientPackage = serde_json::from_slice(&data).unwrap();

            let mut rng = rng();

            match package {
                ServerToClientPackage::Init(init_package) => {
                    self.player_id = Some(init_package.player_id);
                    self.write_package(ClientToServerPackage::PlayerConnected(
                        PlayerConnectedPackage {
                            color: Color {
                                a: rng.random(),
                                r: rng.random(),
                                g: rng.random(),
                                b: rng.random(),
                            },
                        },
                    ))
                    .unwrap();
                }
                ServerToClientPackage::Broadcast(broadcast_package) => {
                    let player_entity_copy = self
                        .player_id
                        .map(|id| {
                            game_state_queue
                                .prediction
                                .find_character_by_player_id_mut(id)
                        })
                        .flatten()
                        .map(|x| x.clone());

                    game_state_queue.penultimate_received = game_state_queue.last_received.clone();
                    game_state_queue.last_received = broadcast_package.game_state.clone();
                    game_state_queue.prediction = broadcast_package.game_state;

                    if broadcast_package.sequence_number < last_sequence_number
                        && !player_state.killed
                    {
                        if let (Some(player_id), Some(player_entity_copy)) =
                            (self.player_id, player_entity_copy)
                        {
                            game_state_queue
                                .prediction
                                .add_or_replace_character_by_player_id(
                                    player_id,
                                    player_entity_copy,
                                )
                        }
                    }

                    let now = Instant::now();
                    self.last_broadcast_insterval = now - self.last_broadcast_instant;
                    self.last_broadcast_instant = now;
                }
                ServerToClientPackage::Kill(_) => player_state.killed = true,
            }
        }

        if let Some(player_id) = self.player_id {
            GameState::lerp_merge(
                &mut game_state_queue.prediction,
                &game_state_queue.penultimate_received,
                &game_state_queue.last_received,
                self.interpolation_value(),
                player_id,
            );
        }

        Ok(())
    }
}

struct Controlls {
    left_pressed: bool,
    right_pressed: bool,
    up_pressed: bool,
    down_pressed: bool,
    space_pressed: bool,
    mouse_pos: Point,
    left_mouse_pressed: bool,
    old_left_mouse_pressed: bool,
}

impl Controlls {
    fn new() -> Self {
        Self {
            left_pressed: false,
            right_pressed: false,
            up_pressed: false,
            down_pressed: false,
            space_pressed: false,
            mouse_pos: Point { x: 0., y: 0. },
            left_mouse_pressed: false,
            old_left_mouse_pressed: false,
        }
    }
}

pub(crate) fn exec_client(addr: SocketAddrV4) -> Result<(), String> {
    println!("Running client. Connecting to {}", addr);

    let mut game_state_queue = GameStateQueue::new();
    let mut controlls = Controlls::new();
    let mut last_sequence_number: u32 = 0;
    let mut networker = Networker::connect(addr).unwrap();
    let sdl_context = sdl2::init()?;
    let mut event_pump = sdl_context.event_pump()?;
    let mut render_model = RenderModel::new(sdl_context)?;
    let mut player_state: PlayerState = Default::default();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,

                Event::KeyDown {
                    keycode: Some(Keycode::W),
                    ..
                } => controlls.up_pressed = true,
                Event::KeyUp {
                    keycode: Some(Keycode::W),
                    ..
                } => controlls.up_pressed = false,

                Event::KeyDown {
                    keycode: Some(Keycode::A),
                    ..
                } => controlls.left_pressed = true,
                Event::KeyUp {
                    keycode: Some(Keycode::A),
                    ..
                } => controlls.left_pressed = false,

                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    ..
                } => controlls.down_pressed = true,
                Event::KeyUp {
                    keycode: Some(Keycode::S),
                    ..
                } => controlls.down_pressed = false,

                Event::KeyDown {
                    keycode: Some(Keycode::D),
                    ..
                } => controlls.right_pressed = true,
                Event::KeyUp {
                    keycode: Some(Keycode::D),
                    ..
                } => controlls.right_pressed = false,

                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => controlls.space_pressed = true,
                Event::KeyUp {
                    keycode: Some(Keycode::Space),
                    ..
                } => controlls.space_pressed = false,

                Event::MouseMotion { x, y, .. } => {
                    controlls.mouse_pos = Point {
                        x: x as f32,
                        y: y as f32,
                    }
                }
                Event::MouseButtonDown {
                    mouse_btn: MouseButton::Left,
                    ..
                } => controlls.left_mouse_pressed = true,
                Event::MouseButtonUp {
                    mouse_btn: MouseButton::Left,
                    ..
                } => controlls.left_mouse_pressed = false,

                _ => {}
            }
        }

        if let Some(player_id) = networker.player_id {
            if player_state.killed {
                if controlls.space_pressed {
                    player_state.killed = false;
                    networker
                        .write_package(ClientToServerPackage::RespawnRequest(
                            RespawnRequestPackage {},
                        ))
                        .unwrap();
                }
            } else {
                let velocity = 5.;

                let mut movement = Vector { x: 0., y: 0. };

                if controlls.left_pressed {
                    movement.x = -velocity;
                } else if controlls.right_pressed {
                    movement.x = velocity;
                }
                if controlls.up_pressed {
                    movement.y = -velocity;
                } else if controlls.down_pressed {
                    movement.y = velocity;
                }

                if let Some(mut entity) = game_state_queue
                    .prediction
                    .find_character_by_player_id_mut(player_id)
                {
                    entity.pos.x = entity.pos.x + movement.x;
                    entity.pos.y = entity.pos.y + movement.y;
                    let old_rot = entity.rot;
                    entity.rot = (controlls.mouse_pos - entity.pos).normalize_into_complex();

                    for bound in game_state_queue.prediction.world_bounds().edges() {
                        if let Some(exit_vec) = entity.vertices().segments().collide(&[bound]) {
                            entity.pos += exit_vec;
                        }
                    }

                    if movement.x != 0.
                        || movement.y != 0.
                        || entity.rot != old_rot
                        || controlls.old_left_mouse_pressed != controlls.left_mouse_pressed
                    {
                        last_sequence_number += 1;
                        networker
                            .write_package(ClientToServerPackage::PlayerInput(PlayerInputPackage {
                                sequence_number: last_sequence_number,
                                movement,
                                rotation: entity.rot,
                                left_mouse_pressed: controlls.left_mouse_pressed,
                            }))
                            .unwrap();
                    }

                    controlls.old_left_mouse_pressed = controlls.left_mouse_pressed
                }
            }
        }

        networker
            .proceed(
                &mut game_state_queue,
                &mut player_state,
                last_sequence_number,
            )
            .unwrap();

        render_model.render(
            &game_state_queue.prediction,
            &player_state,
            networker.interpolation_value(),
        );

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}
