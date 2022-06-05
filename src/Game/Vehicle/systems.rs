use bevy			    ::	prelude :: *;
use bevy_fly_camera     ::  FlyCamera;

use super               ::  *;
use self                ::  spawn;

pub fn respawn_vehicle_system(
	mut	game		: ResMut<GameState>,

		q_phys		: Query<&PhysicsConfig>,
	mut	q_body		: Query<(
		&	 BodyConfig,
		&	 PhysicsConfig,
		&mut Transform
	)>,
		q_accel_cfg	: Query<&AcceleratorConfig>,
		q_steer_cfg	: Query<&SteeringConfig>,
		q_axle_cfg	: Query<&AxleConfig>,
		q_wheel_cfg	: Query<&WheelConfig>,
	mut	q_camera	: Query<&mut FlyCamera>,
		ass			: Res<AssetServer>,
	mut	commands	: Commands,
) {
	let (mut body, respawn_body) = match game.body {
		Some(re)		=> (re.entity, re.respawn),
		_				=> return,
	};
	let (body_cfg, body_phys_cfg, mut body_pos) = q_body.get_mut(body).unwrap();
	let accel_cfg		= q_accel_cfg.get(body).unwrap();
	let steer_cfg		= q_steer_cfg.get(body).unwrap();

	if true == respawn_body {
		commands.entity(body).despawn_recursive();

		body_pos.translation = Vec3::new(0.0, 5.5, 0.0);
		body_pos.rotation = Quat::IDENTITY;
		body 			= spawn::body(*body_pos, body_cfg, body_phys_cfg, accel_cfg, steer_cfg, &ass, &mut commands);
		game.body 		= Some(RespawnableEntity { entity : body, ..Default::default() });
		// TODO: is there an event we can attach to? 
		let mut camera 	= q_camera.get_mut(game.camera.unwrap()).unwrap();
		camera.target 	= Some(body);
		println!		("camera.target Entity ID {:?}", camera.target);

		println!		("respawned body Entity ID {:?}", body);
	}

	for side_ref in WHEEL_SIDES {
		let side 		= *side_ref;
		let re_axle 	= game.axles[side].unwrap();
		let re_wheel	= game.wheels[side].unwrap();

		let mut axle	= re_axle.entity;
		let axle_pos 	: Transform;

		let axle_cfg	= q_axle_cfg.get(axle).unwrap().clone();
		let axle_phys_cfg = q_phys.get(axle).unwrap().clone();

		if !re_axle.respawn && !re_wheel.respawn && !respawn_body {
			continue;
		}

		commands.entity(axle).despawn_recursive();

		let axle_offset = body_cfg.axle_offset(side);
		(axle, axle_pos) = spawn::axle_with_joint(
			  side
			, body
			, *body_pos
			, axle_offset
			, &axle_cfg
			, &axle_phys_cfg
			, &ass
			, &mut commands
		);

		game.axles[side] = Some(RespawnableEntity{ entity : axle, respawn: false });

		println!		("respawned {} axle Entity ID {:?}", side, axle);
		
		let mut wheel	= re_wheel.entity;
		let wheel_cfg	= q_wheel_cfg.get(wheel).unwrap().clone();
		let wheel_phys_cfg = q_phys.get(wheel).unwrap().clone();

		commands.entity(wheel).despawn_recursive();

		let wheel_offset = axle_cfg.wheel_offset(side);
		wheel = spawn::wheel_with_joint(
			  side
			, axle
			, axle_pos
			, wheel_offset
			, &wheel_cfg
			, &wheel_phys_cfg
			, &ass
			, &mut commands
		);

		game.wheels[side] = Some(RespawnableEntity{ entity : wheel, respawn: false });

		println!		("respawned {} wheel Entity ID {:?}", side, wheel);
	}
}