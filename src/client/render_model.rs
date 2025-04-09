use std::{
    cell::{LazyCell, OnceCell},
    rc::Rc,
    sync::{Arc, OnceLock},
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use font_loader::system_fonts;
use sdl2::{
    gfx::primitives::DrawRenderer,
    pixels,
    rect::{self},
    render::{Canvas, TextureQuery},
    rwops::RWops,
    ttf::Sdl2TtfContext,
    video::Window,
    Sdl,
};

use crate::common::{Color, GameState, PlayerState, Rect};

fn game_color_to_sdl_color(c: Color) -> pixels::Color {
    pixels::Color {
        r: c.r,
        g: c.g,
        b: c.b,
        a: c.a,
    }
}

fn game_rect_to_sdl_rect(c: Rect) -> rect::Rect {
    rect::Rect::new(c.x as i32, c.y as i32, c.w as u32, c.h as u32)
}

struct OwnedFont {
    bytes: Box<Vec<u8>>,
    ctx: Arc<Sdl2TtfContext>,
}

impl OwnedFont {
    pub fn draw_text(
        &self,
        canvas: &mut Canvas<sdl2::video::Window>,
        point: rect::Point,
        color: pixels::Color,
        text: &str,
        font_point_size: u16,
    ) {
        let rwops = RWops::from_bytes(&self.bytes[..]).unwrap();
        let font = self
            .ctx
            .load_font_from_rwops(rwops, font_point_size)
            .unwrap();

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
            .copy(
                &texture,
                None,
                rect::Rect::from_center(point, width, height),
            )
            .unwrap();
    }
}

impl OwnedFont {
    fn new() -> Self {
        let mut property = system_fonts::FontPropertyBuilder::new().monospace().build();
        let sysfonts = system_fonts::query_specific(&mut property);

        static CTX_LOCK: OnceLock<Arc<Sdl2TtfContext>> = OnceLock::new();
        let ctx = CTX_LOCK
            .get_or_init(|| Arc::new(sdl2::ttf::init().map_err(|e| e.to_string()).unwrap()))
            .clone();

        Self {
            bytes: Box::new(
                system_fonts::get(
                    &system_fonts::FontPropertyBuilder::new()
                        .family(sysfonts.first().unwrap())
                        .build(),
                )
                .unwrap()
                .0,
            ),
            ctx,
        }
    }
}

pub(crate) struct RenderModel {
    canvas: Canvas<Window>,
    font: OwnedFont,
    creation_instant: Instant,
}

impl RenderModel {
    pub(crate) fn new(sdl_context: Sdl) -> Result<RenderModel, String> {
        let video_subsystem = sdl_context.video()?;

        let window = video_subsystem
            .window("Fast pased mp game client", 800, 600)
            .position_centered()
            .opengl()
            .build()
            .map_err(|e| e.to_string())?;

        Ok(RenderModel {
            canvas: window.into_canvas().build().map_err(|e| e.to_string())?,
            font: OwnedFont::new(),
            creation_instant: Instant::now(),
        })
    }

    pub(crate) fn render(
        &mut self,
        game_state: &GameState,
        player_state: &PlayerState,
        interpolation_value: f64,
    ) {
        let now = Instant::now();

        self.canvas.set_draw_color(pixels::Color::RGB(255, 0, 0));
        self.canvas.clear();

        self.canvas.set_draw_color(pixels::Color::RGB(255, 255, 0));
        self.canvas
            .draw_rect(game_rect_to_sdl_rect(game_state.world_bounds()))
            .unwrap();

        for entity in game_state.entities() {
            match entity.role {
                crate::common::EntityRole::Character => {
                    let v = entity.vertices();

                    self.canvas
                        .filled_polygon(
                            &[v[0].x as i16, v[1].x as i16, v[2].x as i16, v[3].x as i16],
                            &[v[0].y as i16, v[1].y as i16, v[2].y as i16, v[3].y as i16],
                            game_color_to_sdl_color(entity.color.clone()),
                        )
                        .unwrap();
                }
                crate::common::EntityRole::Projectile => self
                    .canvas
                    .filled_circle(
                        entity.pos.x as i16,
                        entity.pos.y as i16,
                        entity.inscribed_circle_radius() as i16,
                        game_color_to_sdl_color(entity.color.clone()),
                    )
                    .unwrap(),
            }
        }

        self.canvas.set_draw_color(pixels::Color::RGB(0, 255, 0));

        self.font.draw_text(
            &mut self.canvas,
            (40, 40).into(),
            pixels::Color::RGB(0, 255, 255),
            &format!("{:.2}", interpolation_value),
            12,
        );

        let window_size = self.canvas.window().size();
        if player_state.killed {
            self.font.draw_text(
                &mut self.canvas,
                (window_size.0 as i32 / 2, window_size.1 as i32 / 2).into(),
                pixels::Color::RGB(255, 255, 0),
                &format!("You are killed. SPACE to respawn"),
                (10. * (((now - self.creation_instant).as_millis_f32() * 0.002).sin() + 2.)) as u16,
            );
        }

        self.canvas.present();
    }
}
