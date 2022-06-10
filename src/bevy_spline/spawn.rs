use bevy				:: prelude :: { * };
use bevy_mod_picking	:: { * };
use bevy_polyline		:: { prelude :: * };

use bevy::render::mesh::shape as render_shape;

use super				:: { * };
use crate				:: { game };
use crate				:: { draggable, draggable :: * };

pub fn tangent(
	id					: usize,
	key					: &Key,
	parent_e			: Entity,
	sargs				: &mut SpawnArguments,
) -> (Entity, Entity) {
	let cp_pos			= key.value;
	let (tan_pos0, tan_pos1) = match key.interpolation {
		Interpolation::StrokeBezier(V0, V1) => (V0, V1),
		_ => panic!("unsupported interpolation!"),
	};

	let mut spawn = |local_id, transform| -> Entity {
		let mut tan_id 	= Entity::from_raw(0);
		sargs.commands.entity(parent_e).with_children(|parent| {
			tan_id = parent.spawn_bundle(PbrBundle {
				// mesh	: sargs.meshes.add		(Mesh::from(render_shape::Box::new(0.3, 0.3, 0.3))),
				mesh	: sargs.meshes.add		(Mesh::from(render_shape::UVSphere{ radius: 0.15, ..default() })),
				material : sargs.materials.add(
				StandardMaterial {
					base_color	: Color::rgb(0.66, 0.66, 0.56).into(),
					unlit		: true,
					..default()
				}),
				transform : transform,
				..Default::default()
			})
			.insert		(Tangent{ global_id : id, local_id : local_id })
			.insert		(Gizmo)
			.insert_bundle(PickableBundle::default())
			.insert		(Draggable::default())
			.id			();
		});

		tan_id
	};

	// For spline calculation tangent is in the same space as the control point.
	// But in engine it's a child of control point (for convenience) so we have to calculate its pos as a child of control point.
	let transform		= Transform::from_translation(tan_pos0 - cp_pos);
	let tan_id0 		= spawn(0, transform);
	let transform		= Transform::from_translation(tan_pos1 - cp_pos);
	let tan_id1 		= spawn(1, transform);

	(tan_id0, tan_id1)
}

pub fn control_point(
	id					: usize,
	spline				: &Spline,
	parent_e			: Entity,
	with_tangent		: bool,
	polylines			: &mut ResMut<Assets<Polyline>>,
	polyline_materials 	: &mut ResMut<Assets<PolylineMaterial>>,
	sargs				: &mut SpawnArguments,
) -> Entity {
	let mut cp_id 		= Entity::from_raw(0);

	let key				= spline.keys()[id];
	let mut spline_rot	= Quat::IDENTITY;
	if id > 0 {
		let prev_key	= spline.keys()[id - 1];
		spline_rot 		= Quat::from_rotation_arc(Vec3::Z, (key.value - prev_key.value).normalize());
	}

	let transform		= Transform { translation: key.value, rotation: spline_rot, ..default() };

	sargs.commands.entity(parent_e).with_children(|parent| {
		cp_id = parent.spawn_bundle(PbrBundle {
			mesh		: sargs.meshes.add		(Mesh::from(render_shape::UVSphere{ radius: 0.2, ..default() })),
			material	: sargs.materials.add(
			StandardMaterial {
				base_color	: Color::rgb(0.76, 0.76, 0.66).into(),
				unlit		: true,
				..default()
			}),
			transform	: transform,
			..Default::default()
		})
		.insert			(ControlPoint::ID(id))
		.insert			(Gizmo)
		.insert_bundle	(PickableBundle::default())
		.insert			(Draggable::default())
		.id				();
	});

	if with_tangent {
		/*spawn::*/tangent(
			id,
			&key,
			cp_id,
			sargs
		);
	}

	let line_id = sargs.commands.spawn_bundle(PolylineBundle {
		polyline : polylines.add(Polyline {
			vertices	: vec![-Vec3::Z, Vec3::Z],
			..default()
		}),
		material : polyline_materials.add(PolylineMaterial {
			width		: 100.0,
			color		: Color::rgb(0.2, 0.8, 0.2),
			perspective	: true,
			..default()
		}),
		..default()
	})
	.insert				(ControlPointPolyline)
	.id					();

	sargs.commands.entity(cp_id).add_child(line_id);

	cp_id
}

pub fn new(
	transform			: &Transform,
	length				: f32,
	line_width			: f32,
	line_color			: Color,

	polylines			: &mut ResMut<Assets<Polyline>>,
	polyline_materials 	: &mut ResMut<Assets<PolylineMaterial>>,

	mut sargs			: &mut SpawnArguments,
) -> Entity {
	let root_e			= draggable::spawn::root_handle(transform, &mut sargs);

	let offset_y		= 0.5;
	
	// spline requires at least 4 points: 2 control points(Key) and 2 tangents
	//
	//
	let tan_offset		= length / 4.0;

	// limit_z and offset_z are used both for final tile coordinates and for final value of t to have road length tied to spline length and vice versa
	let key0_pos		= Vec3::new(0.0, offset_y, 0.0);
	
	// StrokeBezier allows having two tangent points and we're going to use that
	let tangent00		= Vec3::new(0.0, offset_y, 0.0 - tan_offset);
	let tangent01		= Vec3::new(0.0, offset_y, 0.0 + tan_offset);

	let tangent10		= Vec3::new(0.0, offset_y, length - tan_offset);
	let tangent11		= Vec3::new(0.0, offset_y, length + tan_offset);

	let key1_pos		= Vec3::new(0.0, offset_y, length);

	let t0				= 0.0;
	let t1				= (key1_pos - key0_pos).length();

	let key0			= Key::new(t0, key0_pos, Interpolation::StrokeBezier(tangent00, tangent01));
	let key1			= Key::new(t1, key1_pos, Interpolation::StrokeBezier(tangent10, tangent11));
	let spline			= Spline::from_vec(vec![key0, key1]);

	let key0_e 			= self::control_point(0, &spline, root_e, true, polylines, polyline_materials, &mut sargs);
	let key1_e 			= self::control_point(1, &spline, root_e, true, polylines, polyline_materials, &mut sargs);

	// left border, center and right border lines
	for i in 0..3 {
		let line_id = sargs.commands.spawn_bundle(PolylineBundle {
			polyline : polylines.add(Polyline {
				vertices	: vec![key0_pos, key1_pos],
				..default()
			}),
			material : polyline_materials.add(PolylineMaterial {
				width		: line_width,
				color		: line_color,
				perspective	: true,
				..default()
			}),
			..default()
		})
		.insert				(SplinePolyline)
		.id					();

		sargs.commands.entity(root_e).add_child(line_id);
	}

	sargs.commands.entity(root_e)
		.insert			(spline)
		.add_child		(key0_e)
		.add_child		(key1_e)
		;

	root_e
}