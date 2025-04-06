use serde::{Deserialize, Serialize};
use std::{
    cell::{Ref, RefCell, RefMut},
    num::NonZero,
};

use super::{Complex, Point};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct Color {
    pub(crate) a: u8,
    pub(crate) r: u8,
    pub(crate) g: u8,
    pub(crate) b: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct Entity {
    pub(crate) id: u32,
    pub(crate) player_id: NonZero<u64>,
    pub(crate) pos: Point,
    pub(crate) rot: Complex,
    pub(crate) color: Color,
}

impl Entity {
    pub(crate) fn lerp(a: Self, b: Self, t: f64) -> Self {
        Entity {
            id: b.id,
            player_id: b.player_id,
            pos: Point::lerp(a.pos, b.pos, t),
            rot: Complex::lerp(a.rot, b.rot, t),
            color: b.color,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct EntityCreateInfo {
    pub(crate) pos: Point,
    pub(crate) rot: Complex,
    pub(crate) color: Color,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct GameState {
    entities: Vec<RefCell<Entity>>,
    next_entity_id: u32,
}

impl GameState {
    pub(crate) fn new() -> Self {
        Self {
            entities: vec![],
            next_entity_id: 0,
        }
    }

    pub(crate) fn create(&mut self, entity: EntityCreateInfo, player_id: NonZero<u64>) {
        self.entities.push(RefCell::new(Entity {
            id: self.next_entity_id,
            player_id,
            pos: entity.pos,
            rot: entity.rot,
            color: entity.color,
        }));
        self.next_entity_id += 1;
    }

    pub(crate) fn entities<'a>(&'a self) -> impl Iterator<Item = Ref<'a, Entity>> {
        self.entities.iter().filter_map(|x| x.try_borrow().ok())
    }

    pub(crate) fn entities_mut<'a>(&'a self) -> impl Iterator<Item = RefMut<'a, Entity>> {
        self.entities.iter().filter_map(|x| x.try_borrow_mut().ok())
    }

    pub(crate) fn find_by_player_id_mut<'a>(
        &'a self,
        player_id: NonZero<u64>,
    ) -> Option<RefMut<'a, Entity>> {
        self.entities
            .iter()
            .find_map(|x| match x.try_borrow_mut().ok() {
                Some(x) => {
                    if x.player_id == player_id {
                        Some(x)
                    } else {
                        None
                    }
                }
                None => None,
            })
    }

    pub(crate) fn add_or_replace_by_player_id(&mut self, player_id: NonZero<u64>, e: Entity) {
        match self.find_by_player_id_mut(player_id) {
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
            // let r = result.find_by_id_mut(b.id);

            // println!(")))))");

            if let Some(a) = a {
                // println!("AAA");
                result.add_or_replace_by_id(b.id, Entity::lerp(a.clone(), b.clone(), t));
            } else {
                // println!("BBB");
                result.add_or_replace_by_id(b.id, b.clone());
            }

            // match (a, r) {
            //     (Some(a), Some(r)) => {
            //         println!("AAAA");
            //         *r = Entity::lerp(a.clone(), b.clone(), t)
            //     },
            //     (None, Some(a)) => ;
            //     b.clone()},
            //     _ => {}
            // }
        }
    }
}
