use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{
    cell::{Ref, RefCell, RefMut},
    num::NonZero,
    time::{Duration, Instant},
};

use super::{Complex, Point, Rect, Segment, Vector};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct Color {
    pub(crate) a: u8,
    pub(crate) r: u8,
    pub(crate) g: u8,
    pub(crate) b: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EntityRole {
    Character,
    Projectile,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct Entity {
    pub(crate) id: u32,
    pub(crate) player_id: NonZero<u64>,
    #[serde(skip)]
    pub(crate) birth_instant: Option<Instant>,
    pub(crate) pos: Point,
    pub(crate) rot: Complex,
    pub(crate) color: Color,
    pub(crate) role: EntityRole,
}

impl Entity {
    pub(crate) fn lerp(a: Self, b: Self, t: f64) -> Self {
        Entity {
            id: b.id,
            player_id: b.player_id,
            birth_instant: b.birth_instant,
            pos: Point::lerp(a.pos, b.pos, t),
            rot: Complex::lerp(a.rot, b.rot, t),
            color: b.color,
            role: b.role,
        }
    }

    pub(crate) fn inscribed_circle_radius(&self) -> f32 {
        match self.role {
            EntityRole::Character => 8.,
            EntityRole::Projectile => 4.,
        }
    }

    pub(crate) fn vertices(&self) -> [Point; 4] {
        [
            self.pos + Vector { x: -8., y: -8. } * self.rot,
            self.pos + Vector { x: 8., y: -8. } * self.rot,
            self.pos + Vector { x: 8., y: 8. } * self.rot,
            self.pos + Vector { x: -8., y: 8. } * self.rot,
        ]
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct EntityCreateInfo {
    pub(crate) pos: Point,
    pub(crate) rot: Complex,
    pub(crate) color: Color,
    pub(crate) role: EntityRole,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct GameState {
    entities: Vec<RefCell<Entity>>,
    world_bounds: Rect,
    next_entity_id: u32,
    kills: Vec<u32>,
}

impl GameState {
    pub(crate) fn new() -> Self {
        Self {
            entities: vec![],
            world_bounds: Rect {
                x: 32.,
                y: 32.,
                w: 800. - 64.,
                h: 600. - 64.,
            },
            next_entity_id: 0,
            kills: Default::default(),
        }
    }

    pub(crate) fn world_bounds(&self) -> Rect {
        self.world_bounds
    }

    pub(crate) fn random_point_inside_bounds<R: Rng>(&self, rng: &mut R) -> Point {
        Point {
            x: rng.random_range(self.world_bounds.x..(self.world_bounds.x + self.world_bounds.w)),
            y: rng.random_range(self.world_bounds.y..(self.world_bounds.y + self.world_bounds.h)),
        }
    }

    pub(crate) fn create(&mut self, entity: EntityCreateInfo, player_id: NonZero<u64>) {
        self.entities.push(RefCell::new(Entity {
            id: self.next_entity_id,
            player_id,
            birth_instant: Some(Instant::now()),
            pos: entity.pos,
            rot: entity.rot,
            color: entity.color,
            role: entity.role,
        }));
        self.next_entity_id += 1;
    }

    pub(crate) fn entities<'a>(&'a self) -> impl Iterator<Item = Ref<'a, Entity>> {
        self.entities.iter().filter_map(|x| x.try_borrow().ok())
    }

    pub(crate) fn entities_mut<'a>(&'a self) -> impl Iterator<Item = RefMut<'a, Entity>> {
        self.entities.iter().filter_map(|x| x.try_borrow_mut().ok())
    }

    pub(crate) fn find_character_by_player_id_mut<'a>(
        &'a self,
        player_id: NonZero<u64>,
    ) -> Option<RefMut<'a, Entity>> {
        self.entities
            .iter()
            .find_map(|x| match x.try_borrow_mut().ok() {
                Some(x) => {
                    if x.player_id == player_id && x.role == EntityRole::Character {
                        Some(x)
                    } else {
                        None
                    }
                }
                None => None,
            })
    }

    pub(crate) fn add_or_replace_character_by_player_id(
        &mut self,
        player_id: NonZero<u64>,
        e: Entity,
    ) {
        match self.find_character_by_player_id_mut(player_id) {
            Some(mut entity) => return *entity = e,
            None => {}
        }
        self.entities.push(RefCell::new(e))
    }

    pub(crate) fn add_or_replace_by_id(&mut self, id: u32, e: Entity) {
        match self.find_by_id_mut(id) {
            Some(mut entity) => return *entity = e,
            None => {}
        }
        self.entities.push(RefCell::new(e))
    }

    pub(crate) fn find_by_id_mut<'a>(&'a self, id: u32) -> Option<RefMut<'a, Entity>> {
        self.entities
            .iter()
            .find_map(|x| match x.try_borrow_mut().ok() {
                Some(x) => {
                    if x.id == id {
                        Some(x)
                    } else {
                        None
                    }
                }
                None => None,
            })
    }

    pub(crate) fn proceed(&mut self, dt: Duration) {
        let now = Instant::now();
        self.entities.retain(|e| {
            let mut e = e.borrow_mut();
            match e.role {
                EntityRole::Character => true,
                EntityRole::Projectile => {
                    let velosity = 200.;

                    let motion_segment = Segment {
                        p0: e.pos,
                        p1: e.pos + Vector::polar(e.rot, velosity * 2. * dt.as_secs_f32()),
                    };

                    for (i, edge) in self.world_bounds.edges().into_iter().enumerate() {
                        if let Some(r) = edge.ray_cast(motion_segment) {
                            if r.intersects() {
                                match i {
                                    0 => {
                                        e.rot = Complex {
                                            r: e.rot.r,
                                            i: e.rot.i.abs(),
                                        }
                                    }
                                    1 => {
                                        e.rot = Complex {
                                            r: -e.rot.r.abs(),
                                            i: e.rot.i,
                                        }
                                    }
                                    2 => {
                                        e.rot = Complex {
                                            r: e.rot.r,
                                            i: -e.rot.i.abs(),
                                        }
                                    }
                                    3 => {
                                        e.rot = Complex {
                                            r: e.rot.r.abs(),
                                            i: e.rot.i,
                                        }
                                    }
                                    _ => panic!("Wups!!!"),
                                }
                                break;
                            }
                        }
                    }

                    e.pos = e.pos + Vector::polar(e.rot, velosity * dt.as_secs_f32());

                    now - e.birth_instant.unwrap() < Duration::from_secs(60)
                }
            }
        });

        for character in &self.entities {
            let character = character.borrow_mut();
            if character.role == EntityRole::Character {
                for projectile in &self.entities {
                    if let Ok(projectile) = projectile.try_borrow_mut() {
                        if projectile.role == EntityRole::Projectile
                            && (projectile.player_id != character.player_id
                                || (now - projectile.birth_instant.unwrap())
                                    > Duration::from_millis(200))
                        {
                            if (character.pos - projectile.pos).len()
                                < (character.inscribed_circle_radius()
                                    + projectile.inscribed_circle_radius())
                            {
                                self.kills.push(character.id);
                                self.kills.push(projectile.id);
                                break;
                            }
                        }
                    }
                }
            }
        }

        self.entities.retain(|e| {
            let e = e.borrow();
            if let Ok(index) = self.kills.binary_search_by_key(&e.id, |x| *x) {
                if e.role == EntityRole::Projectile {
                    self.kills.remove(index);
                    return false;
                }
            }
            true
        });
    }

    pub(crate) fn account_kill(&mut self, player_id: NonZero<u64>) -> bool {
        let orig_len = self.entities.len();
        self.entities.retain(|e| {
            let e = e.borrow();
            if let Ok(index) = self.kills.binary_search_by_key(&e.id, |x| *x) {
                if e.role == EntityRole::Character && e.player_id == player_id {
                    self.kills.remove(index);
                    return false;
                }
            }
            true
        });
        orig_len != self.entities.len()
    }

    pub(crate) fn lerp_merge(
        result: &mut Self,
        a: &Self,
        b: &Self,
        t: f64,
        ignore_with_player_id: NonZero<u64>,
    ) {
        for b in b.entities_mut() {
            if b.player_id == ignore_with_player_id {
                continue;
            }
            let a = a.find_by_id_mut(b.id);
            if let Some(a) = a {
                result.add_or_replace_by_id(b.id, Entity::lerp(a.clone(), b.clone(), t));
            } else {
                result.add_or_replace_by_id(b.id, b.clone());
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct PlayerState {
    pub(crate) color: Color,
    pub(crate) killed: bool,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            color: Color {
                a: 0,
                r: 0,
                g: 0,
                b: 0,
            },
            killed: Default::default(),
        }
    }
}
