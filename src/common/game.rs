use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{
    cell::{Ref, RefCell, RefMut},
    collections::VecDeque,
    f32::consts::PI,
    num::NonZero,
    ops::{Deref, DerefMut},
    time::{Duration, Instant},
};

use super::{
    Collide as _, Complex, DynSizeSegments as _, Point, Rect, Segment, Segments, Vector, I,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct Color {
    pub(crate) a: u8,
    pub(crate) r: u8,
    pub(crate) g: u8,
    pub(crate) b: u8,
}

impl Color {
    pub(crate) fn with_a(self, a: u8) -> Self {
        Self {
            a,
            r: self.r,
            g: self.g,
            b: self.b,
        }
    }

    pub(crate) fn with_r(self, r: u8) -> Self {
        Self {
            a: self.a,
            r,
            g: self.g,
            b: self.b,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub(crate) struct Shield {
    pub(crate) width: f32,
    pub(crate) dst_from_character: f32,
}

impl Shield {
    pub(crate) fn segment(&self, character_pos: Point, character_rot: Complex) -> Segment {
        let c = character_pos + Vector::polar(character_rot, self.dst_from_character);
        Segment {
            p0: c + Vector::polar(character_rot * (-I), self.width / 2.),
            p1: c + Vector::polar(character_rot * I, self.width / 2.),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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
    Shield {
        shield: Shield,
        self_destruct_timeout: Duration,
    },
    MineGun {
        fire_interval: Duration,
        life_duration: Duration,
        owner_invincibility_duration: Duration,
        activation_duration: Duration,
        start_velocity: f32,
        acceleration: f32,
        radius: f32,
        detection_radius: f32,
        explosion_radius: f32,
        debris_kind: Box<ProjectileKind>,
        debris_count: u8,
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
    },
    Mine {
        life_duration: Duration,
        owner_invincibility_duration: Duration,
        activation_duration: Duration,
        velocity: f32,
        acceleration: f32,
        radius: f32,
        detection_radius: f32,
        explosion_radius: f32,
        debris_kind: Box<ProjectileKind>,
        debris_count: u8,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub(crate) enum EntityRole {
    Character { weapon: CharacterWeapon },
    Projectile { kind: ProjectileKind },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct EntityTail {
    pub(crate) end: Point,
    pub(crate) rotation: Complex,
    pub(crate) reflection_points: VecDeque<Point>,
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
    pub(crate) tail: Option<EntityTail>,
    pub(crate) activated: bool,
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
            tail: b.tail,
            activated: b.activated,
        }
    }

    pub(crate) fn inscribed_circle_radius(&self) -> f32 {
        match &self.role {
            EntityRole::Character { .. } => 8.,
            EntityRole::Projectile { kind } => match kind {
                ProjectileKind::Ball { radius, .. } => *radius,
                ProjectileKind::Ray { .. } => 2.,
                ProjectileKind::Mine { radius, .. } => *radius,
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
    pub(crate) tail: Option<EntityTail>,
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
                    ProjectileKind::Mine { .. } => 0,
                },
            },
            role: entity.role,
            tail: entity.tail,
            activated: false,
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
        shields: &[Segment],
        motion_segment: Segment,
    ) -> bool {
        for shield in shields {
            if let Some(r) = shield.ray_cast(motion_segment) {
                if r.intersects() {
                    *position = r.intersection_point_u_mul(0.99);
                    *rotation = rotation.reflect_from(shield.vec());
                    return true;
                }
            }
        }

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
        let mut create_infos: Vec<(EntityCreateInfo, NonZero<u64>)> = Default::default();

        let shields: Vec<Segment> = self
            .entities
            .iter()
            .map(|x| {
                let x = x.borrow();
                match &x.role {
                    EntityRole::Character { weapon } => match weapon {
                        CharacterWeapon::Shield { shield, .. } => {
                            Some(shield.segment(x.pos, x.rot))
                        }
                        _ => None,
                    },
                    _ => None,
                }
            })
            .flatten()
            .collect();

        self.entities.retain(|entity| -> bool {
            let mut entity = entity.borrow_mut();
            match entity.role.clone() {
                EntityRole::Character { .. } => true,
                EntityRole::Projectile { kind } => {
                    let velosity = match kind {
                        ProjectileKind::Ball { velocity, .. } => velocity,
                        ProjectileKind::Ray { velocity, .. } => velocity,
                        ProjectileKind::Mine { velocity, .. } => velocity,
                    };

                    let life_duration = match kind {
                        ProjectileKind::Ball { life_duration, .. } => life_duration,
                        ProjectileKind::Ray { life_duration, .. } => life_duration,
                        ProjectileKind::Mine { life_duration, .. } => life_duration,
                    };

                    fn step(
                        position: &mut Point,
                        rotation: &mut Complex,
                        reflection_points: Option<&mut VecDeque<Point>>,
                        tail: bool,
                        bounds: &Rect,
                        shields: &[Segment],
                        velosity: f32,
                        dt: &Duration,
                    ) {
                        let motion_segment = Segment {
                            p0: *position,
                            p1: *position
                                + Vector::polar(*rotation, velosity * 2. * dt.as_secs_f32()),
                        };
                        if GameState::reflect(position, rotation, bounds, shields, motion_segment) {
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
                                ..
                            } => {
                                let tail = entity.tail.as_mut().unwrap();

                                if now - entity.birth_instant.unwrap() < *life_duration {
                                    step(
                                        &mut entity.pos,
                                        &mut entity.rot,
                                        Some(&mut tail.reflection_points),
                                        false,
                                        &self.world_bounds,
                                        &shields,
                                        velosity,
                                        &dt,
                                    );
                                }

                                if now - entity.birth_instant.unwrap() > *tail_freeze_duration {
                                    step(
                                        &mut tail.end,
                                        &mut tail.rotation,
                                        Some(&mut tail.reflection_points),
                                        true,
                                        &self.world_bounds,
                                        &shields,
                                        velosity,
                                        &dt,
                                    );
                                }
                                now - entity.birth_instant.unwrap()
                                    < (*life_duration + *tail_freeze_duration)
                            }
                            ProjectileKind::Mine {
                                life_duration,
                                owner_invincibility_duration,
                                activation_duration,
                                velocity,
                                acceleration,
                                radius,
                                detection_radius,
                                explosion_radius,
                                debris_kind,
                                debris_count,
                            } => {
                                entity.activated =
                                    now - entity.birth_instant.unwrap() > *activation_duration;

                                let new_velocity = *velocity + *acceleration * dt.as_secs_f32();
                                if velocity.signum() == new_velocity.signum() {
                                    *velocity = new_velocity;
                                }

                                step(
                                    &mut entity.pos,
                                    &mut entity.rot,
                                    None,
                                    false,
                                    &self.world_bounds,
                                    &shields,
                                    velosity,
                                    &dt,
                                );

                                now - entity.birth_instant.unwrap() < *life_duration
                            }
                            _ => {
                                step(
                                    &mut entity.pos,
                                    &mut entity.rot,
                                    None,
                                    false,
                                    &self.world_bounds,
                                    &shields,
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
                            let mut acc: Option<Vector> = None;
                            match & projectile.role {
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
                                        ..
                                    } => {
                                        let tail = projectile.tail.as_ref().unwrap();

                                        if projectile.player_id != character.player_id
                                            || (now - projectile.birth_instant.unwrap())
                                                > *owner_invincibility_duration
                                        {
                                            if character.health != 0 && projectile.health != 0 {
                                                let projectile_trace: Vec<_> = [tail.end]
                                                    .into_iter()
                                                    .chain(
                                                        tail.reflection_points.clone().into_iter(),
                                                    )
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
                                    ProjectileKind::Mine {
                                        life_duration,
                                        owner_invincibility_duration,
                                        activation_duration,
                                        velocity,
                                        acceleration,
                                        radius,
                                        detection_radius,
                                        explosion_radius,
                                        debris_kind,
                                        debris_count,
                                    } => {
                                        if projectile.activated
                                            && (projectile.player_id != character.player_id
                                                || (now - projectile.birth_instant.unwrap())
                                                    > *owner_invincibility_duration)
                                        {
                                            if (character.pos - projectile.pos).len()
                                            < (character.inscribed_circle_radius()
                                                + *detection_radius) {
                                                    acc = Some((character.pos - projectile.pos).normalize() * -2. * *acceleration * dt.as_secs_f32());
                                                }


                                            if (character.pos - projectile.pos).len()
                                                < (character.inscribed_circle_radius()
                                                    + explosion_radius)
                                            {
                                                for i in 0..*debris_count {
                                                    let rot = Complex::from_rad(
                                                        (i as f32 / *debris_count as f32) * 2. * PI,
                                                    );
                                                    create_infos.push((
                                                        EntityCreateInfo {
                                                            pos: projectile.pos,
                                                            rot,
                                                            color: projectile.color.clone(),
                                                            role: EntityRole::Projectile {
                                                                kind: debris_kind.deref().clone(),
                                                            },

                                                            tail: Some(EntityTail {
                                                                end: projectile.pos,
                                                                rotation: rot,
                                                                reflection_points: Default::default(
                                                                ),
                                                            }),
                                                        },
                                                        projectile.player_id,
                                                    ));
                                                }

                                                self.kills.push(projectile.id);
                                                break;
                                            }
                                        }
                                    }
                                },
                                _ => {}
                            }
                            if let Some(acc) = acc {
                            projectile.pos += acc;
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

        for (create_info, player_id) in create_infos {
            self.create(create_info, player_id);
        }
    }

    pub(crate) fn register_kill(&mut self, id: u32) {
        assert!(!self.kills.contains(&id));
        self.kills.push(id);
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
