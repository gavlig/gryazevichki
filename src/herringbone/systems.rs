use bevy_polyline		:: { prelude :: * };
use bevy_prototype_debug_lines :: { DebugLines };

use super           	:: { * };
use crate				:: { bevy_spline };

pub fn brick_road_system(
	mut q_spline		: Query<(Entity, &Children, &GlobalTransform, &mut Spline, &mut HerringboneControl, &mut Herringbone2Config, &mut TileState), Changed<HerringboneControl>>,

	mut despawn			: ResMut<DespawnResource>,
		time			: Res<Time>,

		q_tiles			: Query<Entity, With<Herringbone2>>,

	mut	meshes			: ResMut<Assets<Mesh>>,
	mut	materials		: ResMut<Assets<StandardMaterial>>,
	mut commands		: Commands
) {
	if q_spline.is_empty() {
		return;
	}

	let mut sargs = SpawnArguments {
		meshes		: &mut meshes,
		materials	: &mut materials,
		commands	: &mut commands,
	};

	for (root_e, children_e, transform, mut spline, mut control, mut config, mut tile_state) in q_spline.iter_mut() {
		if control.reset {
			for e in q_tiles.iter() {
				if !children_e.contains(&e) {
					continue;
				}
				despawn.entities.push(e);
			}
		
			tile_state.reset_changed();
		
			control.reset = false;
		}

		let do_spawn 	= control.next || control.animate;
		if !do_spawn || tile_state.finished {
			return;
		}

		let cur_time	= time.seconds_since_startup();
		if (cur_time - control.last_update) < control.anim_delay_sec && !control.instant {
			return;
		}

		control.last_update = cur_time;

		if !control.instant {
			// spawn::brick_road_iter(&mut tile_state, &mut config, &mut spline, control.debug, &ass, &mut sargs);
		} else {
			let mut tiles_cnt = 0;
			// while !tile_state.finished {// && ((tiles_cnt < 36 && control.debug) || !control.debug) {
				// spawn::brick_road_iter(&mut tile_state, &mut config, &mut spline, control.debug, &ass, &mut sargs);
				// tiles_cnt += 1;
			// }
			// println!("total tiles: {}", tiles_cnt);
			control.instant = false;
		}

		control.next	= false;
		if tile_state.finished {
			control.animate	= false;
		}
	}
}

pub fn on_spline_tangent_moved(
		time			: Res<Time>,
		q_tangent_parent: Query<&Parent, (With<Tangent>, Changed<Transform>)>,
		q_controlp_parent : Query<&Parent, With<ControlPoint>>, // <- parent of this ^
	mut q_control		: Query<&mut HerringboneControl>, // <- parent of this ^
) {
	if time.seconds_since_startup() < 0.1 {
		return;
	}

	if q_control.is_empty() {
		return;
	}

	for control_point_e in q_tangent_parent.iter() {
		let control_e 	= q_controlp_parent.get(control_point_e.0).unwrap();
		let mut control	= q_control.get_mut(control_e.0).unwrap();

		control.reset 	= true;
		control.next 	= true;
		control.instant = true;
	}
}

pub fn on_spline_control_point_moved(
		time			: Res<Time>,
		q_controlp 		: Query<(&Parent, &ControlPoint), Changed<Transform>>,
	mut q_spline		: Query<(&Spline, &mut HerringboneControl)>,
) {
	if time.seconds_since_startup() < 0.1 {
		return;
	}

	if q_spline.is_empty() {
		return;
	}

	for (spline_e, controlp) in q_controlp.iter() {
		let (spline, mut control) = q_spline.get_mut(spline_e.0).unwrap();

		match controlp {
			ControlPoint::T(_) => {
				control.reset	= true;
				control.next 	= true;
				control.instant = true;
			},
		}
	}
}