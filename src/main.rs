use bevy::prelude::*;
use bevy_rapier3d::{prelude::*, physics::JointHandleComponent};
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};
//use bevy_editor_pls::prelude::*;
use bevy::app::AppExit;

use nalgebra as nalg;
use nalg::vector;

use bevy::render::mesh::shape as render_shape;

#[derive(Default)]
pub struct Game {
	  camera		: Option<Entity>
	, body 			: Option<Entity>

	, rf_axle_joint	: Option<Entity>
	, lf_axle_joint	: Option<Entity>
	, rr_axle_joint	: Option<Entity>
	, lr_axle_joint	: Option<Entity>

	, rf_wheel_joint: Option<Entity>
	, lf_wheel_joint: Option<Entity>
	, rr_wheel_joint: Option<Entity>
	, lr_wheel_joint: Option<Entity>

	, rf_wheel		: Option<Entity>
	, lf_wheel		: Option<Entity>
	, rr_wheel		: Option<Entity>
	, lr_wheel		: Option<Entity>
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
		.add_startup_system(setup_graphics_system)
		.add_startup_system(setup_physics_system)
		.add_startup_system(setup_grab_system)
		.add_startup_system_to_stage(StartupStage::PostStartup, setup_camera_system)
		.add_system(cursor_grab_system)
		.add_system(toggle_button_system)
		.add_system(camera_collision_system)
		.add_system(accelerate_system)
		.add_system_to_stage(CoreStage::PostUpdate, display_events_system)
		.run();
}

fn setup_grab_system(mut windows: ResMut<Windows>) {
	let window = windows.get_primary_mut().unwrap();

	window.set_cursor_lock_mode(true);
	window.set_cursor_visibility(false);
}

fn setup_graphics_system(
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	mut game: ResMut<Game>,
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

	//

	spawn_world_axis(meshes, materials, &mut commands);

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
	game.camera = Some(camera);
	println!("camera Entity ID {:?}", camera);
}

pub fn setup_physics_system(
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
	let body = spawn_body(body_pos, half_size, RigidBodyType::Dynamic, &mut commands);
	game.body = Some(body);
	println!("body Entity ID {:?}", body);

	{
		let offset = Vec3::new(1.6, -0.8, 1.4);
		let (rf_axle_joint, rf_wheel_joint, rf_wheel) = spawn_attached_wheel(body, body_pos, offset, &mut commands);
		(game.rf_axle_joint, game.rf_wheel_joint, game.rf_wheel) = (Some(rf_axle_joint), Some(rf_wheel_joint), Some(rf_wheel));
		println!("rf_wheel Entity ID {:?}", rf_wheel);
	}

	if true {
		let offset = Vec3::new(-1.6, -0.8, 1.4);
		let (lf_axle_joint, lf_wheel_joint, lf_wheel) = spawn_attached_wheel(body, body_pos, offset, &mut commands);
		(game.lf_axle_joint, game.lf_wheel_joint, game.lf_wheel) = (Some(lf_axle_joint), Some(lf_wheel_joint), Some(lf_wheel));
		println!("lf_wheel Entity ID {:?}", lf_wheel);
	}

	if true {
		let offset = Vec3::new(1.6, -0.8, -1.4);
		let (rr_axle_joint, rr_wheel_joint, rr_wheel) = spawn_attached_wheel(body, body_pos, offset, &mut commands);
		(game.rr_axle_joint, game.rr_wheel_joint, game.rr_wheel) = (Some(rr_axle_joint), Some(rr_wheel_joint), Some(rr_wheel));
		println!("rr_wheel Entity ID {:?}", rr_wheel);
	}

	if true {
		let offset = Vec3::new(-1.6, -0.8, -1.4);
		let (lr_axle_joint, lr_wheel_joint, lr_wheel) = spawn_attached_wheel(body, body_pos, offset, &mut commands);
		(game.lr_axle_joint, game.lr_wheel_joint, game.lr_wheel) = (Some(lr_axle_joint), Some(lr_wheel_joint), Some(lr_wheel));
		println!("lr_wheel Entity ID {:?}", lr_wheel);
	}
}

fn setup_camera_system(
	mut game			: ResMut<Game>,
	mut query			: Query<&mut FlyCamera>
) {
	// initialize camera with target to look at
	if game.camera.is_some() && game.body.is_some() {
		let mut camera = query.get_mut(game.camera.unwrap()).unwrap();
		camera.target = Some(game.body.unwrap());
	}
}

fn spawn_attached_wheel(
	body			: Entity,
	body_pos		: Vec3,
	main_offset		: Vec3,
//		half_height	: f32,
//		radius		: f32, 
	mut	commands	: &mut Commands
) -> (Entity, Entity, Entity) {
	let half_height = 0.5;
	let radius 		= 0.8;

	let x_sign		= main_offset.x * (1.0 / main_offset.x.abs());
	let wheel_offset= Vec3::X * 0.8 * x_sign; // 0.2 offset by x axis

	let axle_size	= Vec3::new(0.1, 0.2, 0.1);
	let axle_pos	= body_pos + main_offset;
	let axle		= spawn_wheel_axle(axle_pos, axle_size, RigidBodyType::Dynamic, &mut commands);

	let mut anchor1	= main_offset;
	let mut anchor2 = Vec3::ZERO;
	let axle_joint 	= spawn_axle_joint(body, axle, point![anchor1.x, anchor1.y, anchor1.z], point![anchor2.x, anchor2.y, anchor2.z], &mut commands);

	let wheel_pos 	= axle_pos + wheel_offset;
	let wheel 		= spawn_wheel(wheel_pos, half_height, radius, RigidBodyType::Dynamic, &mut commands);

	anchor1			= wheel_offset;
	anchor2 		= Vec3::ZERO;
	let wheel_joint = spawn_wheel_joint(axle, wheel, point![anchor1.x, anchor1.y, anchor1.z], point![anchor2.x, anchor2.y, anchor2.z], &mut commands);

	(axle_joint, wheel_joint, wheel)
}

fn spawn_wheel_axle(
	pos_in: Vec3,
	half_size: Vec3,
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

	let translated_position = nalg::Isometry3 {
		translation: vector![0.0, 0.3, 0.0].into(),
		..Default::default()
	};

	let axle_collider = ColliderBundle {
		shape: ColliderShape::cuboid(half_size.x, half_size.y, half_size.z).into(),
		position: translated_position.into(),
		mass_properties: ColliderMassProps::Density(1000.0).into(),
		..ColliderBundle::default()
	};

	commands
		.spawn()
		.insert_bundle(rigid_body)
		.insert_bundle(axle_collider)
		.insert(ColliderDebugRender::default())
		.insert(ColliderPositionSync::Discrete)
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

fn spawn_axle_joint(
	entity1: Entity,
	entity2: Entity,
	anchor1: nalgebra::Point3<Real>,
	anchor2: nalgebra::Point3<Real>,
	commands: &mut Commands,
) -> Entity {
	let axle_joint = RevoluteJoint::new(Vector::y_axis())
		.local_anchor1(anchor1)
		.local_anchor2(anchor2)
		.motor_position(0.0, 1.0, 0.2); // by default we want axle joint to stay fixed 

	commands
		.spawn()
		.insert(JointBuilderComponent::new(axle_joint, entity1, entity2))
		.id()
}

fn spawn_wheel_joint(
	entity1: Entity,
	entity2: Entity,
	anchor1: nalgebra::Point3<Real>,
	anchor2: nalgebra::Point3<Real>,
	commands: &mut Commands,
) -> Entity {
	let wheel_joint = RevoluteJoint::new(Vector::x_axis())
		.local_anchor1(anchor1)
		.local_anchor2(anchor2);

	commands
		.spawn()
		.insert(JointBuilderComponent::new(wheel_joint, entity1, entity2))
		.id()
}

fn spawn_body(
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
		mass_properties: ColliderMassProps::Density(10.0).into(), // joints like it when there is an hierarchy of masses and we want body to be the heaviest
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

fn spawn_world_axis(
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
		if (key.pressed(KeyCode::LControl) && key.just_pressed(KeyCode::Space)) || btn.just_pressed(MouseButton::Left) {
			let toggle = !camera.enabled_follow;
			camera.enabled_follow = toggle;
			camera.enabled_movement = !toggle;
			camera.enabled_view = !toggle;

			if toggle {
				camera.target = game.body;
			}
		}

		if key.just_pressed(KeyCode::Escape) {
			if camera.enabled_follow {
				camera.enabled_follow = false;
			} else {
				exit.send(AppExit);
			}
		}
	}
}

fn motor_velocity(velocity: f32, factor: f32, joint_e: Entity, joints: &mut ResMut<ImpulseJointSet>, query: &mut Query<&mut JointHandleComponent>) {
	let 	joint_comp	= query.get(joint_e).unwrap();
	let mut joint		= joints.get_mut(joint_comp.handle()).unwrap();
			joint.data	= joint.data.motor_velocity(JointAxis::AngX, velocity, factor);
}

fn motor_steer(angle: f32, joint_e: Entity, joints: &mut ResMut<ImpulseJointSet>, query: &mut Query<&mut JointHandleComponent>) {
	let 	joint_comp 	= query.get(joint_e).unwrap();

	let 	stiffness 	= 0.8;
	let 	damping 	= 0.2;
	let		angle_rad	= angle.to_radians();
	let mut joint 		= joints.get_mut(joint_comp.handle()).unwrap();
			joint.data 	= joint.data.motor_position(JointAxis::AngX, angle_rad, stiffness, damping)
							;//.unlimit_axis(JointAxis::AngZ)
							//.limit_axis(JointAxis::AngX, [0.0, 0.0]);

//	println!("motor steer {} limit axes {:?}", angle, joint.data.limit_axes);
//	if angle.abs() > 0.0001 {
//			joint.data 	= joint.data.unlimit_axis(JointAxis::AngZ);
//	} else {
//			joint.data 	= joint.data.limit_axis(JointAxis::AngZ, [-0.0001, 0.0001]);
//	}
}

fn accelerate_system(
		key		: Res<Input<KeyCode>>,
		game	: ResMut<Game>,
	mut	joints	: ResMut<ImpulseJointSet>,
	mut	query	: Query<&mut JointHandleComponent>,
	mut commands: Commands,
) {
	let rf_axle_joint = game.rf_axle_joint.unwrap();
	let lf_axle_joint = game.lf_axle_joint.unwrap();
	let rr_axle_joint = game.rr_axle_joint.unwrap();
	let lr_axle_joint = game.lr_axle_joint.unwrap();

	let rf_wheel_joint = game.rf_wheel_joint.unwrap();
	let lf_wheel_joint = game.lf_wheel_joint.unwrap();
	let rr_wheel_joint = game.rr_wheel_joint.unwrap();
	let lr_wheel_joint = game.lr_wheel_joint.unwrap();

	if key.just_pressed(KeyCode::W) {
		motor_velocity(10.0, 0.7, rr_wheel_joint, &mut joints, &mut query);
		motor_velocity(10.0, 0.7, lr_wheel_joint, &mut joints, &mut query);
	} else if key.just_released(KeyCode::W) {
		motor_velocity(0.0, 0.7, rr_wheel_joint, &mut joints, &mut query);
		motor_velocity(0.0, 0.7, lr_wheel_joint, &mut joints, &mut query);
	}
	
	 if key.just_pressed(KeyCode::S) {
		motor_velocity(-10.0, 0.3, rr_wheel_joint, &mut joints, &mut query);
		motor_velocity(-10.0, 0.3, lr_wheel_joint, &mut joints, &mut query);
	} else if key.just_released(KeyCode::S) {
		motor_velocity(0.0, 0.7, rr_wheel_joint, &mut joints, &mut query);
		motor_velocity(0.0, 0.7, lr_wheel_joint, &mut joints, &mut query);
	}
 
	let steer_angle = 20.0;
	if key.just_pressed(KeyCode::D) {
		motor_steer(steer_angle, rf_axle_joint, &mut joints, &mut query);
		motor_steer(steer_angle, lf_axle_joint, &mut joints, &mut query);
	} else if key.just_released(KeyCode::D) {
		motor_steer(0.0, rf_axle_joint, &mut joints, &mut query);
		motor_steer(0.0, lf_axle_joint, &mut joints, &mut query);
	}

 	if key.just_pressed(KeyCode::A) {
		motor_steer(-steer_angle, rf_axle_joint, &mut joints, &mut query);
		motor_steer(-steer_angle, lf_axle_joint, &mut joints, &mut query);
	} else if key.just_released(KeyCode::A) {
		motor_steer(0.0, rf_axle_joint, &mut joints, &mut query);
		motor_steer(0.0, lf_axle_joint, &mut joints, &mut query);
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
/* fn display_contact_info(narrow_phase: Res<NarrowPhase>) {
    let entity1 = ...; // The first entity with a collider bundle attached.
    let entity2 = ...; // The second entity with a collider bundle attached.
    
    /* Find the contact pair, if it exists, between two colliders. */
    if let Some(contact_pair) = narrow_phase.contact_pair(entity1.handle(), entity2.handle()) {
        // The contact pair exists meaning that the broad-phase identified a potential contact.
        if contact_pair.has_any_active_contact {
            // The contact pair has active contacts, meaning that it
            // contains contacts for which contact forces were computed.
        }

        // We may also read the contact manifolds to access the contact geometry.
        for manifold in &contact_pair.manifolds {
            println!("Local-space contact normal: {}", manifold.local_n1);
            println!("Local-space contact normal: {}", manifold.local_n2);
            println!("World-space contact normal: {}", manifold.data.normal);

            // Read the geometric contacts.
            for contact_point in &manifold.points {
                // Keep in mind that all the geometric contact data are expressed in the local-space of the colliders.
                println!("Found local contact point 1: {:?}", contact_point.local_p1);
                println!("Found contact distance: {:?}", contact_point.dist); // Negative if there is a penetration.
                println!("Found contact impulse: {}", contact_point.data.impulse);
                println!("Found friction impulse: {}", contact_point.data.tangent_impulse);
            }

            // Read the solver contacts.
            for solver_contact in &manifold.data.solver_contacts {
                // Keep in mind that all the solver contact data are expressed in world-space.
                println!("Found solver contact point: {:?}", solver_contact.point);
                println!("Found solver contact distance: {:?}", solver_contact.dist); // Negative if there is a penetration.
            }
        }
    }
} */
