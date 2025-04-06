use core::str;
use std::{
    cell::RefCell,
    net::{SocketAddrV4, TcpStream},
    num::NonZero,
    ops::Deref,
    time::{Duration, Instant},
};

use font_loader::system_fonts;
use rand::{rng, Rng};
use sdl2::{
    event::Event,
    keyboard::Keycode,
    pixels::{self},
    rect::{self, Rect},
    render::{Canvas, TextureQuery},
    rwops::RWops,
    ttf::Font,
};

use crate::common::{
    ClientToServerPackage, Color, EntityCreateInfo, GameState, PacketReader, PacketWriter,
    PlayerConnectedPackage, PlayerInputPackage, Point, ServerToClientPackage, Vector,
};

fn game_color_to_sdl_color(c: Color) -> pixels::Color {
    pixels::Color {
        r: c.r,
        g: c.g,
        b: c.b,
        a: c.a,
    }
}

pub(crate) struct GameStateQueue {
    prediction: GameState,
    last_received: GameState,
    penultimate_received: GameState,
}

impl GameStateQueue {
    fn new() -> Self {
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
                            entity_create_info: EntityCreateInfo {
                                pos: Point { x: 500, y: 500 },
                                color: Color {
                                    a: rng.random(),
                                    r: rng.random(),
                                    g: rng.random(),
                                    b: rng.random(),
                                },
                            },
                        },
                    ))
                    .unwrap();
                }
                ServerToClientPackage::Broadcast(broadcast_package) => {
                    let player_entity_copy = self
                        .player_id
                        .map(|id| game_state_queue.prediction.find_by_player_id_mut(id))
                        .flatten()
                        .map(|x| x.clone());

                    game_state_queue.penultimate_received = game_state_queue.last_received.clone();
                    game_state_queue.last_received = broadcast_package.game_state.clone();
                    game_state_queue.prediction = broadcast_package.game_state;

                    if broadcast_package.sequence_number < last_sequence_number {
                        if let (Some(player_id), Some(player_entity_copy)) =
                            (self.player_id, player_entity_copy)
                        {
                            game_state_queue
                                .prediction
                                .add_or_replace_by_player_id(player_id, player_entity_copy)
                        }
                    }

                    let now = Instant::now();
                    self.last_broadcast_insterval = now - self.last_broadcast_instant;
                    self.last_broadcast_instant = now;
                }
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
}

impl Controlls {
    fn new() -> Self {
        Self {
            left_pressed: false,
            right_pressed: false,
            up_pressed: false,
            down_pressed: false,
        }
    }
}

pub fn draw_text(
    canvas: &mut Canvas<sdl2::video::Window>,
    font: &Font,
    point: rect::Point,
    color: pixels::Color,
    text: &str,
) {
    let texture_creator = canvas.texture_creator();
    let surface = font
        .render(text)
        .blended(color)
        .map_err(|e| e.to_string())
        .unwrap();
    let texture = texture_creator
        .create_texture_from_surface(&surface)
        .map_err(|e| e.to_string())
        .unwrap();

    let TextureQuery { width, height, .. } = texture.query();
    canvas
        .copy(&texture, None, Rect::from_center(point, width, height))
        .unwrap();
}

pub(crate) fn exec_client(addr: SocketAddrV4) -> Result<(), String> {
    println!("Running client. Connecting to {}", addr);

    let mut game_state_queue = GameStateQueue::new();
    let mut controlls = Controlls::new();
    let mut last_sequence_number: u32 = 0;

    let mut networker = Networker::connect(addr).unwrap();

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("Fast pased mp game client", 800, 600)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    let mut property = system_fonts::FontPropertyBuilder::new().monospace().build();
    let sysfonts = system_fonts::query_specific(&mut property);
    let font_bytes = system_fonts::get(
        &system_fonts::FontPropertyBuilder::new()
            .family(sysfonts.first().unwrap())
            .build(),
    )
    .unwrap();
    let rwops = RWops::from_bytes(&font_bytes.0[..]).unwrap();

    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string()).unwrap();

    let font = ttf_context.load_font_from_rwops(rwops, 12).unwrap();

    let mut event_pump = sdl_context.event_pump()?;

    let mut int: f64 = 0.;

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
                    keycode: Some(Keycode::Left),
                    ..
                } => {
                    int -= 0.05;
                    println!("int: {}", int);
                }

                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => {
                    int += 0.05;
                    println!("int: {}", int);
                }

                _ => {}
            }
        }

        if let Some(player_id) = networker.player_id {
            let velocity = 5;

            let mut movement = Vector { x: 0, y: 0 };

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

            if let Some(mut entity) = game_state_queue.prediction.find_by_player_id_mut(player_id) {
                entity.pos.x = entity.pos.x + movement.x;
                entity.pos.y = entity.pos.y + movement.y;

                if movement.x != 0 || movement.y != 0 {
                    last_sequence_number += 1;
                    networker
                        .write_package(ClientToServerPackage::PlayerInput(PlayerInputPackage {
                            sequence_number: last_sequence_number,
                            movement,
                        }))
                        .unwrap();
                }
            }
        }

        networker
            .proceed(&mut game_state_queue, last_sequence_number)
            .unwrap();

        canvas.set_draw_color(pixels::Color::RGB(255, 0, 0));
        canvas.clear();

        for entity in game_state_queue.prediction.entities() {
            canvas.set_draw_color(game_color_to_sdl_color(entity.color.clone()));
            canvas
                .fill_rect(Rect::from_center(
                    (entity.pos.x as i32, entity.pos.y as i32),
                    16,
                    16,
                ))
                .unwrap();
        }

        canvas.set_draw_color(pixels::Color::RGB(0, 255, 0));

        if controlls.left_pressed {
            canvas
                .fill_rect(Rect::from_center((100 - 16, 100), 16, 16))
                .unwrap();
        } else {
            canvas
                .draw_rect(Rect::from_center((100 - 16, 100), 16, 16))
                .unwrap();
        }
        if controlls.right_pressed {
            canvas
                .fill_rect(Rect::from_center((100 + 16, 100), 16, 16))
                .unwrap();
        } else {
            canvas
                .draw_rect(Rect::from_center((100 + 16, 100), 16, 16))
                .unwrap();
        }
        if controlls.up_pressed {
            canvas
                .fill_rect(Rect::from_center((100, 100 - 16), 16, 16))
                .unwrap();
        } else {
            canvas
                .draw_rect(Rect::from_center((100, 100 - 16), 16, 16))
                .unwrap();
        }
        if controlls.down_pressed {
            canvas
                .fill_rect(Rect::from_center((100, 100 + 16), 16, 16))
                .unwrap();
        } else {
            canvas
                .draw_rect(Rect::from_center((100, 100 + 16), 16, 16))
                .unwrap();
        }

        draw_text(
            &mut canvas,
            &font,
            (40, 40).into(),
            pixels::Color::RGB(0, 255, 255),
            &format!("{:.2}", networker.interpolation_value()),
        );

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}
