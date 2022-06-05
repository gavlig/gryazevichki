use super           	:: { Herringbone :: * };
use crate				:: { Game };

pub fn brick_road_system(
	mut q_spline		: Query<(Entity, &GlobalTransform, &mut Spline, &mut Control, &mut Config, &mut TileState), Changed<Control>>,
		q_mouse_pick	: Query<&PickingObject, With<Camera>>,

	mut despawn			: ResMut<DespawnResource>,
		time			: Res<Time>,
		ass				: Res<AssetServer>,

		q_tiles			: Query<Entity, With<Herringbone2>>,

	mut	meshes			: ResMut<Assets<Mesh>>,
	mut	materials		: ResMut<Assets<StandardMaterial>>,
	mut commands		: Commands
) {
	if q_spline.is_empty() {
		return;
	}

	let (root_e, transform, mut spline, mut control, mut config, mut tile_state) = q_spline.single_mut();

	if control.reset {
		for e in q_tiles.iter() {
			despawn.entities.push(e);
		}
	
		tile_state.reset_changed();
	
		control.reset	= false;
	}

	while control.new_spline_point {
		let mouse_pick 	= q_mouse_pick.single();
		let top_pick 	= mouse_pick.intersect_top();

		// There is at least one entity under the cursor
		if top_pick.is_none() {
			break;
		}
		
		let (_topmost_entity, intersection) = top_pick.unwrap();
		let mut new_pos	= intersection.position();
		new_pos			-= transform.translation; // world space -> object space

		// TODO: use line equation here too to put handle precisely under cursor
		new_pos.y		= 0.5;
		let tan			= new_pos - Vec3::Z;

		let new_key_id	= spline.len();
		let key			= SplineKey::new(new_pos.z, new_pos, SplineInterpolation::StrokeBezier(tan, tan));
		spline.add		(key);
		//
		let mut sargs = SpawnArguments {
			meshes : &mut meshes,
			materials : &mut materials,
			commands : &mut commands,
		};
		let key_e 		= Game::spawn::spline_control_point(new_key_id, &key, root_e, true, &mut sargs);
		commands.entity(root_e).add_child(key_e);
		//
		config.limit_z	= new_pos.z;

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
				let last_id = spline.len() - 1;

				if *id == 0 {
					config.offset_z = controlp_pos.z;
				} else if *id == last_id {
					config.limit_z = controlp_pos.z;
				}

				control.reset = true;
				control.next = true;
				control.instant = true;
			},
		}
	}
}