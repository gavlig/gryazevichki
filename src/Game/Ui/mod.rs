use bevy			::	prelude :: *;
use bevy_egui		::	egui :: { Slider, Ui };
use bevy_egui		::	{ egui, EguiContext };
use bevy_mod_raycast::  { * };

mod egui_ext;
pub use egui_ext	::	FileDialog;
	use egui_ext	::	toggle_switch;

use super			::	{ * };
use super			::	Vehicle;
use super			::	Vehicle :: *;

mod systems;
pub use systems		::	{ * };

mod draw;
mod writeback;

pub struct HoverState {
	// pub pos			: Vec2,
	pub entity		: Entity,
	pub active		: bool,
}

impl Default for HoverState {
	fn default() -> Self {
		Self {
			// pos		: Vec2::ZERO,
			entity	: Entity::from_raw(0),
			active	: false,
		}
	}
}