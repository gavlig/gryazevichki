use super           	:: { * };
use crate				:: { bevy_spline };
use bevy_prototype_debug_lines :: { DebugLines };

pub fn brick_road_system(
	mut q_spline		: Query<(
							&Children,
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

	for (children_e, mut spline, mut control, mut config, mut progress_state) in q_spline.iter_mut() {
		// a little logging helper lambda
		let verbose = control.verbose;
		let looped 	= control.looped;
		let log = |str_in : String| {
			if verbose {
				println!	("{}", str_in);
			}
		};

		if control.clean_tiles {
			for e in q_tiles.iter() {
				if !children_e.contains(&e) {
					continue;
				}
				despawn.entities.push(e);
			}
		
			progress_state.set_default();
		
			control.clean_tiles = false;
		}

		let do_spawn 	= control.spawn_tile || control.animate;
		if !do_spawn || progress_state.finished {
			continue;
		}

		let cur_time	= time.seconds_since_startup();
		if (cur_time - control.last_update) < control.anim_delay_sec && !control.instant {
			continue;
		}

		control.last_update = cur_time;

		// do a dry run first to calculate max/min_spline_offset
		if progress_state.hasnt_started() {
			let mut dry_run_control = control.clone();
			let mut progress_state_clone = progress_state.clone();
			dry_run_control.dry_run = true;

			while progress_state_clone.column_id == 0 {
				spawn::brick_road_iter(&mut progress_state_clone, &mut config, &mut spline, &ass, &mut sargs, &dry_run_control, &mut debug_lines);
			}

			progress_state.max_spline_offset = progress_state_clone.max_spline_offset;
			progress_state.min_spline_offset = progress_state_clone.min_spline_offset;

			let column_offset	= spawn::calc_init_column_offset(&progress_state, &config);
			progress_state.pos	= Vec3::Y * 0.5 + Vec3::X * column_offset; // VERTICALITY

			log(format!("Herringbone2 road initialized! max_spline_offset: {:.3} min_spline_offset: {:.3}", progress_state.max_spline_offset, progress_state.min_spline_offset));
		}

		if control.instant {
			log(format!("\ninstant Herringbone2 road spawn started!"));

			let mut tiles_cnt = 0;
			while !progress_state.finished {
				spawn::brick_road_iter(&mut progress_state, &mut config, &mut spline, &ass, &mut sargs, &control, &mut debug_lines);
				tiles_cnt += 1;
			}

			log(format!("total tiles: {}", tiles_cnt));

			if !looped { 
				control.instant = false;
			}
		} else {
			spawn::brick_road_iter(&mut progress_state, &mut config, &mut spline, &ass, &mut sargs, &control, &mut debug_lines);

			if progress_state.finished {
				if looped {
					control.clean_tiles = true;
				} else {
					control.animate	= false;
				}
			}
		}

		if looped && control.spawn_tile == true && progress_state.finished {
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

	for (spline_e, _controlp) in q_controlp.iter() {
		let mut control = q_spline.get_mut(spline_e.0).unwrap();
		control.respawn_all_tiles_instantly();
	}
}