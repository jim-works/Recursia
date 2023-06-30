pub mod styles;
pub mod inventory;
pub mod healthbar;
pub mod crosshair;
pub mod state;
pub mod debug;

use bevy::prelude::*;


pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_state::<state::UIState>()
            .add_startup_system(styles::init)
            .add_plugin(inventory::InventoryPlugin)
            .add_plugin(crosshair::CrosshairPlugin)
            .add_plugin(healthbar::HealthbarPlugin)
            .add_plugin(debug::DebugUIPlugin)
        ;
    }
}