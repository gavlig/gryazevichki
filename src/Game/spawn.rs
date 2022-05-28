use bevy			::	prelude :: { * };
use bevy_rapier3d	::	prelude :: { * };
use bevy_fly_camera	::	FlyCamera;

use bevy::render::mesh::shape as render_shape;
use std::f32::consts::	{ * };

use super			::	{ * };

pub fn camera(
	game				: &mut ResMut<GameState>,
	commands			: &mut Commands
) {
	let camera = commands.spawn_bundle(PerspectiveCameraBundle {
			transform: Transform {
				translation: Vec3::new(0., 1., 10.),
				..default()
			},
			..default()
		})
//		.insert			(Collider::ball(1.0))
		.insert			(FlyCamera{ yaw : 195.0, pitch : 7.0,  ..default() })
		.insert			(NameComponent{ name: "Camera".to_string() })
		.id				();

	game.camera			= Some(camera);
	println!			("camera Entity ID {:?}", camera);
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
	let num = 5;
	let offset = Vec3::new(0.0, 0.0, 3.0);
	let line_hsize = Vec3::new(5.0, 0.025, 30.0);

	for i in 0..=num {
		let mut pos = offset.clone();
		pos.x = i as f32 * ((line_hsize.x * 2.0) + 0.5);

		let friction = i as f32 * (1.0 / num as f32); // so that when i == num => friction == 1
		let friction_inv = 1.0 - friction;
		let color = Color::rgb(friction_inv, friction_inv, friction_inv);

		commands
			.spawn_bundle(PbrBundle {
				mesh		: meshes.add			(Mesh::from(render_shape::Box::new(line_hsize.x * 2.0, line_hsize.y * 2.0, line_hsize.z * 2.0))),
				material	: materials.add			(color.into()),
				..Default::default()
			})
			.insert(RigidBody::Fixed)
			.insert(Transform::from_translation(pos))
			.insert(GlobalTransform::default())
			.insert(Collider::cuboid(line_hsize.x, line_hsize.y, line_hsize.z))
			.insert(Friction{ coefficient : friction, combine_rule : CoefficientCombineRule::Average });
//			.insert(ColliderDebugColor(color));
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
	mut meshes			: &mut ResMut<Assets<Mesh>>,
	mut materials		: &mut ResMut<Assets<StandardMaterial>>,
	mut commands		: &mut Commands
) {
	let num				= 10;
	let offset 			= Vec3::new(0.0, 0.0, 25.0);
	let r 				= 0.5;

	for i in 0 ..= num {
		for j in 0 ..= num {
			let mut pose 	= Transform::from_translation(offset.clone());
			pose.translation.x += i as f32 * ((r * 2.0) + 1.0);
			pose.translation.z += j as f32 * ((r * 2.0) + 1.0);

			let friction 	= i as f32 * (1.0 / num as f32); // so that when i == num => friction == 1
			let friction_inv = 1.0 - friction;
			let color		= Color::rgb(friction_inv, friction_inv, friction_inv);

			commands.spawn_bundle(PbrBundle {
				mesh			: meshes.add	(Mesh::from(render_shape::UVSphere{ radius : r, ..default() })),
				material		: materials.add	(color.into()),
				..default()
			})
			.insert				(RigidBody::Dynamic)
			.insert				(pose)
			.insert				(GlobalTransform::default())
			.insert				(Collider::ball(r));
		//	.insert				(Friction{ coefficient : friction, combine_rule : CoefficientCombineRule::Average });
		}
	}
}

pub fn wall(
	mut meshes			: &mut ResMut<Assets<Mesh>>,
	mut materials		: &mut ResMut<Assets<StandardMaterial>>,
	mut commands		: &mut Commands
) {
	let num				= 10;
	let hsize 			= Vec3::new(1.5, 0.3, 0.3);
	let offset 			= Vec3::new(-7.5, hsize.y, 10.0);


	for i in 0 ..= num {
		for j in 0 ..= 5 {
			let mut pose 	= Transform::from_translation(offset.clone());
			pose.translation.x += i as f32 * (hsize.x * 2.0);// + 0.05;
			pose.translation.y += j as f32 * (hsize.y * 2.0);// + 0.4;

			let friction 	= i as f32 * (1.0 / num as f32); // so that when i == num => friction == 1
			let friction_inv = 1.0 - friction;
			let color		= Color::rgb(friction_inv, friction_inv, friction_inv);

			commands.spawn_bundle(PbrBundle {
				mesh			: meshes.add	(Mesh::from(render_shape::Box::new(hsize.x * 2.0, hsize.y * 2.0, hsize.z * 2.0))),
				material		: materials.add	(color.into()),
				..default()
			})
			.insert				(RigidBody::Dynamic)
			.insert				(pose)
			.insert				(GlobalTransform::default())
			.insert				(Collider::cuboid(hsize.x, hsize.y, hsize.z));
		//	.insert				(Friction{ coefficient : friction, combine_rule : CoefficientCombineRule::Average });
		}
	}
}

#[derive(Component)]
pub struct Herringbone;

#[derive(Default)]
pub struct HerringboneStepRequest {
	pub next			: bool,
	pub reset			: bool,
}

pub struct HerringboneIO {
	pub x 				: u32,
	pub z 				: u32,
	pub iter			: u32,
	pub x_limit			: f32,
	pub z_limit			: f32,
	pub limit			: u32,
	pub finished_hor	: bool,
	pub finished		: bool,

	pub num_x 			: u32,
	pub num_z 			: u32,
	pub body_type 		: RigidBody,
	pub offset 			: Vec3,
	pub hsize 			: Vec3,
//	pub x 				: u32,
//	pub z 				: u32,
//	pub x_iter			: u32,
	pub offset_x		: f32,
	pub offset_z		: f32,

	pub orientation		: Orientation2D,
	pub mesh			: Handle<Mesh>,
	pub material		: Handle<StandardMaterial>,
}

impl Default for HerringboneIO {
	fn default() -> Self {
		Self {
			x 			: 0,
			z 			: 0,
			iter		: 0,
			x_limit		: 0.0,
			z_limit		: 0.0,
			limit		: 0,
			finished_hor: false,
			finished	: false,

			num_x		: 0,
			num_z		: 0,
			body_type 	: RigidBody::Fixed,
			offset 		: Vec3::ZERO,
			hsize 		: Vec3::ZERO,
//			x 			: 0,
//			z 			: 0,
//			x_iter		: 0,
			offset_x	: 0.0,
			offset_z	: 0.0,
			orientation	: Orientation2D::Horizontal,
			mesh		: Handle::<Mesh>::default(),
			material	: Handle::<StandardMaterial>::default(),
		}
	}
}

impl HerringboneIO {
	pub fn set_default(&mut self) {
		*self			= Self::default();
	}
}

pub fn _herringbone_brick_road(
	mut meshes			: &mut ResMut<Assets<Mesh>>,
	mut materials		: &mut ResMut<Assets<StandardMaterial>>,
	mut commands		: &mut Commands
) {
	let body_type		= RigidBody::Fixed;
	let num_x			= 3u32 * 3u32;
	let num_z			= 3u32 * 3u32;
//	let hsize 			= Vec3::new(0.1075 / 2.0, 0.065 / 2.0, 0.215 / 2.0);
	let hsize 			= Vec3::new(0.2 / 2.0, 0.05 / 2.0, 0.4 / 2.0);
//	let hsize 			= Vec3::new(0.1075, 0.065, 0.215);
	let offset 			= Vec3::new(1.0, hsize.y, 1.0);

	let mesh			= meshes.add(Mesh::from(render_shape::Box::new(hsize.x * 2.0, hsize.y * 2.0, hsize.z * 2.0)));
	let material		= materials.add(StandardMaterial {
		base_color		: Color::ANTIQUE_WHITE,
		..default		()
	});

	let mut io = HerringboneIO {
		body_type		: body_type,
		offset			: offset,
		hsize			: hsize,
		offset_x		: offset.x,
		offset_z		: offset.z,
		mesh			: mesh.clone_weak(),
		material		: material.clone_weak(),
		..default()
	};

	for x in 0 .. num_x {
		for z in 0 .. num_z {
			io.x		= x;
			io.z		= z;
			herringbone_brick_road_iter(&mut io, &mut commands);
		} // z
	} // x

	let mut pose 		= Transform::from_translation(offset.clone());
	pose.translation.x	+= hsize.z;
	pose.rotation		= Quat::from_rotation_y(FRAC_PI_2);

	commands.spawn_bundle(PbrBundle{ mesh: mesh, material: material, ..default() })
		.insert			(body_type)
		.insert			(pose)
		.insert			(GlobalTransform::default())
		.insert			(Collider::cuboid(hsize.x, hsize.y, hsize.z));
}

pub fn herringbone_brick_road_iter(
	mut io				: &mut HerringboneIO,
	mut commands		: &mut Commands
) {
	// first diagonal
	// horizontal: n - 1 tiles
	// vertical: n tiles

	let mut rotation	= match io.orientation {
		Orientation2D::Horizontal 	=> Quat::from_rotation_y(FRAC_PI_2),
		Orientation2D::Vertical 	=> Quat::IDENTITY,
	};

//	let mut offset_x	= (io.iter + 1) as f32 * io.hsize.z;
//	let mut offset_z	= (io.iter + 1) as f32 * (io.hsize.x * 2.0);
	let seam			= 0.01;

	let hlenz			= io.hsize.z;
	let lenz			= hlenz * 2.0;

	let hlenx			= io.hsize.x;
	let lenx			= hlenx * 2.0;

	let calc_offset_x = |x : u32, iter : u32, orientation : Orientation2D| -> f32 {
		match orientation {
			Orientation2D::Horizontal 	=> ((iter + 1) as f32 * hlenz) 					+ (x as f32 * (lenz * 2.0)),
			Orientation2D::Vertical 	=> ((iter + 0) as f32 * hlenz) + (hlenx * 1.0)	+ (x as f32 * (lenz * 2.0)),
		}
	};

	let calc_offset_z = |z : u32, iter : u32, orientation : Orientation2D| -> f32 {
		match orientation {
			Orientation2D::Horizontal 	=> ((iter + 0) as f32 * hlenz) + (hlenx * 1.0)	+ (z as f32 * (lenz * 2.0)),
			Orientation2D::Vertical 	=> ((iter + 0) as f32 * hlenz) + (hlenz * 2.0) 	+ (z as f32 * (lenz * 2.0)),
		}
	};

	let mut offset_x 	= calc_offset_x(io.x, io.iter, io.orientation);
	let mut offset_z 	= calc_offset_z(io.z, io.iter, io.orientation);

	offset_x			+= ((io.iter + 0) as f32 * seam) + (((io.x + 0) as f32) * seam * 3.0);
	offset_z			+= ((io.iter + 0) as f32 * seam) + (((io.z + 0) as f32) * seam * 3.0);
	match io.orientation {
		Orientation2D::Vertical => offset_z += seam * 1.5,
		Orientation2D::Horizontal => (),//offset_x += seam * 1.5,
		}

	let mut pose 		= Transform::from_translation(io.offset.clone());
	pose.translation.x	+= offset_x;
	pose.translation.z	+= offset_z;
	pose.rotation		= rotation;

	match (io.x, io.z) {
		// spawn strong reference as a first brick to keep reference count > 0
		(0, 0) => {
			commands.spawn_bundle(PbrBundle{ mesh: io.mesh.clone(), material: io.material.clone(), ..default() })
			.insert			(io.body_type)
			.insert			(pose)
			.insert			(GlobalTransform::default())
			.insert			(Collider::cuboid(io.hsize.x, io.hsize.y, io.hsize.z))
		//	.insert			(Friction{ coefficient : friction, combine_rule : CoefficientCombineRule::Average });
			.insert			(Herringbone);
		}
		_ => {
	commands.spawn_bundle(PbrBundle{ mesh: io.mesh.clone_weak(), material: io.material.clone_weak(), ..default() })
		.insert			(io.body_type)
		.insert			(pose)
		.insert			(GlobalTransform::default())
		.insert			(Collider::cuboid(io.hsize.x, io.hsize.y, io.hsize.z))
	//	.insert			(Friction{ coefficient : friction, combine_rule : CoefficientCombineRule::Average });
		.insert			(Herringbone);
		}
	}
	

	println!			("{} x = {} z = {} offx {:.2} offz {:.2} {:?}", io.iter, io.x, io.z, offset_x, offset_z, io.orientation);

	io.iter				+= 1;

	// if only io.limit is given set limits in cordinates anyway because otherwise we don't know where to stop not on diagonal
	if io.iter == io.limit {
		if io.x_limit == 0.0 {
			io.x_limit = offset_x;
		}

		if io.z_limit == 0.0 {
			io.z_limit = offset_z;
		}
	}

	// check for end conditions
	match io.orientation {
		Orientation2D::Horizontal
		if ((offset_x + io.hsize.z + seam >= io.x_limit) && (io.x_limit != 0.0))
		|| ((offset_z + io.hsize.x + seam >= io.z_limit) && (io.z_limit != 0.0))
		|| (io.iter >= io.limit && io.limit != 0) =>
		{
			io.iter		= 0;
			io.orientation = Orientation2D::Vertical;

			println!	("Horizontal -> Vertical x_limit: {} z_limit: {} limit: {}", io.x_limit, io.z_limit, io.limit);
		},
		Orientation2D::Vertical
		if ((offset_x + io.hsize.x + seam >= io.x_limit) && (io.x_limit != 0.0))
		|| ((offset_z + io.hsize.z + seam >= io.z_limit) && (io.z_limit != 0.0))
		|| (io.iter >= io.limit && io.limit != 0) =>
		{
			io.iter		= 0;
			io.orientation = Orientation2D::Horizontal;

			println!	("Vertical -> Horizontal x_limit: {} z_limit: {} limit: {}", io.x_limit, io.z_limit, io.limit);

			let newoffx	= calc_offset_x(io.x + 1, io.iter, io.orientation);
			let newoffz	= calc_offset_z(io.z + 1, io.iter, io.orientation);
			if newoffx + seam < io.x_limit && !io.finished_hor {
				io.x	+= 1;
				println!("x =+ 1 new offx {:.3}", newoffx);
			} else if newoffz + seam < io.z_limit {
				io.x	= 0;
				io.z	+= 1;
				io.finished_hor = true;
				println!("x = 0, z += 1 new offz {:.3}", newoffz);
			} else {
				io.finished = true;
				println!("herringbone_brick_road_iter finished!");
			}
		},
		_				=> ()
	}
}