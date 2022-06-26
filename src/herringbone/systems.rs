use super           	:: { * };
use crate				:: { bevy_spline };
use bevy_prototype_debug_lines :: { DebugLines };

pub fn brick_road_system(
	mut q_spline		: Query<(
							Entity,
							&Children,
							&GlobalTransform,
							&mut Spline,
							&mut HerringboneControl,
							&mut Herringbone2Config,
							&mut BrickRoadProgressState
						)>,//, Changed<HerringboneControl>>,

	mut despawn			: ResMut<DespawnResource>,
		time			: Res<Time>,
		ass				: Res<AssetServer>,

		q_tiles			: Query<Entity, With<Herringbone2>>,

	mut	meshes			: ResMut<Assets<Mesh>>,
	mut	materials		: ResMut<Assets<StandardMaterial>>,
	mut commands		: Commands,

	mut debug_lines		: ResMut<DebugLines>,
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
		if control.clean_tiles {
			for e in q_tiles.iter() {
				if !children_e.contains(&e) {
					continue;
				}
				despawn.entities.push(e);
			}
		
			tile_state.set_default();
		
			control.clean_tiles = false;
		}

		let do_spawn 	= control.spawn_tile || control.animate;
		if !do_spawn || tile_state.finished {
			continue;
		}

		let cur_time	= time.seconds_since_startup();
		if (cur_time - control.last_update) < control.anim_delay_sec && !control.instant {
			continue;
		}

		control.last_update = cur_time;

		let verbose = control.verbose;
		let looped = control.looped;
		let mut tiles_row_prev : Vec<TileRowIterState> = Vec::new();
		let mut tiles_row_cur : Vec<TileRowIterState> = Vec::new();

		if control.instant {
			if verbose {
				println!("\ninstant Herringbone2 road spawn started!");
			}

			let mut tiles_cnt = 0;
			while !tile_state.finished {
				spawn::brick_road_iter(&mut tile_state, &mut config, &mut spline, &mut tiles_row_prev, &mut tiles_row_cur, &transform, &ass, &mut sargs, &control, &mut debug_lines);
				tiles_cnt += 1;
			}

			if verbose {
				println!("total tiles: {}", tiles_cnt);
			}

			if !looped { 
				control.instant = false;
			}
		} else {
			spawn::brick_road_iter(&mut tile_state, &mut config, &mut spline, &mut tiles_row_prev, &mut tiles_row_cur, &transform, &ass, &mut sargs, &control, &mut debug_lines);

			if tile_state.finished {
				if looped {
					control.clean_tiles = true;
				} else {
					control.animate	= false;
				}
			}
		}

		if looped && control.spawn_tile == true && tile_state.finished {
			control.clean_tiles	= true;
		} else {
			control.spawn_tile	= false;
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
		control.respawn_all_tiles_instantly();
	}
}

pub fn on_spline_control_point_moved(
		time			: Res<Time>,
		q_controlp 		: Query<(&Parent, &ControlPoint), Changed<Transform>>,
	mut q_spline		: Query<&mut HerringboneControl>,
) {
	if time.seconds_since_startup() < 0.1 {
		return;
	}

	if q_spline.is_empty() {
		return;
	}

	for (spline_e, controlp) in q_controlp.iter() {
		let mut control = q_spline.get_mut(spline_e.0).unwrap();
		control.respawn_all_tiles_instantly();
	}
}