use bevy::prelude::*;
use bevy_rapier3d::{prelude::*, physics::JointHandleComponent};
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};
//use bevy_editor_pls::prelude::*;
use bevy::app::AppExit;
use nalgebra as nalg;

use bevy::render::mesh::shape as render_shape;

#[derive(Default)]
pub struct Game {
	  body 		: Option<Entity>

	, rf_wheel	: Option<Entity>
	, lf_wheel	: Option<Entity>
	, rr_wheel	: Option<Entity>
	, lr_wheel	: Option<Entity>

	, rf_joint	: Option<Entity>
	, lf_joint	: Option<Entity>
	, rr_joint	: Option<Entity>
	, lr_joint	: Option<Entity>
}

fn main() {
	App::new()
		.insert_resource(ClearColor(Color::rgb(
			0xF9 as f32 / 255.0,
			0xF9 as f32 / 255.0,
			0xFF as f32 / 255.0,
		)))
		.insert_resource(Msaa::default())
		.init_resource::<Game>()
		.add_plugins(DefaultPlugins)
		.add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
		.add_plugin(RapierRenderPlugin)
		.add_plugin(FlyCameraPlugin)
//		.add_plugin(EditorPlugin)
		.add_startup_system(setup_graphics.system())
		.add_startup_system(setup_physics.system())
		.add_startup_system(setup_grab_system)
		.add_system(cursor_grab_system)
		.add_system(toggle_button_system)
		.add_system(camera_collision_system)
		.add_system(accelerate_system)
		.add_system_to_stage(CoreStage::PostUpdate, display_events_system)
		.run();
}

fn setup_grab_system(mut windows: ResMut<Windows>) {
	let window = windows.get_primary_mut().unwrap();

	//window.set_cursor_lock_mode(true);
	//window.set_cursor_visibility(false);
}

fn setup_graphics(
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	mut commands: Commands,
) {
	const HALF_SIZE: f32 = 100.0;

	commands.spawn_bundle(DirectionalLightBundle {
		directional_light: DirectionalLight {
			illuminance: 10000.0,
			// Configure the projection to better fit the scene
			shadow_projection: OrthographicProjection {
				left: -HALF_SIZE,
				right: HALF_SIZE,
				bottom: -HALF_SIZE,
				top: HALF_SIZE,
				near: -10.0 * HALF_SIZE,
				far: 100.0 * HALF_SIZE,
				..Default::default()
			},
			shadows_enabled: true,
			..Default::default()
		},
		transform: Transform {
			translation: Vec3::new(10.0, 2.0, 10.0),
			rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4),
			..Default::default()
		},
		..Default::default()
	});

	// axis

	spawn_axis(meshes, materials, &mut commands);

	// camera

	let camera_collider = ColliderBundle {
		shape: ColliderShape::ball(1.0).into(),
		..ColliderBundle::default()
	};

	let camera = commands
		.spawn_bundle(PerspectiveCameraBundle {
		transform: Transform {
			translation: Vec3::new(0., 1., 10.),
			..Default::default()
		},
		..Default::default()
	})
	.insert_bundle(camera_collider)
		.insert(FlyCamera::default())
		.id();
	println!("camera Entity ID {:?}", camera);
}

pub fn setup_physics(
	mut configuration	: ResMut<RapierConfiguration>,
	mut game			: ResMut<Game>,
	mut commands		: Commands
) {
	configuration.timestep_mode = TimestepMode::VariableTimestep;

	// Ground
	let ground_size = 200.1;
	let ground_height = 0.1;

	let ground_bundle = ColliderBundle {
		shape: ColliderShape::cuboid(ground_size, ground_height, ground_size).into(),
		position: Vec3::new(0.0, -ground_height, 0.0).into(),
		..ColliderBundle::default()
	};

	let ground = commands
		.spawn_bundle(ground_bundle)
		.insert(ColliderDebugRender::default())
		.insert(ColliderPositionSync::Discrete)
		.id();

	println!("ground Entity ID {:?}", ground);

	if false {
		spawn_cubes(&mut commands);
	}

	// wheels and body 1.13
	let half_height: f32 = 0.5;
	let radius: f32 = 0.80;

	let body_pos = Vec3::new(0.0, 5.5, 0.0);
	let half_size = Vec3::new(0.5, 0.5, 1.0);
	let body = spawn_box(body_pos, half_size, RigidBodyType::Dynamic, &mut commands);
	game.body = Some(body);
	println!("body Entity ID {:?}", body);

	{
		let anchor1 = Vec3::new(1.6, -0.8, 1.4);
		let anchor2 = Vec3::ZERO;

		let wheel_pos = body_pos + anchor1;

		let rf_wheel = spawn_wheel(wheel_pos, half_height, radius, RigidBodyType::Dynamic, &mut commands);
		game.rf_wheel = Some(rf_wheel);
		println!("rf_wheel Entity ID {:?}", rf_wheel);

		let rf_joint = create_6dof_joint(body, rf_wheel, point![anchor1.x, anchor1.y, anchor1.z], point![anchor2.x, anchor2.y, anchor2.z], &mut commands);
		game.rf_joint = Some(rf_joint);
	}

	if true {
		let anchor1 = Vec3::new(-1.6, -0.8, 1.4);
		let anchor2 = Vec3::ZERO;

		let wheel_pos = body_pos + anchor1;

		let lf_wheel = spawn_wheel(wheel_pos, half_height, radius, RigidBodyType::Dynamic, &mut commands);
		game.lf_wheel = Some(lf_wheel);
		println!("lf_wheel Entity ID {:?}", lf_wheel);

		let lf_joint = create_6dof_joint(body, lf_wheel, point![anchor1.x, anchor1.y, anchor1.z], point![anchor2.x, anchor2.y, anchor2.z], &mut commands);
		game.lf_joint = Some(lf_joint);
	}

	if true {
		let anchor1 = Vec3::new(1.6, -0.8, -1.4);
		let anchor2 = Vec3::ZERO;

		let wheel_pos = body_pos + anchor1;

		let rr_wheel = spawn_wheel(wheel_pos, half_height, radius, RigidBodyType::Dynamic, &mut commands);
		game.rr_wheel = Some(rr_wheel);
		println!("rr_wheel Entity ID {:?}", rr_wheel);

		let rr_joint = create_6dof_joint(body, rr_wheel, point![anchor1.x, anchor1.y, anchor1.z], point![anchor2.x, anchor2.y, anchor2.z], &mut commands);
		game.rr_joint = Some(rr_joint);
	}

	if true {
		let offset = Vec3::new(-1.6, -0.8, -1.4);
		let (lr_wheel, lr_joint) = spawn_attached_wheel(body, body_pos, offset, &mut commands);
		(game.lr_wheel, game.lr_joint) = (Some(lr_wheel), Some(lr_joint));
	}
}

fn create_6dof_joint(
	entity1: Entity,
	entity2: Entity,
	anchor1: nalgebra::Point3<Real>,
	anchor2: nalgebra::Point3<Real>,
	commands: &mut Commands,
) -> Entity {
	let mut joint_6dof = RevoluteJoint::new(Vector::x_axis())
		.local_anchor1(anchor1)
		.local_anchor2(anchor2);
	//joint_6dof = joint_6dof.motor_velocity(1., 0.1);

	commands
		.spawn()
		.insert(JointBuilderComponent::new(joint_6dof, entity1, entity2))
		.id()
}

fn spawn_wheel(
	pos_in: Vec3,
	half_height: f32,
	radius: f32,
	body_type: RigidBodyType,
	commands: &mut Commands,
) -> Entity {
	let mut pos_comp = RigidBodyPositionComponent::default();
	pos_comp.position.translation = pos_in.clone().into();

	let rigid_body = RigidBodyBundle {
		position: pos_comp,
		body_type: RigidBodyTypeComponent(body_type),
		..RigidBodyBundle::default()
	};

	// by default cylinder spawns with its flat surface on the ground and we want the round part
	let wheel_rotation = nalgebra::UnitQuaternion::from_axis_angle(
		&nalgebra::Vector3::z_axis(),
		std::f32::consts::FRAC_PI_2,
	);

	let rotated_position = nalg::Isometry3 {
		rotation: wheel_rotation,
		..Default::default()
	};

	let wheel_collider = ColliderBundle {
		shape: ColliderShape::cylinder(half_height, radius).into(),
		position: rotated_position.into(),
		flags: (ActiveEvents::INTERSECTION_EVENTS | ActiveEvents::CONTACT_EVENTS).into(),
		..ColliderBundle::default()
	};

	commands
		.spawn()
		.insert_bundle(rigid_body)
		.insert_bundle(wheel_collider)
		.insert(ColliderDebugRender::default())
		.insert(ColliderPositionSync::Discrete)
		.id()
}

fn spawn_attached_wheel(
		body		: Entity,
		body_pos	: Vec3,
		offset		: Vec3,
//		half_height	: f32,
//		radius		: f32, 
	mut	commands	: &mut Commands
) -> (Entity, Entity) {
	let half_height = 0.5;
	let radius 		= 0.8;

	let anchor1		= offset;
	let anchor2 	= Vec3::ZERO;
	let wheel_pos 	= body_pos + anchor1;

	let wheel 		= spawn_wheel(wheel_pos, half_height, radius, RigidBodyType::Dynamic, &mut commands);
	let joint 		= create_6dof_joint(body, wheel, point![anchor1.x, anchor1.y, anchor1.z], point![anchor2.x, anchor2.y, anchor2.z], &mut commands);

	(wheel, joint)
}

fn spawn_box(
	pos_in: Vec3,
	half_size: Vec3,
	body_type: RigidBodyType,
	commands: &mut Commands,
) -> Entity {
	let mut component = RigidBodyPositionComponent::default();
	component.position.translation = pos_in.clone().into();

	let rigid_body = RigidBodyBundle {
		position: component,
		body_type: RigidBodyTypeComponent(body_type),
		..RigidBodyBundle::default()
	};

	let box_collider = ColliderBundle {
		shape: ColliderShape::cuboid(half_size.x, half_size.y, half_size.z).into(),
		..ColliderBundle::default()
	};

	commands
		.spawn()
		.insert_bundle(rigid_body)
		.insert_bundle(box_collider)
		.insert(ColliderDebugRender::default())
		.insert(ColliderPositionSync::Discrete)
		.id()
}

fn spawn_cubes(commands: &mut Commands) {
	/*
	 * Create the cubes
	 */
	 let num = 8;
	 let rad = 1.0;
 
	 let shift = rad * 2.0 + rad;
	 let centerx = shift * (num / 2) as f32;
	 let centery = shift / 2.0;
	 let centerz = shift * (num / 2) as f32;
 
	 let mut offset = -(num as f32) * (rad * 2.0 + rad) * 0.5;
	 let mut color = 0;
 
	 for j in 0usize..20 {
		 for i in 0..num {
			 for k in 0usize..num {
				 let x = i as f32 * shift - centerx + offset;
				 let y = j as f32 * shift + centery + 3.0;
				 let z = k as f32 * shift - centerz + offset;
				 color += 1;
 
				 // Build the rigid body.
				 let rigid_body = RigidBodyBundle {
					 position: Vec3::new(x, y, z).into(),
					 ..RigidBodyBundle::default()
				 };
 
				 let collider = ColliderBundle {
					 shape: ColliderShape::cuboid(rad, rad, rad).into(),
					 ..ColliderBundle::default()
				 };
 
				 commands
					 .spawn()
					 .insert_bundle(rigid_body)
					 .insert_bundle(collider)
					 .insert(ColliderDebugRender::with_id(color))
					 .insert(ColliderPositionSync::Discrete);
			 }
		 }
 
		 offset -= 0.05 * rad * (num as f32 - 1.0);
	 }
}

fn spawn_axis(
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	commands: &mut Commands,
) {
	// X
	commands.spawn_bundle(PbrBundle {
		mesh: meshes.add(Mesh::from(render_shape::Box::new(1.0, 0.1, 0.1))),
		material: materials.add(Color::rgb(0.8, 0.1, 0.1).into()),
		transform: Transform::from_xyz(0.5, 0.0 + 0.05, 0.0),
		..Default::default()
	});
	// Y
	commands.spawn_bundle(PbrBundle {
		mesh: meshes.add(Mesh::from(render_shape::Box::new(0.1, 1.0, 0.1))),
		material: materials.add(Color::rgb(0.1, 0.8, 0.1).into()),
		transform: Transform::from_xyz(0.0, 0.5 + 0.05, 0.0),
		..Default::default()
	});
	// Z
	commands.spawn_bundle(PbrBundle {
		mesh: meshes.add(Mesh::from(render_shape::Box::new(0.1, 0.1, 1.0))),
		material: materials.add(Color::rgb(0.1, 0.1, 0.8).into()),
		transform: Transform::from_xyz(0.0, 0.0 + 0.05, 0.5),
		..Default::default()
	});
}

fn cursor_grab_system(
	mut windows: ResMut<Windows>,
	btn: Res<Input<MouseButton>>,
	key: Res<Input<KeyCode>>,
) {
	let window = windows.get_primary_mut().unwrap();

	if btn.just_pressed(MouseButton::Left) {
		window.set_cursor_lock_mode(true);
		window.set_cursor_visibility(false);
	}

	if key.just_pressed(KeyCode::Escape) {
		window.set_cursor_lock_mode(false);
		window.set_cursor_visibility(true);
	}
}

fn toggle_button_system(
		btn		: Res<Input<MouseButton>>,
		key		: Res<Input<KeyCode>>,
		game	: Res<Game>,
	mut exit	: EventWriter<AppExit>,
	mut query	: Query<&mut FlyCamera>,
) {
	for mut camera in query.iter_mut() {
		if btn.just_pressed(MouseButton::Left) {
			camera.enabled_view = true;
		}

		if key.just_pressed(KeyCode::Escape) {
			if camera.enabled_view {
				camera.enabled_view = false;
			} else {
				exit.send(AppExit);
			}
		}

		if key.just_pressed(KeyCode::Space) && key.pressed(KeyCode::LControl) {
			camera.enabled_movement = !camera.enabled_movement;
			camera.enabled_view = camera.enabled_movement;

			if !camera.enabled_movement {
				camera.target = game.body;
			}
		}
	}
}

fn accelerate_system(
		key		: Res<Input<KeyCode>>,
	mut	game	: ResMut<Game>,
	mut joints	: ResMut<ImpulseJointSet>,
		query	: Query<&mut JointHandleComponent>,
	mut commands: Commands,
) {
	let motor_velocity = |velocity: f32, factor: f32, game: ResMut<Game>, mut joints: ResMut<ImpulseJointSet>| {
		let rr_joint_e 	= match game.rr_joint 	{ Some(e) => e, _ => return	};
		let lr_joint_e 	= match game.lr_joint 	{ Some(e) => e, _ => return	};

		let lr_joint_comp = query.get(lr_joint_e).unwrap();
		let rr_joint_comp = query.get(rr_joint_e).unwrap();
		{
			let mut lr_joint = joints.get_mut(lr_joint_comp.handle()).unwrap();
			lr_joint.data = lr_joint.data.motor_velocity(JointAxis::AngX, velocity, factor);
		}
		{
			let mut rr_joint = joints.get_mut(rr_joint_comp.handle()).unwrap();
			rr_joint.data = rr_joint.data.motor_velocity(JointAxis::AngX, velocity, factor);
		}
	};

	if key.just_pressed(KeyCode::W) {
		motor_velocity(10.0, 0.1, game, joints);
	} else if key.just_released(KeyCode::W) {
		motor_velocity(0.0, 1.0, game, joints);
	} else if key.just_pressed(KeyCode::S) {
		motor_velocity(-10.0, 0.5, game, joints);
	} else if key.just_released(KeyCode::S) {
		motor_velocity(0.0, 0.5, game, joints);
	}
}

fn camera_collision_system(
	mut query: Query<(
		&	 FlyCamera,
		&	 Transform,
		&mut ColliderPositionComponent,
	)>,
) {
	for (_options, transform, mut collider_position) in query.iter_mut() {
		collider_position.translation = transform.translation.into();
	}
}

fn display_events_system(
	mut intersection_events: EventReader<IntersectionEvent>,
	mut contact_events: EventReader<ContactEvent>,
) {
	if true {return}

	for intersection_event in intersection_events.iter() {
		println!("Received intersection event: collider1 {:?} collider2 {:?}", intersection_event.collider1.entity(), intersection_event.collider2.entity());
	}

	for contact_event in contact_events.iter() {
		match contact_event {
			ContactEvent::Started(collider1, collider2) => println!("Received contact START event: collider1 {:?} collider2 {:?}", collider1.entity(), collider2.entity()),
			ContactEvent::Stopped(collider1, collider2) => println!("Received contact STOP event: collider1 {:?} collider2 {:?}", collider1.entity(), collider2.entity()),
		}
	}
}

// contact info + modification. I'd rather add more info to event
// see contact filtering for bevy adaptation
/* struct OneWayPlatformHook {
	platform1: ColliderHandle,
	platform2: ColliderHandle,
} 
impl PhysicsHooks<RigidBodySet, ColliderSet> for OneWayPlatformHook {
	fn modify_solver_contacts(
		&self,
		context: &mut ContactModificationContext<RigidBodySet, ColliderSet>,
	) {
		// The allowed normal for the first platform is its local +y axis, and the
		// allowed normal for the second platform is its local -y axis.
		//
		// Now we have to be careful because the `manifold.local_n1` normal points
		// toward the outside of the shape of `context.co1`. So we need to flip the
		// allowed normal direction if the platform is in `context.collider_handle2`.
		//
		// Therefore:
		// - If context.collider_handle1 == self.platform1 then the allowed normal is +y.
		// - If context.collider_handle2 == self.platform1 then the allowed normal is -y.
		// - If context.collider_handle1 == self.platform2 then its allowed normal +y needs to be flipped to -y.
		// - If context.collider_handle2 == self.platform2 then the allowed normal -y needs to be flipped to +y.
		let mut allowed_local_n1 = Vector::zeros();

		if context.collider1 == self.platform1 {
			allowed_local_n1 = Vector::y();
		} else if context.collider2 == self.platform1 {
			// Flip the allowed direction.
			allowed_local_n1 = -Vector::y();
		}

		if context.collider1 == self.platform2 {
			allowed_local_n1 = -Vector::y();
		} else if context.collider2 == self.platform2 {
			// Flip the allowed direction.
			allowed_local_n1 = Vector::y();
		}

		// Call the helper function that simulates one-way platforms.
		context.update_as_oneway_platform(&allowed_local_n1, 0.1);

		// Set the surface velocity of the accepted contacts.
		let tangent_velocity =
			if context.collider1 == self.platform1 || context.collider2 == self.platform2 {
				-12.0
			} else {
				12.0
			};

		for contact in context.solver_contacts.iter_mut() {
			contact.tangent_velocity.x = tangent_velocity;
	}
}
} */
