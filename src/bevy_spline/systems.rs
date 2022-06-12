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
	 	q_control_point	: Query<(&Parent, &Children, &Transform), With<ControlPoint>>,
	mut q_tangent_set	: ParamSet<(
						  Query<(&Parent, Entity, &Transform, &Tangent), (Changed<Transform>, Without<ControlPoint>)>,
						  Query<&mut Transform, (With<Tangent>, (Without<DraggableActive>, Without<ControlPoint>))>
	)>,
	mut q_spline		: Query<&mut Spline>
) {
	if time.seconds_since_startup() < 0.1 {
		return;
	}

	if q_spline.is_empty() {
		return;
	}

	let sync_tangents	= key.pressed(KeyCode::LControl);

	struct OppositeTangent<'a> {
		entity : Entity,
		pos : Vec3,
		control_point_tform : &'a Transform,
	}
	let mut opposite_tangents : Vec<OppositeTangent> = Vec::new();

	for (control_point_e, tan_e, tan_tform, tan) in q_tangent_set.p0().iter() {
		let (spline_e, control_point_children_e, control_point_tform) = q_control_point.get(control_point_e.0).unwrap();
		let mut spline	= q_spline.get_mut(spline_e.0).unwrap();

		// in spline space (or parent space for tangent handles). _p == parent space
		let tan_tform_p	= (*control_point_tform) * (*tan_tform);
		let tan_pos_p	= tan_tform_p.translation;
		screen_print!("tan_pos {:.3} {:.3} {:.3}", tan_tform.translation.x, tan_tform.translation.y, tan_tform.translation.z);
		screen_print!("tan_pos_p {:.3} {:.3} {:.3}", tan_pos_p.x, tan_pos_p.y, tan_pos_p.z);

		let opposite_tan_pos_p =
		// mirror tangent placement relatively to control point if requested
		if sync_tangents {
			let tan_tform_inv = Transform::from_matrix(tan_tform.clone().compute_matrix().inverse());
			let opposite_tan_tform_p = (*control_point_tform) * tan_tform_inv;
			let opposite_tan_pos_p = opposite_tan_tform_p.translation;

			opposite_tan_pos_p
		// otherwise just set one point of interpolation where the object is
		} else {
			let prev_interpolation = spline.get_interpolation(tan.t);
			let opposite_tan_pos_p = match prev_interpolation {
				Interpolation::StrokeBezier(V0, V1) => {
					if tan.local_id == 0 { *V1 } else { *V0 }
				},
				_ => panic!("unsupported interpolation type!"),
			};

			opposite_tan_pos_p
		};

		let tan0 = if tan.local_id == 0 { tan_pos_p } else { opposite_tan_pos_p };
		let tan1 = if tan.local_id == 1 { tan_pos_p } else { opposite_tan_pos_p };

		spline.set_interpolation(tan.t, Interpolation::StrokeBezier(tan0, tan1));

		for child_e_ref in control_point_children_e.iter() {
			let child_e = *child_e_ref;

			if sync_tangents && child_e != tan_e {
				opposite_tangents.push(
				OppositeTangent {
					entity : child_e,
					pos : opposite_tan_pos_p,
					control_point_tform : control_point_tform,
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
		q_tangent 		: Query<(&Transform, &Tangent)>,
	mut q_spline		: Query<&mut Spline>,
) {
	if time.seconds_since_startup() < 0.1 {
		return;
	}

	if q_spline.is_empty() {
		return;
	}

	for (spline_e, children_e, control_point_tform, mut controlp) in q_controlp.iter_mut() {
		let mut spline = q_spline.get_mut(spline_e.0).unwrap();

		let controlp_pos = control_point_tform.translation;
		match *controlp {
			ControlPoint::T(t_old) => {
				println!("on_control_point_moved {:.3} {:.3} {:.3}", controlp_pos.x, controlp_pos.y, controlp_pos.z);
				let t	= spline.calculate_t_for_pos(controlp_pos);
				let id 	= spline.get_key_id(t_old);
				spline.set_control_point_by_id(id, t, controlp_pos);
				println!("id: {} t: {} controlp_pos: {}", id, t, controlp_pos);

				// we have to recalculate tangent positions because in engine they are children of control point
				// but spline wants them in the same space as control points
				let mut tan0 = Vec3::ZERO;
				let mut tan1 = Vec3::ZERO;
				for tangent_e in children_e.iter() {
					let (tan_tform, tan) = match q_tangent.get(*tangent_e) {
						Ok((tf, tn)) => (tf, tn),
						Err(_) => { continue },
					};
					let final_tform = (*control_point_tform) * (*tan_tform);
					if tan.local_id == 0 {
						tan0 = final_tform.translation;
					} else if tan.local_id == 1 {
						tan1 = final_tform.translation;
					}
				}
				
				spline.set_interpolation(t_old, Interpolation::StrokeBezier(tan0, tan1));

				*controlp = ControlPoint::T(t);
			},
		}
	}
}

pub fn draw_road(
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

			let road_width = match road_width_in { Some(rw) => *rw, None => RoadWidth::W(1.0) };
			let road_width = match road_width { RoadWidth::W(w) => w };

			let mut prev_spline_p = spline.clamped_sample(0.0).unwrap();//Vec3::ZERO;
			let delta = total_length / total_verts as f32;

			screen_print!("keys: {} total_length: {} road_width: {} delta: {}", keys.len(), total_length, road_width, delta);

			for i in 0 ..= total_verts {
				let t = i as f32 * delta;
				let spline_p = spline.clamped_sample(t).unwrap();
				let vert_offset = Vec3::Y * 0.5;

				let offset_x = (-road_width / 2.0) + line_id as f32 * (road_width / 2.0);
				let mut www = Vec3::new(offset_x, 0.0, 0.0);

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
	}
}