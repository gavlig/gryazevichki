use super           	:: { Herringbone :: * };

pub fn brick_road_system(
	mut q_spline		: Query<(&mut Spline, &mut Control, &mut Config, &mut TileState), Changed<Control>>,

	mut despawn			: ResMut<DespawnResource>,
		time			: Res<Time>,
		ass				: Res<AssetServer>,

		q_tiles			: Query<Entity, With<Herringbone2>>,

		_meshes			: ResMut<Assets<Mesh>>,
		_materials		: ResMut<Assets<StandardMaterial>>,
	mut commands		: Commands
) {
	if q_spline.is_empty() {
		return;
	}

	let (mut spline, mut control, mut config, mut tile_state) = q_spline.single_mut();

	if control.reset {
		for e in q_tiles.iter() {
			despawn.entities.push(e);
		}
	
		tile_state.reset_changed();
	
		control.reset	= false;
	}

	if control.new_spline_point {
		// let spline 		= spline_wcontrols.spline;
		// let key0		= spline.get(0).unwrap();
		// let mut key0_e 	= SplineControlPEntity::new(0, key0, state.parent, &mut meshes, &mut materials, &mut commands);

		control.new_spline_point = false;
	}

	let do_spawn 		= control.next || control.animate;
	if !do_spawn || tile_state.finished {
		return;
	}

	let cur_time		= time.seconds_since_startup();
	if (cur_time - control.last_update) < control.anim_delay_sec && !control.instant {
		return;
	}

	control.last_update = cur_time;

	if !control.instant {
		spawn::brick_road_iter(&mut tile_state, &mut config, &mut spline, &ass, &mut commands);
	} else {
		loop {
			spawn::brick_road_iter(&mut tile_state, &mut config, &mut spline, &ass, &mut commands);
			if tile_state.finished {
				control.instant = false;
				break;
			}
		}
	}

	control.next			= false;

	if tile_state.finished {
		control.animate	= false;
	}
}

pub fn on_spline_tangent_moved(
		time			: Res<Time>,
		q_control_point	: Query<(&Parent, &Transform), With<SplineControlPoint>>,
		q_tangent 		: Query<(&Parent, &Transform, &SplineTangent), Changed<Transform>>,
	mut q_spline		: Query<(&mut Spline, &mut Control)>,
) {
	if time.seconds_since_startup() < 0.1 {
		return;
	}

	if q_spline.is_empty() {
		return;
	}

	for (control_point_e, tanget_tform, tan) in q_tangent.iter() {
		let (spline_e, control_point_tform) = q_control_point.get(control_point_e.0).unwrap();
		let (mut spline, mut control) = q_spline.get_mut(spline_e.0).unwrap();
		// in spline space (or object space)
		let tan_tform	= (*control_point_tform) * (*tanget_tform);
		let tan_pos		= tan_tform.translation;
		match tan {
			SplineTangent::ID(id) => {
				spline.set_interpolation(*id, SplineInterpolation::StrokeBezier(tan_pos, tan_pos));
				control.reset = true;
				control.next = true;
				control.instant = true;
			},
		}
	}
}

pub fn on_spline_control_point_moved(
		time			: Res<Time>,
		q_controlp 		: Query<(&Parent, &Transform, &SplineControlPoint), Changed<Transform>>,
	mut q_spline		: Query<(&mut Spline, &mut Control, &mut Config)>,
) {
	if time.seconds_since_startup() < 0.1 {
		return;
	}

	if q_spline.is_empty() {
		return;
	}

	for (spline_e, tform, controlp) in q_controlp.iter() {
		let (mut spline, mut control, mut config) = q_spline.get_mut(spline_e.0).unwrap();

		let controlp_pos = tform.translation;
		match controlp {
			SplineControlPoint::ID(id) => {
				spline.set_control_point(*id, controlp_pos);

				match *id {
					0 => {
						config.offset_z = controlp_pos.z;
					},
					1 => {
						config.limit_z = controlp_pos.z;
					},
					_ => (),
				};

				control.reset = true;
				control.next = true;
				control.instant = true;
			},
		}
	}
}