use super           	:: { * };
use crate				:: { bevy_spline };

pub fn brick_road_system(
	mut q_spline		: Query<(
							&Children,
							&Spline,
							&mut BrickRoadProgressState,
							&Herringbone2Config,
							&mut HerringboneControl,
						)>,//, Changed<HerringboneControl>>,

	mut despawn			: ResMut<DespawnResource>,
		time			: Res<Time>,
		ass				: Res<AssetServer>,

		q_tiles			: Query<Entity, With<Herringbone2>>,

	mut	meshes			: ResMut<Assets<Mesh>>,
	mut	materials		: ResMut<Assets<StandardMaterial>>,
	mut commands		: Commands,
) {
	if q_spline.is_empty() {
		return;
	}

	let mut sargs = SpawnArguments {
		meshes		: &mut meshes,
		materials	: &mut materials,
		commands	: &mut commands,
	};

	for (children_e, spline, mut progress_state, config, mut control) in q_spline.iter_mut() {
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

		if spline.keys().len() < 2 {
			continue;
		}

		control.last_update = cur_time;

		if progress_state.hasnt_started() {
			let first_key = spline.keys().first().unwrap().value;
			let spline_r = spline.calc_init_rotation();
			let hwidth_rotated = spline_r.mul_vec3(Vec3::X * (-config.width / 2.0));

			progress_state.pos = first_key - hwidth_rotated;
			progress_state.dir = Direction2D::Right;

			log(format!("Herringbone2 road initialized! init pos: [{:.3} {:.3} {:.3}]", progress_state.pos.x, progress_state.pos.y, progress_state.pos.z));
		}

		if control.instant {
			log(format!("\ninstant Herringbone2 road spawn started!"));

			let mut tiles_cnt = 0;
			while !progress_state.finished {
				spawn::brick_road_iter(spline, &mut progress_state, &config, &ass, &control, &mut sargs);
				tiles_cnt += 1;
			}

			log(format!("total tiles: {}", tiles_cnt));

			if !looped { 
				control.instant = false;
			}
		} else {
			spawn::brick_road_iter(spline, &mut progress_state, &config, &ass, &control, &mut sargs);

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