use bevy::prelude::*;

use crate::world::Level;

use super::{UseItemEvent, ItemType};

pub fn use_block_item(
    mut reader: EventReader<UseItemEvent>,
    level: Res<Level>,
    mut commands: Commands,
) {
    for event in reader.iter() {
        if let ItemType::Block(block_type) = event.1 {
            if let Some(hit) = level.blockcast(event.2.translation(), event.2.forward()*10.0) {
                level.set_block(hit.block_pos+hit.normal, block_type, &mut commands);
            }
        }
    }
}