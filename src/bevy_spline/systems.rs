use std::ops::ControlFlow;

use bevy			:: { prelude :: * };
use bevy_polyline	:: { prelude :: * };
use bevy_prototype_debug_lines :: { DebugLines };
use bevy_debug_text_overlay :: { screen_print };

use super           :: { * };

// Convert engine Transform of an entity to spline tangent Vec3. Spline tangents are in the same space as control points.
// Spline tangent handles(as in bevy entities with transforms) are children of control point entities so we have to juggle between spline space and tangent space
pub fn on_tangent_moved(
		time			: Res<Time>,
		key				: Res<Input<KeyCode>>,
	mut	polylines		: ResMut<Assets<Polyline>>,
		q_polyline		: Query<&Handle<Polyline>>,
	mut	q_control_point	: Query<(&Parent, &Children, &Transform, &mut ControlPoint)>,
	mut q_tangent_set	: ParamSet<(
						  Query<(&Parent, Entity, &Transform, &Tangent), (Changed<Transform>, Without<ControlPoint>)>,
						  Query<&mut Transform, (With<Tangent>, (Without<DraggableActive>, Without<ControlPoint>))>
	)>,
	mut q_spline		: Query<(&mut Spline, &mut SplineControl)>
) {
	if time.seconds_since_startup() < 0.1 {
		return;
	}

	if q_spline.is_empty() {
		return;
	}

	let sync_tangents	= key.pressed(KeyCode::LControl);

	struct OppositeTangent {
		entity : Entity,
		pos : Vec3,
		control_point_tform : Transform,
	}
	let mut opposite_tangents : Vec<OppositeTangent> = Vec::new();

	for (control_point_e, tan_e, tan_tform, tan) in q_tangent_set.p0().iter() {
		let (spline_e, control_point_children_e, control_point_tform, mut control_point) = q_control_point.get_mut(control_point_e.0).unwrap();
		let (mut spline, mut spline_control) = q_spline.get_mut(spline_e.0).unwrap();
		let key = spline.get_key_from_pos(control_point_tform.translation).unwrap();
		let t = key.t;

		// in spline space (or parent space for tangent handles). _p == parent space
		let tan_tform_p	= (*control_point_tform) * (*tan_tform);
		let tan_pos_p	= tan_tform_p.translation;

		let opposite_tan_pos_p =
		// mirror tangent placement relatively to control point if requested
		if sync_tangents {
			let tan_tform_inv = Transform::from_matrix(tan_tform.clone().compute_matrix().inverse());
			let opposite_tan_tform_p = (*control_point_tform) * tan_tform_inv;
			let opposite_tan_pos_p = opposite_tan_tform_p.translation;

			opposite_tan_pos_p
		// otherwise just set one point of interpolation where the object is
		} else {
			let prev_interpolation = spline.get_interpolation(t);
			let opposite_tan_pos_p = match prev_interpolation {
				Interpolation::StrokeBezier(V0, V1) => {
					if tan.id == 0 { *V1 } else { *V0 }
				},
				_ => panic!("unsupported interpolation type!"),
			};

			opposite_tan_pos_p
		};

		let tan0 = if tan.id == 0 { tan_pos_p } else { opposite_tan_pos_p };
		let tan1 = if tan.id == 1 { tan_pos_p } else { opposite_tan_pos_p };

		spline.set_interpolation(t, Interpolation::StrokeBezier(tan0, tan1));

		spline_control.recalc_length = true;

		for child_e_ref in control_point_children_e.iter() {
			let child_e = *child_e_ref;

			if sync_tangents && child_e != tan_e {
				opposite_tangents.push(
				OppositeTangent {
				 	entity : child_e,
				 	pos : opposite_tan_pos_p,
				 	control_point_tform : *control_point_tform,
				});
			}

			if let Ok(handle) = q_polyline.get(child_e) {
				let control_point_tform_inv = Transform::from_matrix(control_point_tform.clone().compute_matrix().inverse());

				let line	= polylines.get_mut(handle).unwrap();
				line.vertices.resize(3, Vec3::ZERO);
				line.vertices[0] = control_point_tform_inv.mul_vec3(tan0);
				line.vertices[2] = control_point_tform_inv.mul_vec3(tan1);

				line.vertices[1] = Vec3::ZERO;
			}
		}
	}

	for opp in opposite_tangents {
		if let Ok(mut tform) = q_tangent_set.p1().get_mut(opp.entity) {
			let control_point_tform_inv = Transform::from_matrix(opp.control_point_tform.compute_matrix().inverse());
			tform.translation = control_point_tform_inv.mul_vec3(opp.pos);
		}
	}
}

pub fn on_control_point_moved(
		time			: Res<Time>,
	mut	q_controlp 		: Query<(&Parent, &Children, &Transform, &mut ControlPoint), Changed<Transform>>,
	mut	q_tangent 		: Query<(&Transform, &mut Tangent)>,
	mut q_spline		: Query<(&mut Spline, &mut SplineControl)>
) {
	if time.seconds_since_startup() < 0.1 {
		return;
	}

	if q_spline.is_empty() {
		return;
	}

	for (spline_e, children_e, control_point_tform, mut controlp) in q_controlp.iter_mut() {
		let (mut spline, mut spline_control) = q_spline.get_mut(spline_e.0).unwrap();

		let controlp_pos = match *controlp { ControlPoint::POS(p) => p };
		let id = spline.get_key_id_from_pos(controlp_pos).unwrap();
		let t = spline.get_key(id).t;

		let controlp_pos = control_point_tform.translation;
		
		spline.set_control_point_pos(id, controlp_pos);
		println!("id: {} t: {} controlp_pos: {}", id, t, controlp_pos);

		// we have to recalculate tangent positions because in engine they are children of control point
		// but spline wants them in the same space as control points
		let mut tan0 = Vec3::ZERO;
		let mut tan1 = Vec3::ZERO;
		for tangent_e in children_e.iter() {
			let (tan_tform, mut tan) = match q_tangent.get_mut(*tangent_e) {
				Ok((tf, tn)) => (tf, tn),
				Err(_) => { continue },
			};
			let final_tform = (*control_point_tform) * (*tan_tform);
			if tan.id == 0 {
				tan0 = final_tform.translation;
			} else if tan.id == 1 {
				tan1 = final_tform.translation;
			}
		}
		println!("setting interpolation id: {} tan0: {} tan1: {}", id, tan0, tan1);
		spline.set_interpolation(t, Interpolation::StrokeBezier(tan0, tan1));

		*controlp = ControlPoint::POS(controlp_pos);
		spline_control.recalc_length = true;
	}
}

pub fn road_draw(
	mut debug_lines		: ResMut<DebugLines>,
	mut polylines		: ResMut<Assets<Polyline>>,
		q_polyline		: Query<&Handle<Polyline>>,
	mut q_spline		: Query<(&Children, &GlobalTransform, &mut Spline, Option<&RoadWidth>), Changed<Spline>>,
) {
	if q_spline.is_empty() {
		return;
	}

	for (children_e, transform, spline, road_width_in) in q_spline.iter_mut() {
		let mut line_id = 0;

		let vertices 	= generate_spline_vertices(spline.as_ref(), road_width_in, 40);

		for &child in children_e.iter() {
			let handle 	= match q_polyline.get(child) {
				Ok(handle) => handle,
				Err(_) 	=> continue,
			};

			let line	= polylines.get_mut(handle).unwrap();


			// line.vertices.clear();
			if line_id == 0 {
				line.vertices = vertices.to_owned();
			}
    		// line.vertices.reserve(total_verts + 1);

			// let offset_x = (-road_width / 2.0) + *line_id as f32 * (road_width / 2.0);
			// let mut www = Vec3::new(offset_x, 0.0, 0.0);
			// www = spline_r.mul_vec3(www);

			// let line_start = transform.translation + spline_p + vert_offset;
			// let line_end = transform.translation + spline_p + www + vert_offset;
			// debug_lines.line(
			// 	line_start,
			// 	line_end,
			// 	0.01,
			// );

			line_id += 1;
		}
	}
}

fn generate_spline_vertices(
	spline				: &Spline,
	road_width_in		: Option<&RoadWidth>,
	verts_per_segment 	: usize,
) -> Vec<Vec3> {
    let keys = spline.keys();
	let key0 = keys[0];
    if keys.len() < 2 {
		return [key0.value].into();
	}

    let total_keys = keys.len();
    let total_verts	= verts_per_segment * total_keys;
    
    let total_length 	= spline.total_length();
    let road_width 		= match road_width_in 	{ Some(rw) => *rw, None => RoadWidth::W(1.0) };
    let road_width 		= match road_width 		{ RoadWidth::W(w) => w };
    let delta 			= total_length / (total_verts as f32);
	
    screen_print!("keys: {} total_length: {} verts: {} road_width: {} delta: {}", keys.len(), total_length, total_verts, road_width, delta);

	let mut vertices : Vec<Vec3> = [].into();
	vertices.reserve(total_verts);

    for i in 0 ..= total_verts {
		let t = i as f32 * delta;

		let mut option = spline.sample(t);
		if option.is_none() && t >= total_length {
			option = Some(keys.last().unwrap().value);
		}
		let spline_p = option.unwrap();

		let mut t_next = t + 0.01;
		if i == total_verts {
			t_next = t - 0.01;
		}

		let next_spline_p = spline.clamped_sample(t + 0.01).unwrap();
		let vert_offset = Vec3::Y * 0.5;

		let spline_dir	= (next_spline_p - spline_p).normalize();
		let mut spline_r = Quat::from_rotation_arc(Vec3::Z, spline_dir);

		vertices.push(spline_p + vert_offset);
	}

	vertices
}

pub fn road_system(
	mut polylines		: ResMut<Assets<Polyline>>,
	mut	polyline_materials : ResMut<Assets<PolylineMaterial>>,
	mut q_spline		: Query<(Entity, &Children, &GlobalTransform, &mut Spline, &mut SplineControl), Changed<SplineControl>>,
	mut	q_controlp 		: Query<&mut ControlPoint>,
		q_mouse_pick	: Query<&PickingObject, With<Camera>>,

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

	let mouse_pick 	= q_mouse_pick.single();

	for (root_e, children_e, transform, mut spline, mut control) in q_spline.iter_mut() {
		while control.new_point {
			spawn::new_point(
				root_e,
				mouse_pick,
				transform,
				spline.as_mut(),
				&mut polylines,
				&mut polyline_materials,
				&mut sargs
			);

			control.new_point = false;
		}

		while control.recalc_length {
			let keys = spline.keys();
			let keys_cnt = keys.len();
			let mut total_length = 0.0;
			for key_id in 1 .. keys_cnt {
				let new_t = spline.calculate_segment_length(key_id);
				spline.set_control_point_t(key_id, new_t + total_length);
				println!("[{}]recalc_length {:.3}\n", key_id, new_t + total_length);
				total_length += new_t;
			}

			control.recalc_length = false;
		}
	}
}