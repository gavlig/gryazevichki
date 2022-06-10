use bevy			:: { prelude :: * };
use bevy_polyline	:: { prelude :: * };

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

		let opposite_tan_pos_p =
		// mirror tangent placement relatively to control point if requested
		if sync_tangents {
			let tan_tform_inv = Transform::from_matrix(tan_tform.clone().compute_matrix().inverse());
			let opposite_tan_tform_p = (*control_point_tform) * tan_tform_inv;
			let opposite_tan_pos_p = opposite_tan_tform_p.translation;

			opposite_tan_pos_p
		// otherwise just set one point of interpolation where the object is
		} else {
			let prev_interpolation = spline.get_interpolation(tan.global_id);
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

		spline.set_interpolation(tan.global_id, Interpolation::StrokeBezier(tan0, tan1));

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
		q_controlp 		: Query<(&Parent, &Children, &Transform, &ControlPoint), Changed<Transform>>,
		q_tangent 		: Query<(&Transform, &Tangent)>,
	mut q_spline		: Query<&mut Spline>,
) {
	if time.seconds_since_startup() < 0.1 {
		return;
	}

	if q_spline.is_empty() {
		return;
	}

	for (spline_e, children_e, control_point_tform, controlp) in q_controlp.iter() {
		let mut spline = q_spline.get_mut(spline_e.0).unwrap();

		let controlp_pos = control_point_tform.translation;
		match controlp {
			ControlPoint::ID(id_ref) => {
				let id = *id_ref;
				spline.set_control_point(id, controlp_pos);

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
				
				spline.set_interpolation(id, Interpolation::StrokeBezier(tan0, tan1));
			},
		}
	}
}