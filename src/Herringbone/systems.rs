use bevy_polyline		:: { prelude :: * };
use bevy_prototype_debug_lines :: {DebugLines, DebugLinesPlugin};

use super           	:: { * };
use crate				:: { Game };

pub fn brick_road_system(
	mut debug_lines		: ResMut<DebugLines>,
	mut polylines		: ResMut<Assets<Polyline>>,
	mut	polyline_materials : ResMut<Assets<PolylineMaterial>>,
		q_polyline		: Query<&Handle<Polyline>, With<Herringbone2Line>>,
	mut q_spline		: Query<(Entity, &Children, &GlobalTransform, &mut Spline, &mut Control, &mut Herringbone2Config, &mut TileState), Changed<Control>>,
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

		while control.new_spline_point {
			new_spline_point(root_e, &q_mouse_pick, transform, config.as_mut(), spline.as_mut(), &mut polylines, &mut polyline_materials, &mut sargs);

			control.new_spline_point = false;
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

		let mut line_id = 0;
		for &child in children_e.iter() {
			let handle = match q_polyline.get(child) {
				Ok(handle) => handle,
				Err(_) => continue,
			};

			let keys 	= spline.keys();
			let total_keys = keys.len();
			let total_verts	= 32 * total_keys;

			let line	= polylines.get_mut(handle).unwrap();
			line.vertices.resize(total_verts + 1, Vec3::ZERO);
			let total_length = spline.total_length();

			let mut prev_spline_p = Vec3::ZERO;
			let delta = total_length / total_verts as f32;
			for i in 0 ..= total_verts {
				let t = i as f32 * delta;
				let spline_p = spline.clamped_sample(t).unwrap();
				let vert_offset = Vec3::Y * 0.5;

				let offset_x = (-config.width / 2.0) + line_id as f32 * (config.width / 2.0);
				let mut www = Vec3::ZERO; www.x = offset_x;

				let spline_r = {
					let spline_dir	= (spline_p - prev_spline_p).normalize();
					Quat::from_rotation_arc(Vec3::Z, spline_dir)
				};
				prev_spline_p = spline_p;

				www = spline_r.mul_vec3(www);
				line.vertices[i] = spline_p + www;
				line.vertices[i] += vert_offset;		
				
				//
				if i % 7 != 0 { continue }; 
				let normal = spline_r;
				let line_start = transform.translation + spline_p + vert_offset;
				let line_end = transform.translation + spline_p + (normal.mul_vec3(Vec3::X * 3.0)) + vert_offset;
				debug_lines.line(
					line_start,
					line_end,
					0.1,
				);
			}

			line_id += 1;
		}

		control.next	= false;
		if tile_state.finished {
			control.animate	= false;
		}
	}
}

fn new_spline_point(
		root_e		: Entity,
		q_mouse_pick : &Query<&PickingObject, With<Camera>>,
		transform	: &GlobalTransform,
	mut config		: &mut Herringbone2Config,
	mut spline		: &mut Spline,
	polylines		: &mut ResMut<Assets<Polyline>>,
	polyline_materials : &mut ResMut<Assets<PolylineMaterial>>,
	mut	sargs		: &mut SpawnArguments,
) {
	let mouse_pick 	= q_mouse_pick.single();
	let top_pick 	= mouse_pick.intersect_top();

	// There is at least one entity under the cursor
	if top_pick.is_none() {
		return;
	}
	
	let (_topmost_entity, intersection) = top_pick.unwrap();
	let mut new_pos	= intersection.position();
	new_pos			-= transform.translation; // world space -> object space

	// TODO: use line equation here too to put handle precisely under cursor
	new_pos.y		= 0.5;
	let tan0		= new_pos - Vec3::Z * config.init_tangent_offset;
	let tan1		= new_pos + Vec3::Z * config.init_tangent_offset;

	let t			= new_pos.z;
	let key			= SplineKey::new(t, new_pos, SplineInterpolation::StrokeBezier(tan0, tan1));
	spline.add		(key);
	//
	let new_key_id	= spline.get_key_id(t);
	let key_e 		= Game::spawn::spline_control_point(new_key_id, &key, root_e, true, polylines, polyline_materials,  &mut sargs);
	sargs.commands.entity(root_e).add_child(key_e);
	//
	config.limit_z	= new_pos.z;
}

pub fn on_spline_tangent_moved(
		time			: Res<Time>,
		q_tangent_parent: Query<&Parent, (With<SplineTangent>, Changed<Transform>)>,
		q_controlp_parent : Query<&Parent, With<SplineControlPoint>>, // <- parent of this ^
	mut q_control		: Query<&mut Control>, // <- parent of this ^
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
		q_controlp 		: Query<(&Parent, &SplineControlPoint), Changed<Transform>>,
	mut q_spline		: Query<(&Spline, &mut Control)>,
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
			SplineControlPoint::ID(id_ref) => {
				let id 		= *id_ref;
				let last_id = spline.len() - 1;

				control.reset	= true;
				control.next 	= true;
				control.instant = true;
			},
		}
	}
}