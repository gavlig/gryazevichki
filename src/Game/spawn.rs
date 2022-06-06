use bevy				::	prelude :: { * };
use bevy_rapier3d		::	prelude :: { * };
use bevy_fly_camera		::	FlyCamera;
use bevy_mod_picking	::	{ * };

use bevy::render::mesh::shape as render_shape;

use super				::	{ * };

pub fn camera(
	game				: &mut ResMut<GameState>,
	commands			: &mut Commands
) {
	let camera = commands.spawn_bundle(PerspectiveCameraBundle {
			transform: Transform {
				translation: Vec3::new(5., 7., -2.),
				..default()
			},
			..default()
		})
//		.insert			(Collider::ball(1.0))
		.insert			(FlyCamera{ yaw : -225.0, pitch : 45.0, ..default() })
		.insert_bundle	(PickingCameraBundle::default())
// 		.insert			(GizmoPickSource::default())
		.insert			(NameComponent{ name: "Camera".to_string() })
		.id				();

	game.camera			= Some(camera);
	println!			("camera Entity ID {:?}", camera);

	commands.spawn_bundle(UiCameraBundle::default());
}

pub fn ground(
	_game				: &ResMut<GameState>,
	meshes				: &mut ResMut<Assets<Mesh>>,
	materials			: &mut ResMut<Assets<StandardMaterial>>,
	commands			: &mut Commands
) {
	let ground_size 	= 2000.1;
	let ground_height 	= 0.1;

	let ground			= commands
		.spawn			()
		.insert_bundle	(PbrBundle {
			mesh		: meshes.add(Mesh::from(render_shape::Box::new(ground_size * 2.0, ground_height * 2.0, ground_size * 2.0))),
			material	: materials.add(Color::rgb(0.8, 0.8, 0.8).into()),
			transform	: Transform::from_xyz(0.0, -ground_height, 0.0),
			..Default::default()
		})
		.insert			(Collider::cuboid(ground_size, ground_height, ground_size))
		.insert			(Transform::from_xyz(0.0, -ground_height, 0.0))
		.insert			(GlobalTransform::default())
		.insert			(PickableMesh::default())
		.id				();
		
	println!			("ground Entity ID {:?}", ground);
}

pub fn world_axis(
	meshes			: &mut ResMut<Assets<Mesh>>,
	materials		: &mut ResMut<Assets<StandardMaterial>>,
	commands		: &mut Commands,
) {
	// X
	commands.spawn_bundle(PbrBundle {
		mesh		: meshes.add			(Mesh::from(render_shape::Box::new(1.0, 0.1, 0.1))),
		material	: materials.add			(Color::rgb(0.8, 0.1, 0.1).into()),
		transform	: Transform::from_xyz	(0.5, 0.0 + 0.05, 0.0),
		..Default::default()
	});
	// Y
	commands.spawn_bundle(PbrBundle {
		mesh		: meshes.add			(Mesh::from(render_shape::Box::new(0.1, 1.0, 0.1))),
		material	: materials.add			(Color::rgb(0.1, 0.8, 0.1).into()),
		transform	: Transform::from_xyz	(0.0, 0.5 + 0.05, 0.0),
		..Default::default()
	});
	// Z
	commands.spawn_bundle(PbrBundle {
		mesh		: meshes.add			(Mesh::from(render_shape::Box::new(0.1, 0.1, 1.0))),
		material	: materials.add			(Color::rgb(0.1, 0.1, 0.8).into()),
		transform	: Transform::from_xyz	(0.0, 0.0 + 0.05, 0.5),
		..Default::default()
	});
}

pub fn cubes(commands: &mut Commands) {
	let num = 8;
	let rad = 1.0;

	let shift = rad * 2.0 + rad;
	let centerx = shift * (num / 2) as f32;
	let centery = shift / 2.0;
	let centerz = shift * (num / 2) as f32;

	let mut offset = -(num as f32) * (rad * 2.0 + rad) * 0.5;
	let mut color = 0;
	let colors = [
		Color::hsl(220.0, 1.0, 0.3),
		Color::hsl(180.0, 1.0, 0.3),
		Color::hsl(260.0, 1.0, 0.7),
	];

	for j in 0usize..20 {
		for i in 0..num {
			for k in 0usize..num {
				let x = i as f32 * shift - centerx + offset;
				let y = j as f32 * shift + centery + 3.0;
				let z = k as f32 * shift - centerz + offset;
				color += 1;

				commands
					.spawn()
					.insert(RigidBody::Dynamic)
					.insert(Transform::from_xyz(x, y, z))
					.insert(GlobalTransform::default())
					.insert(Collider::cuboid(rad, rad, rad))
					.insert(ColliderDebugColor(colors[color % 3]));
			}
		}

		offset -= 0.05 * rad * (num as f32 - 1.0);
	}
}

pub fn friction_tests(
	meshes				: &mut ResMut<Assets<Mesh>>,
	materials			: &mut ResMut<Assets<StandardMaterial>>,
	commands			: &mut Commands
) {
	let num				= 5;
	let offset			= Vec3::new(0.0, 0.0, 3.0);
	let line_hsize 		= Vec3::new(5.0, 0.025, 30.0);

	for i in 0..= num {
		let mut pos 	= offset.clone();
		pos.x			= i as f32 * ((line_hsize.x * 2.0) + 0.5);

		let friction 	= i as f32 * (1.0 / num as f32); // so that when i == num => friction == 1
		let friction_inv = 1.0 - friction;
		let color 		= Color::rgb(friction_inv, friction_inv, friction_inv);

		commands
			.spawn_bundle(PbrBundle {
				mesh	: meshes.add	(Mesh::from(render_shape::Box::new(line_hsize.x * 2.0, line_hsize.y * 2.0, line_hsize.z * 2.0))),
				material: materials.add	(color.into()),
				..Default::default()
			})
			.insert		(RigidBody		::Fixed)
			.insert		(Transform		::from_translation(pos))
			.insert		(GlobalTransform::default())
			.insert		(Collider		::cuboid(line_hsize.x, line_hsize.y, line_hsize.z))
			.insert		(Friction 		{ coefficient : friction, combine_rule : CoefficientCombineRule::Average });
//			.insert		(ColliderDebugColor(color));
	}
}

pub fn fixed_cube(
	pose				: Transform,
	hsize				: Vec3,
	color				: Color,
	meshes				: &mut ResMut<Assets<Mesh>>,
	materials			: &mut ResMut<Assets<StandardMaterial>>,
	commands			: &mut Commands
) {
	commands.spawn_bundle(PbrBundle {
		mesh			: meshes.add	(Mesh::from(render_shape::Box::new(hsize.x * 2.0, hsize.y * 2.0, hsize.z * 2.0))),
		material		: materials.add	(color.into()),
		..default()
	})
	.insert				(RigidBody::Fixed)
	.insert				(pose)
	.insert				(GlobalTransform::default())
	.insert				(Collider::cuboid(hsize.x, hsize.y, hsize.z));
//	.insert				(Friction{ coefficient : friction, combine_rule : CoefficientCombineRule::Average });
}

pub fn obstacles(
	mut meshes			: &mut ResMut<Assets<Mesh>>,
	mut materials		: &mut ResMut<Assets<StandardMaterial>>,
	mut commands		: &mut Commands
) {
	let num				= 10;
	let offset 			= Vec3::new(0.0, -22.0, 50.0);
	let hsize 			= Vec3::new(25.0, 25.0, 25.0);

	for i in 0 ..= num {
		let mut pose 	= Transform::from_translation(offset.clone());
		pose.translation.x += i as f32 * ((hsize.x * 2.0) + 5.0);
		pose.rotation	= Quat::from_rotation_x(-std::f32::consts::FRAC_PI_8 / 2.0);

		let friction 	= i as f32 * (1.0 / num as f32); // so that when i == num => friction == 1
		let friction_inv = 1.0 - friction;
		let color		= Color::rgb(friction_inv, friction_inv, friction_inv);

		fixed_cube		(pose, hsize, color, &mut meshes, &mut materials, &mut commands);

		pose.translation.z += 60.;
		pose.rotation	= Quat::from_rotation_x(std::f32::consts::FRAC_PI_8 / 2.0);

		fixed_cube		(pose, hsize, color, &mut meshes, &mut materials, &mut commands);
	}
}

pub fn spheres(
	meshes				: &mut ResMut<Assets<Mesh>>,
	materials			: &mut ResMut<Assets<StandardMaterial>>,
	commands			: &mut Commands
) {
	let num				= 10;
	let offset 			= Vec3::new(0.0, 0.0, 25.0);
	let r 				= 0.5;

	for i in 0 ..= num {
		for j in 0 ..= num {
			let mut pose = Transform::from_translation(offset.clone());
			pose.translation.x += i as f32 * ((r * 2.0) + 1.0);
			pose.translation.z += j as f32 * ((r * 2.0) + 1.0);

			let friction = i as f32 * (1.0 / num as f32); // so that when i == num => friction == 1
			let friction_inv = 1.0 - friction;
			let color	= Color::rgb(friction_inv, friction_inv, friction_inv);

			commands.spawn_bundle(PbrBundle {
				mesh	: meshes.add	(Mesh::from(render_shape::UVSphere{ radius : r, ..default() })),
				material: materials.add	(color.into()),
				..default()
			})
			.insert		(RigidBody::Dynamic)
			.insert		(pose)
			.insert		(GlobalTransform::default())
			.insert		(Collider::ball(r));
		//	.insert		(Friction{ coefficient : friction, combine_rule : CoefficientCombineRule::Average });
		}
	}
}

pub fn wall(
	meshes				: &mut ResMut<Assets<Mesh>>,
	materials			: &mut ResMut<Assets<StandardMaterial>>,
	commands			: &mut Commands
) {
	let num				= 10;
	let hsize 			= Vec3::new(1.5, 0.3, 0.3);
	let offset 			= Vec3::new(-7.5, hsize.y, 10.0);

	for i in 0 ..= num {
		for j in 0 ..= 5 {
			let mut pose = Transform::from_translation(offset.clone());
			pose.translation.x += i as f32 * (hsize.x * 2.0);// + 0.05;
			pose.translation.y += j as f32 * (hsize.y * 2.0);// + 0.4;

			let friction = i as f32 * (1.0 / num as f32); // so that when i == num => friction == 1
			let friction_inv = 1.0 - friction;
			let color	= Color::rgb(friction_inv, friction_inv, friction_inv);

			commands.spawn_bundle(PbrBundle {
				mesh	: meshes.add	(Mesh::from(render_shape::Box::new(hsize.x * 2.0, hsize.y * 2.0, hsize.z * 2.0))),
				material: materials.add	(color.into()),
				..default()
			})
			.insert		(RigidBody::Dynamic)
			.insert		(pose)
			.insert		(GlobalTransform::default())
			.insert		(Collider::cuboid(hsize.x, hsize.y, hsize.z));
		//	.insert		(Friction{ coefficient : friction, combine_rule : CoefficientCombineRule::Average });
		}
	}
}

pub fn spline_tangent(
	id					: usize,
	key					: &SplineKey,
	parent_e			: Entity,
	sargs				: &mut SpawnArguments,
) -> (Entity, Entity) {
	let cp_pos			= key.value;
	let (tan_pos0, tan_pos1) = match key.interpolation {
		SplineInterpolation::StrokeBezier(V0, V1) => (V0, V1),
		_ => panic!("unsupported interpolation!"),
	};

	let mut spawn = |local_id, transform| -> Entity {
		let mut tan_id 	= Entity::from_raw(0);
		sargs.commands.entity(parent_e).with_children(|parent| {
			tan_id = parent.spawn_bundle(PbrBundle {
				mesh	: sargs.meshes.add		(Mesh::from(render_shape::Box::new(0.3, 0.3, 0.3))),
				material : sargs.materials.add	(Color::INDIGO.into()),
				transform : transform,
				..Default::default()
			})
			.insert		(SplineTangent{ global_id : id, local_id : local_id })
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

pub fn spline_control_point(
	id					: usize,
	key					: &SplineKey,
	parent_e			: Entity,
	with_tangent		: bool,
	sargs				: &mut SpawnArguments,
) -> Entity {
	let mut cp_id 		= Entity::from_raw(0);
	let transform		= Transform::from_translation(key.value);

	sargs.commands.entity(parent_e).with_children(|parent| {
		cp_id = parent.spawn_bundle(PbrBundle {
			mesh		: sargs.meshes.add		(Mesh::from(render_shape::Box::new(0.4, 0.3, 0.4))),
			material	: sargs.materials.add	(Color::BEIGE.into()),
			transform	: transform,
			..Default::default()
		})
		.insert			(SplineControlPoint::ID(id))
		.insert			(Gizmo)
		.insert_bundle	(PickableBundle::default())
		.insert			(Draggable::default())
		.id				();
	});

	if with_tangent {
		/*spawn::*/spline_tangent(
			id,
			key,
			cp_id,
			sargs
		);
	}

	cp_id
}

pub fn root_handle(
	transform			: Transform,
	sargs				: &mut SpawnArguments,
) -> Entity {
	sargs.commands.spawn_bundle(
	PbrBundle {
		mesh			: sargs.meshes.add		(Mesh::from(render_shape::Box::new(0.4, 0.3, 0.4))),
		material		: sargs.materials.add	(Color::LIME_GREEN.into()),
		transform		: transform,
		..Default::default()
	})
	.insert				(RootHandle)
	.insert				(Gizmo)
	.insert_bundle		(PickableBundle::default())
	.insert				(Draggable::default())
	.id					()
}  