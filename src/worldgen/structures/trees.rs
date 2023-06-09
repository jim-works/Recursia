use std::f32::consts::PI;

use bracket_noise::prelude::{FastNoise, NoiseType};

use crate::{world::{chunk::*, BlockBuffer, BlockCoord, BlockChange, BlockId, BlockRegistry, BlockName}, util::l_system::{LSystem, TreeAlphabet}};
use bevy::prelude::*;
use super::StructureGenerator;

//uses an L-system (https://en.wikipedia.org/wiki/L-system) to generate a tree
pub struct LTreeGenerator<P: Fn(&TreeAlphabet, u32) -> Option<Vec<TreeAlphabet>>, I: Fn(BlockCoord) -> Vec<TreeAlphabet>, B: Fn(&mut BlockBuffer<BlockId>, Vec3, Vec3), L: Fn(&mut BlockBuffer<BlockId>, BlockCoord)> {
    l_system: LSystem<TreeAlphabet, P>,
    iterations: u32,
    initial_sentence: I,
    block_placer: B,
    leaf_placer: L,
    spawnable_block: BlockId,
    rng: FastNoise,
}
impl<P: Fn(&TreeAlphabet, u32) -> Option<Vec<TreeAlphabet>>, I: Fn(BlockCoord) -> Vec<TreeAlphabet>, B: Fn(&mut BlockBuffer<BlockId>, Vec3, Vec3), L: Fn(&mut BlockBuffer<BlockId>, BlockCoord)> StructureGenerator for LTreeGenerator<P,I,B,L> {
    fn rarity(&self) -> f32 {
        1.0
    }

    fn generate(
        &self,
        buffer: &mut BlockBuffer<BlockId>,
        pos: BlockCoord,
        local_pos: ChunkIdx,
        chunk: &GeneratingChunk
    ) {
        let _my_span = info_span!("tree_validate", name = "tree_validate").entered();
        //determine if location is suitable for a tree
        if chunk[local_pos] != self.spawnable_block {
            return;
        }
        for y in (local_pos.y + 1)..CHUNK_SIZE_U8 {
            if !matches!(chunk[ChunkIdx::new(local_pos.x, y, local_pos.z)], BlockId(crate::world::Id::Empty)) {
                return;
            }
        }
        let _my_span = info_span!("tree_generate", name = "tree_generate").entered();
        //generate tree
        let vec3_pos = pos.to_vec3();
        let seed = self.rng.get_noise3d(vec3_pos.x,vec3_pos.y,vec3_pos.z).to_bits();
        let tree = self.l_system.iterate(&(self.initial_sentence)(pos), self.iterations, seed);
        //place tree
        let mut heads = Vec::new();
        let mut curr_head = Transform::from_translation(pos.to_vec3());
        for instruction in tree {
            match instruction {
                TreeAlphabet::Move(v) => {
                    let old_pos = curr_head.translation;
                    curr_head.translation += curr_head.forward()*v;
                    (self.block_placer)(buffer, old_pos, curr_head.translation);
                },
                TreeAlphabet::Rotate(r) => {
                    curr_head.rotate(r)
                },
                TreeAlphabet::StartBranch => {
                    heads.push(curr_head);
                },
                TreeAlphabet::EndBranch => {
                    if let Some(h) = heads.pop() {
                        curr_head = h;
                    } else {
                        warn!("Branch end mismatch in L-tree at blockcoord: {:?}", pos);
                    }
                },
                TreeAlphabet::Replace(_) => {
                    (self.leaf_placer)(buffer, curr_head.translation.into());
                }
            }
        }
    }
}
impl<P: Fn(&TreeAlphabet, u32) -> Option<Vec<TreeAlphabet>>, I: Fn(BlockCoord) -> Vec<TreeAlphabet>, B: Fn(&mut BlockBuffer<BlockId>, Vec3, Vec3), L: Fn(&mut BlockBuffer<BlockId>, BlockCoord)> LTreeGenerator<P,I,B,L> {
    pub fn new(rng: FastNoise, system: LSystem<TreeAlphabet, P>, iterations: u32, initial_sentence: I, block_placer: B, leaf_placer: L, spawnable_block: BlockId) -> Self {
        LTreeGenerator {
            rng,
            l_system: system,
            iterations,
            initial_sentence,
            block_placer,
            leaf_placer,
            spawnable_block
        }
    }
}

pub fn get_short_tree(seed: u64, registry: &BlockRegistry) -> Box<dyn StructureGenerator+Send+Sync> {
    let mut noise = FastNoise::seeded(seed);
    //white noise doesn't work
    noise.set_noise_type(NoiseType::Value);
    noise.set_frequency(436_781.25);
    let wood = registry.get_id(&BlockName::core("log"));
    let leaves = registry.get_id(&BlockName::core("leaves"));
    let grass = registry.get_id(&BlockName::core("grass"));
    let mut selection_noise = FastNoise::seeded(seed+1);
    selection_noise.set_noise_type(NoiseType::Value);
    selection_noise.set_frequency(1_230_481.1);
    Box::new(LTreeGenerator::new(
            noise,
        LSystem::new(move |x,idx| {
            //use random sample to select which production to use
            const RANGE_SPLIT: u32 = 1<<16;//idx will use the whole u32 range, and I ran into fp precision issues. so we split to use x and y axes of noise function
            let sample_x = (idx/RANGE_SPLIT) as f32*0.01;
            let sample_y = (idx%RANGE_SPLIT) as f32*0.01;
            let random = selection_noise.get_noise(sample_x, sample_y);
            const OPTIONS: usize = 2;
            fn get_moves(idx: usize, x: f32) -> Vec<TreeAlphabet> {
                let forward = TreeAlphabet::Move(x*0.5);
                let replace = TreeAlphabet::Replace(x*0.5);
                let rotate1 = TreeAlphabet::Rotate(Quat::from_euler(EulerRot::XYZ, PI/6.0,0.0,PI/6.0));
                let rotate2 = TreeAlphabet::Rotate(Quat::from_euler(EulerRot::XYZ, PI/6.0,0.0,-PI/6.0));
                let rotate3 = TreeAlphabet::Rotate(Quat::from_euler(EulerRot::XYZ, -PI/6.0,0.0,-PI/6.0));
                let rotate4 = TreeAlphabet::Rotate(Quat::from_euler(EulerRot::XYZ, -PI/6.0,0.0,0.0));
                let rotate5 = TreeAlphabet::Rotate(Quat::from_euler(EulerRot::XYZ, 0.0,0.0,-PI/6.0));
                match idx.min(OPTIONS-1) {
                    0 => vec![TreeAlphabet::Move(x),
                            rotate1,
                            TreeAlphabet::StartBranch,
                            forward,
                            rotate4,
                            replace,
                            TreeAlphabet::EndBranch,
                            rotate3,
                            TreeAlphabet::StartBranch,
                            replace,
                            rotate2,
                            forward,
                            replace,
                            TreeAlphabet::EndBranch,
                            replace,
                        ],
                    1 => vec![
                            rotate5,
                            replace,
                            TreeAlphabet::StartBranch,
                            forward,
                            rotate4,
                            replace,
                            TreeAlphabet::EndBranch,
                            replace,
                    ],
                    2 => vec![
                        rotate1,
                        forward,
                        replace,
                        TreeAlphabet::StartBranch,
                        rotate5,
                        replace,
                        TreeAlphabet::EndBranch,
                        replace
                    ],
                    3 => vec![
                        forward,
                        TreeAlphabet::StartBranch,
                        rotate2,
                        replace,
                        TreeAlphabet::EndBranch,
                        rotate3,
                        replace,
                        TreeAlphabet::StartBranch,
                        rotate4,
                        replace,
                        TreeAlphabet::EndBranch
                    ],
                    _ => unreachable!()
                }
            }
            match x {
            TreeAlphabet::Replace(x) => {
                //map from (-1,1) to (0,OPTIONS)
                let selected = ((random+1.0)*0.5*OPTIONS as f32) as usize;    
                Some(get_moves(selected, *x))
            }
            _ => None
        }}),
        3,
        |_| vec![TreeAlphabet::Rotate(Quat::from_euler(EulerRot::XYZ, PI*0.5, 0.0, 0.0)), TreeAlphabet::Move(5.0), TreeAlphabet::Replace(10.0)],
        move |p,a,b| p.place_descending(BlockChange::Set(wood), a.into(), b.into()),
        move |buffer, pos| {
                    const LEAF_SIZE: i32 = 2;
                    for x in -LEAF_SIZE..LEAF_SIZE+1 {
                        for y in -LEAF_SIZE..LEAF_SIZE+1 {
                            for z in -LEAF_SIZE..LEAF_SIZE+1 {
                                if x*x+y*y+z*z < LEAF_SIZE*LEAF_SIZE+1 {
                                    buffer.set(BlockCoord::new(x,y,z)+pos, BlockChange::SetIfEmpty(leaves));
                                }
                            }
                        }
                    }
        },
        grass
    ))
}