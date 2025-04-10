use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{
    cell::{Ref, RefCell, RefMut},
    collections::VecDeque,
    num::NonZero,
    ops::DerefMut,
    time::{Duration, Instant},
};

use super::{Collide as _, Complex, DynSizeSegments as _, Point, Rect, Segment, Segments, Vector};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct Color {
    pub(crate) a: u8,
    pub(crate) r: u8,
    pub(crate) g: u8,
    pub(crate) b: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub(crate) enum CharacterWeapon {
    BallGun {
        life_duration: Duration,
        owner_invincibility_duration: Duration,
        fire_interval: Duration,
        velocity: f32,
        projectile_health: u8,
        radius: f32,
    },
    RayGun {
        life_duration: Duration,
        owner_invincibility_duration: Duration,
        tail_freeze_duration: Duration,
        fire_interval: Duration,
        velocity: f32,
        projectile_health: u8,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub(crate) enum ProjectileKind {
    Ball {
        life_duration: Duration,
        owner_invincibility_duration: Duration,
        velocity: f32,
        health: u8,
        radius: f32,
    },
    Ray {
        life_duration: Duration,
        owner_invincibility_duration: Duration,
        tail_freeze_duration: Duration,
        velocity: f32,
        health: u8,

        tail: Point,
        tail_rotation: Complex,
        reflection_points: VecDeque<Point>,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub(crate) enum EntityRole {
    Character { weapon: CharacterWeapon },
    Projectile { kind: ProjectileKind },
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
    pub(crate) health: u8,
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
            health: b.health,
        }
    }

    pub(crate) fn inscribed_circle_radius(&self) -> f32 {
        match &self.role {
            EntityRole::Character { .. } => 8.,
            EntityRole::Projectile { kind } => match kind {
                ProjectileKind::Ball { radius, .. } => *radius,
                ProjectileKind::Ray { .. } => 2.,
            },
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
                y: 32. + 16.,
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
            health: match &entity.role {
                EntityRole::Character { .. } => 3,
                EntityRole::Projectile { kind } => match kind {
                    ProjectileKind::Ball { health, .. } => *health,
                    ProjectileKind::Ray { health, .. } => *health,
                },
            },
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
                Some(x) => match x.role {
                    EntityRole::Character { .. } if x.player_id == player_id => Some(x),
                    _ => None,
                },
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

    fn reflect(
        position: &mut Point,
        rotation: &mut Complex,
        bounds: &Rect,
        motion_segment: Segment,
    ) -> bool {
        for (i, edge) in bounds.edges().into_iter().enumerate() {
            if let Some(r) = edge.ray_cast(motion_segment) {
                if r.intersects() {
                    *position = r.intersection_point();
                    match i {
                        0 => {
                            *rotation = Complex {
                                r: rotation.r,
                                i: rotation.i.abs(),
                            }
                        }
                        1 => {
                            *rotation = Complex {
                                r: -rotation.r.abs(),
                                i: rotation.i,
                            }
                        }
                        2 => {
                            *rotation = Complex {
                                r: rotation.r,
                                i: -rotation.i.abs(),
                            }
                        }
                        3 => {
                            *rotation = Complex {
                                r: rotation.r.abs(),
                                i: rotation.i,
                            }
                        }
                        _ => panic!("Wups!!!"),
                    }
                    return true;
                }
            }
        }
        false
    }

    pub(crate) fn proceed(&mut self, dt: Duration) {
        let now = Instant::now();
        self.entities.retain(|entity| {
            let mut entity = entity.borrow_mut();
            match entity.role.clone() {
                EntityRole::Character { .. } => true,
                EntityRole::Projectile { kind } => {
                    let velosity = match kind {
                        ProjectileKind::Ball { velocity, .. } => velocity,
                        ProjectileKind::Ray { velocity, .. } => velocity,
                    };

                    let life_duration = match kind {
                        ProjectileKind::Ball { life_duration, .. } => life_duration,
                        ProjectileKind::Ray { life_duration, .. } => life_duration,
                    };

                    fn step(
                        position: &mut Point,
                        rotation: &mut Complex,
                        reflection_points: Option<&mut VecDeque<Point>>,
                        tail: bool,
                        bounds: &Rect,
                        velosity: f32,
                        dt: &Duration,
                    ) {
                        let motion_segment = Segment {
                            p0: *position,
                            p1: *position
                                + Vector::polar(*rotation, velosity * 2. * dt.as_secs_f32()),
                        };
                        if GameState::reflect(position, rotation, bounds, motion_segment) {
                            if let Some(reflection_points) = reflection_points {
                                if tail {
                                    reflection_points.pop_front();
                                } else {
                                    reflection_points.push_back(*position);
                                }
                            }
                        } else {
                            *position =
                                *position + Vector::polar(*rotation, velosity * dt.as_secs_f32());
                        }
                    }

                    let entity = entity.deref_mut();
                    match &mut entity.role {
                        EntityRole::Projectile { kind } => match kind {
                            ProjectileKind::Ray {
                                life_duration,
                                tail_freeze_duration,
                                tail,
                                tail_rotation,
                                reflection_points,
                                ..
                            } => {
                                if now - entity.birth_instant.unwrap() < *life_duration {
                                    step(
                                        &mut entity.pos,
                                        &mut entity.rot,
                                        Some(reflection_points),
                                        false,
                                        &self.world_bounds,
                                        velosity,
                                        &dt,
                                    );
                                }

                                if now - entity.birth_instant.unwrap() > *tail_freeze_duration {
                                    step(
                                        tail,
                                        tail_rotation,
                                        Some(reflection_points),
                                        true,
                                        &self.world_bounds,
                                        velosity,
                                        &dt,
                                    );
                                }
                                now - entity.birth_instant.unwrap()
                                    < (*life_duration + *tail_freeze_duration)
                            }
                            _ => {
                                step(
                                    &mut entity.pos,
                                    &mut entity.rot,
                                    None,
                                    false,
                                    &self.world_bounds,
                                    velosity,
                                    &dt,
                                );
                                now - entity.birth_instant.unwrap() < life_duration
                            }
                        },
                        _ => true,
                    }
                }
            }
        });

        for character in &self.entities {
            let mut character = character.borrow_mut();
            match character.role {
                EntityRole::Character { .. } => {
                    for projectile in &self.entities {
                        if let Ok(mut projectile) = projectile.try_borrow_mut() {
                            match &projectile.role {
                                EntityRole::Projectile { kind } => match kind {
                                    ProjectileKind::Ball {
                                        owner_invincibility_duration,
                                        ..
                                    } => {
                                        if projectile.player_id != character.player_id
                                            || (now - projectile.birth_instant.unwrap())
                                                > *owner_invincibility_duration
                                        {
                                            if character.health != 0
                                                && projectile.health != 0
                                                && (character.pos - projectile.pos).len()
                                                    < (character.inscribed_circle_radius()
                                                        + projectile.inscribed_circle_radius())
                                            {
                                                character.health -= 1;
                                                projectile.health -= 1;

                                                if character.health == 0 {
                                                    self.kills.push(character.id);
                                                }

                                                if projectile.health == 0 {
                                                    self.kills.push(projectile.id);
                                                }
                                                break;
                                            }
                                        }
                                    }
                                    ProjectileKind::Ray {
                                        owner_invincibility_duration,
                                        tail,
                                        reflection_points,
                                        ..
                                    } => {
                                        if projectile.player_id != character.player_id
                                            || (now - projectile.birth_instant.unwrap())
                                                > *owner_invincibility_duration
                                        {
                                            if character.health != 0 && projectile.health != 0 {
                                                let projectile_trace: Vec<_> = [*tail]
                                                    .into_iter()
                                                    .chain(reflection_points.clone().into_iter())
                                                    .chain([projectile.pos])
                                                    .collect();

                                                if projectile_trace.segments().any(|seg| {
                                                    [seg]
                                                        .collide(
                                                            &character.vertices().segments_ringe(),
                                                        )
                                                        .is_some()
                                                }) {
                                                    character.health -= 1;
                                                    projectile.health -= 1;

                                                    if character.health == 0 {
                                                        self.kills.push(character.id);
                                                    }

                                                    if projectile.health == 0 {
                                                        self.kills.push(projectile.id);
                                                    }
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                },
                                _ => {}
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        self.entities.retain(|e| {
            let e = e.borrow();
            if let Some(index) = self.kills.iter().position(|x| *x == e.id) {
                match e.role {
                    EntityRole::Projectile { .. } => {
                        self.kills.remove(index);
                        return false;
                    }
                    _ => {}
                }
            }
            true
        });
    }

    pub(crate) fn account_kill(&mut self, player_id: NonZero<u64>) -> bool {
        let orig_len = self.entities.len();
        self.entities.retain(|e| {
            let e = e.borrow();
            match e.role {
                EntityRole::Character { .. } if e.player_id == player_id => {
                    if let Some(index) = self.kills.iter().position(|x| *x == e.id) {
                        self.kills.remove(index);
                        return false;
                    }
                }
                _ => {}
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
