use bevy			:: prelude :: *;
use iyes_loopless	:: prelude :: { * };

mod egui_ext;
pub use egui_ext    :: FileDialog;

use crate           :: Game :: *;

mod systems;
use systems		    :: { * };

mod draw;
mod writeback;

pub struct UiPlugin;

// This plugin is responsible to control the game audio
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system	(vehicle_params_ui_system.run_in_state(GameMode::Editor))
            .add_system	(coords_on_hover_ui_system.run_in_state(GameMode::Editor))
            ;
    }
}