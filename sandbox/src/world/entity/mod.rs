use std::collections::VecDeque;
use crate::tools::*;
use crate::tools::asset::AssetPool;
use crate::tools::phys::Physics;
use crate::world::block::{BlockSide, ChunkLocation, ChunkMap};
use crate::world::entity::render::EntityRenderer;
use crate::world::{FrameEvent, TickEvent};
use crate::world::item::Item;

pub mod types;
pub mod render;

pub trait Entity {
    fn entity_type(&self) -> &'static str;

    fn uuid(&self) -> Uuid;

    fn position(&self) -> Vector<f32, 2>;

    fn attach_collision(&mut self, physics: &mut Physics) {
        // Do nothing by default
        let _ = physics;
    }

    fn detach_collision(&mut self, physics: &mut Physics) {
        // Do nothing by default
        let _ = physics;
    }

    fn attach_appearance(&mut self, assets: &mut AssetPool, renderer: &mut EntityRenderer) {
        // Do nothing by default
        let _ = (assets, renderer);
    }

    fn detach_appearance(&mut self, renderer: &mut EntityRenderer) {
        // Do nothing by default
        let _ = renderer;
    }

    fn run_frame(&mut self, dt: f32, physics: &mut Physics, renderer: &mut EntityRenderer, responses: &mut EntityResponseQueue) {
        // Do nothing by default
        let _ = (dt, physics, renderer, responses);
    }

    fn tick(&mut self, chunks: &ChunkMap, responses: &mut EntityResponseQueue) {
        // Do nothing by default
        let _ = (chunks, responses);
    }

    fn set_held_item(&mut self, item: Item, responses: &mut EntityResponseQueue) {
        // Do nothing by default
        let _ = (item, responses);
    }

    fn update(&mut self, request: EntityRequest, responses: &mut EntityResponseQueue) {
        match request {
            EntityRequest::RunFrame { dt, physics, renderer } => {
                self.run_frame(dt, physics, renderer, responses);
            }
            EntityRequest::Tick { chunks } => {
                self.tick(chunks, responses);
            }
            EntityRequest::SetHeldItem(item) => {
                self.set_held_item(item, responses);
            }
        }
    }
}

pub enum EntityRequest<'a> {
    RunFrame {
        dt: f32,
        physics: &'a mut Physics,
        renderer: &'a mut EntityRenderer,
    },
    Tick {
        chunks: &'a ChunkMap,
    },
    SetHeldItem(Item),
}

pub enum EntityResponse {
    FrameEvent(FrameEvent),
    TickEvent(TickEvent),
    Die,
    UseHeldItem {
        item: Item,
        chunk_location: ChunkLocation,
        block_x: usize,
        block_y: usize,
        side: BlockSide,
    },
    DestroyBlock {
        chunk_location: ChunkLocation,
        block_x: usize,
        block_y: usize,
    },
}

pub type EntityResponseQueue = VecDeque<(Uuid, EntityResponse)>;
