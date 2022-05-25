use bevy			::	prelude :: *;
use bevy_rapier3d	::	prelude :: *;
use bevy_egui		::	egui :: { Slider, Ui };
use bevy_egui		::	{ egui, EguiContext };

mod egui_ext;
pub use egui_ext::FileDialog;
	use egui_ext::toggle_switch;

use super::*;
use super::Vehicle;
use super::Vehicle::*;

fn set_cylinder_hh(
	new_hh: f32,
	shared_shape: &mut Mut<Collider>,
) {
	let 	shape 	= shared_shape.raw.make_mut();
	let mut cylinder= shape.as_cylinder_mut().unwrap();
	cylinder.half_height = new_hh;
}

fn set_cylinder_r(
	new_r: f32,
	shared_shape: &mut Mut<Collider>,
) {
	let 	shape 	= shared_shape.raw.make_mut();
	let mut cylinder= shape.as_cylinder_mut().unwrap();
	cylinder.radius = new_r;
}

fn set_box_half_size(
	new_hs: Vec3,
	shared_shape: &mut Mut<Collider>,
) {
	let 	shape 	= shared_shape.raw.make_mut();
	let mut cuboid	= shape.as_cuboid_mut().unwrap();
	cuboid.half_extents	= new_hs.into();
}

fn set_density(
		density_in			: f32,
	mut mass_props_co		: &mut Mut<ColliderMassProperties>,
) {
	match &mut mass_props_co as &mut ColliderMassProperties {
		ColliderMassProperties::Density(density) => {
			*density 		= density_in;
			**mass_props_co = ColliderMassProperties::Density(*density);
		},
		ColliderMassProperties::MassProperties(_) => (),
	};
}

fn set_friction(
		friction_in			: f32,
	mut friction			: &mut Mut<Friction>,
) {
	friction.as_mut().coefficient = friction_in;
}

fn set_restitution(
		restitution_in		: f32,
	mut restitution			: &mut Mut<Restitution>,
) {
	restitution.as_mut().coefficient = restitution_in;
}

fn set_damping(
	lin_damping_in			: f32,
	ang_damping_in			: f32,
mut damping					: &mut Mut<Damping>,
) {
	damping.as_mut().linear_damping = lin_damping_in;
	damping.as_mut().angular_damping = ang_damping_in;
}

fn draw_density_param_ui(
		ui					: &mut Ui,
		range				: [f32; 2],
		density				: &mut f32,
		mass				: f32,
) -> bool {
	ui.add(
		Slider::new			(density, std::ops::RangeInclusive::new(range[0], range[1])).text(format!("Density (Mass {:.3})", mass))
	).changed()
}

fn draw_phys_params_ui(
	  ui					: &mut Ui
	, cfg					: &mut PhysicsConfig
) -> bool {

  let mut changed	= false;

  ui.collapsing("Physics", |ui| {
  ui.vertical(|ui| {

  changed |= draw_density_param_ui(ui, [0.05, 2000.0], &mut cfg.density, cfg.mass);

  changed |= ui.add(
	  Slider::new(&mut cfg.friction, 0.0 ..= 1.0).text("Friction"),
  ).changed();

  changed |= ui.add(
	  Slider::new(&mut cfg.restitution, 0.0 ..= 1.0).text("Restitution"),
  ).changed();

  changed |= ui.add(
	  Slider::new(&mut cfg.lin_damping, 0.0 ..= 100.0).text("Linear Damping"),
  ).changed();

  changed |= ui.add(
	  Slider::new(&mut cfg.ang_damping, 0.0 ..= 100.0).text("Angular Damping"),
  ).changed();

  }); // ui.vertical
  }); // ui.collapsing

  changed
}

fn draw_body_params_ui_collapsing(
	ui						: &mut Ui,
	name					: &String,
	density_range			: [f32; 2],
	mass					: f32,
	cfg						: &mut BodyConfig,
	phys					: &mut PhysicsConfig,
	section_name			: &str
) -> bool {
	let mut changed			= false;		
	let cache				= cfg.clone();	

	ui.collapsing(section_name, |ui| {
		ui.vertical(|ui| {
			changed 		|= ui.add(
				Slider::new(&mut cfg.half_size[0], 0.05 ..= 5.0).text("Half Size X"),
			).changed();
			
			changed 		|= ui.add(
				Slider::new(&mut cfg.half_size[1], 0.05 ..= 5.0).text("Half Size Y"),
			).changed();

			changed 		|= ui.add(
				Slider::new(&mut cfg.half_size[2], 0.05 ..= 5.0).text("Half Size Z"),
			).changed();

			ui.checkbox		(&mut cfg.auto_offset, "Auto Offset Axle");

			if changed && cfg.auto_offset {
				let delta 	= cfg.half_size - cache.half_size;
				cfg.axle_offset += delta;
			}

			ui.separator	();

			changed 		|= ui.add(
				Slider::new(&mut cfg.axle_offset[0], -5.0 ..= 5.0).text("Axle Offset X"),
			).changed();
			
			changed 		|= ui.add(
				Slider::new(&mut cfg.axle_offset[1], -5.0 ..= 5.0).text("Axle Offset Y"),
			).changed();

			changed 		|= ui.add(
				Slider::new(&mut cfg.axle_offset[2],-5.0 ..= 5.0).text("Axle Offset Z"),
			).changed();

			ui.separator	();

			changed 		|= draw_phys_params_ui(ui, phys);
		}); // ui.vertical
	}); // ui.collapsing

	changed
}

fn draw_axle_params_ui(
	  ui					: &mut Ui
	, cfg					: &mut AxleConfig
	, phys					: &mut PhysicsConfig
	, section_name			: &str
) -> bool {

	let mut changed			= false;

	ui.collapsing(section_name, |ui| {
	ui.vertical(|ui| {

	let cache = cfg.clone();

	changed |= ui.add(
		Slider::new(&mut cfg.half_size[0], 0.05 ..= 5.0).text("Half Size X"),
	).changed();
	
	changed |= ui.add(
		Slider::new(&mut cfg.half_size[1], 0.05 ..= 5.0).text("Half Size Y"),
	).changed();

	changed |= ui.add(
		Slider::new(&mut cfg.half_size[2], 0.05 ..= 5.0).text("Half Size Z"),
	).changed();

	ui.checkbox(&mut cfg.auto_offset, "Auto Offset Wheel");

	if changed && cfg.auto_offset {
		let mut delta		= cfg.half_size - cache.half_size;
		// we care only for x axis
		delta				*= Vec3::X;
		cfg.wheel_offset 	+= delta;
	}

	ui.separator();

	changed |= ui.add(
		Slider::new(&mut cfg.wheel_offset[0], -5.0 ..= 5.0).text("Wheel Offset X"),
	).changed();
	
	changed |= ui.add(
		Slider::new(&mut cfg.wheel_offset[1], -5.0 ..= 5.0).text("Wheel Offset Y"),
	).changed();

	changed |= ui.add(
		Slider::new(&mut cfg.wheel_offset[2], -5.0 ..= 5.0).text("Wheel Offset Z"),
	).changed();

	ui.separator();

	changed |= draw_phys_params_ui(ui, phys);

	}); // ui.vertical
	}); // ui.collapsing

	changed
}

fn draw_wheel_params_ui(
	  ui					: &mut Ui
	, cfg					: &mut WheelConfig
	, cfg_phys				: &mut PhysicsConfig
	, section_name			: &str
) -> bool {

	let mut changed	= false;

	ui.collapsing(section_name, |ui| {
	ui.vertical(|ui| {

	changed |= ui.add(
		Slider::new(&mut cfg.r, 0.05 ..= 2.0).text("Radius"),
	).changed();

	changed |= ui.add(
		Slider::new(&mut cfg.hh, 0.05 ..= 2.0).text("Half Height"),
	).changed();

	changed |= draw_phys_params_ui(ui, cfg_phys);

	}); // ui.vertical
	}); // ui.collapsing

	changed
}

fn draw_acceleration_params_ui(
	ui				: &mut Ui,
	accel_cfg		: &mut AcceleratorConfig,
) -> bool {
	let mut changed = false;

	ui.collapsing("Acceleration".to_string(), |ui| {
	ui.vertical(|ui| {

	changed |= ui.add(
		Slider::new(&mut accel_cfg.vel_fwd, 0.05 ..= 400.0).text("Target Speed Forward"),
	).changed();

	changed |= ui.add(
		Slider::new(&mut accel_cfg.damping_fwd, 0.05 ..= 1000.0).text("Acceleration Damping Forward"),
	).changed();

	ui.add_space(1.0);
	
	changed |= ui.add(
		Slider::new(&mut accel_cfg.vel_bwd, 0.05 ..= 400.0).text("Target Speed Backward"),
	).changed();
	changed |= ui.add(
		Slider::new(&mut accel_cfg.damping_bwd, 0.05 ..= 1000.0).text("Acceleration Damping Backward"),
	).changed();

	ui.add_space(1.0);

	changed |= ui.add(
		Slider::new(&mut accel_cfg.damping_stop, 0.05 ..= 1000.0).text("Stopping Damping"),
	).changed();

	}); // ui.vertical
	}); // ui.collapsing

	changed
}

fn draw_steering_params_ui(
	ui				: &mut Ui,
	steer_cfg		: &mut SteeringConfig,
) -> bool{
	let mut changed = false;

	ui.collapsing("Steering".to_string(), |ui| {
	ui.vertical(|ui| {

	changed |= ui.add(
		Slider::new(&mut steer_cfg.angle, 0.05 ..= 180.0).text("Steering Angle"),
	).changed();
	changed |= ui.add(
		Slider::new(&mut steer_cfg.damping, 0.05 ..= 10000.0).text("Steering Damping"),
	).changed();
	changed |= ui.add(
		Slider::new(&mut steer_cfg.damping_release, 0.05 ..= 10000.0).text("Steering Release Damping"),
	).changed();
	changed |= ui.add(
		Slider::new(&mut steer_cfg.stiffness, 0.05 ..= 100000.0).text("Steering Stiffness"),
	).changed();
	changed |= ui.add(
		Slider::new(&mut steer_cfg.stiffness_release, 0.05 ..= 100000.0).text("Steering Release Stiffness"),
	).changed();

	}); // ui.vertical
	}); // ui.collapsing

	changed
}

pub fn ui_system(
	mut ui_context	: ResMut<EguiContext>,
	mut	game		: ResMut<GameState>,

	mut q_child		: Query<(
		&Parent,
		&mut Collider,
		&mut ColliderMassProperties,
		&mut Friction,
		&mut Restitution,
	)>,
    mut	q_parent	: Query<(
		&Vehicle::Part,
		&SideZ,
		&NameComponent,
		&MassProperties,
		&mut Damping,
	)>,
	mut q_phys_cfg	: Query<
		&mut PhysicsConfig
	>,
	mut q_body_cfg	: Query<
		&mut BodyConfig
	>,
	mut q_wheel_cfg	: Query<(
		Entity,
	 	&mut WheelConfig,
	 	&SideX,
		&SideZ
	)>,
	mut q_axle_cfg	: Query<(
		Entity,
	 	&mut AxleConfig,
	 	&SideX,
	 	&SideZ
	)>,
	mut q_accel_cfg	: Query<
	 	&mut AcceleratorConfig
	>,
	mut q_steer_cfg	: Query<
	 	&mut SteeringConfig
	>,
) {
	let body			= game.body.unwrap().entity;

	let mut accel_cfg	= q_accel_cfg.get_mut(body).unwrap();
	let mut steer_cfg	= q_steer_cfg.get_mut(body).unwrap();

	let window 			= egui::Window::new("Parameters");
	//let out = 
	window.show(ui_context.ctx_mut(), |ui| {
		ui.horizontal(|ui| {
			let mut body_phys_cfg = q_phys_cfg.get_mut(body).unwrap();
			if ui.add(toggle_switch::toggle(&mut body_phys_cfg.fixed))
				.on_hover_text("Put vehicle in the air and keep it fixed there.")
				.clicked()
			{
				game.body 			= Some(RespawnableEntity{ entity : game.body.unwrap().entity, respawn: true });
			}
			ui.label("Lifted Car Mode");
		});
		
		ui.separator();

		// common wheel/axle configs
		// ^^
		let (_, fl_wheel, _, _)		= q_wheel_cfg.get(game.wheels[FRONT_LEFT].unwrap().entity).unwrap();
		let (_, rl_wheel, _, _)		= q_wheel_cfg.get(game.wheels[REAR_LEFT].unwrap().entity).unwrap();
		let (_, fl_axle, _, _)		= q_axle_cfg.get(game.axles[FRONT_LEFT].unwrap().entity).unwrap();
		let (_, rl_axle, _, _)		= q_axle_cfg.get(game.axles[REAR_LEFT].unwrap().entity).unwrap();

		let mut front_wheel_phys_common = q_phys_cfg.get_mut(game.wheels[FRONT_LEFT].unwrap().entity).unwrap().clone();
		let mut rear_wheel_phys_common	= q_phys_cfg.get_mut(game.wheels[REAR_LEFT].unwrap().entity).unwrap().clone();
		let mut front_axle_phys_common	= q_phys_cfg.get_mut(game.axles[FRONT_LEFT].unwrap().entity).unwrap().clone();
		let mut rear_axle_phys_common	= q_phys_cfg.get_mut(game.axles[REAR_LEFT].unwrap().entity).unwrap().clone();

		let mut front_wheel_common 	= fl_wheel.clone();
		let mut rear_wheel_common 	= rl_wheel.clone();

		let mut front_axle_common 	= fl_axle.clone();
		let mut rear_axle_common 	= rl_axle.clone();

		let front_wheels_changed 	=
			draw_wheel_params_ui	(ui, &mut front_wheel_common, &mut front_wheel_phys_common, "Front Wheels");

		let rear_wheels_changed		=
			draw_wheel_params_ui	(ui, &mut rear_wheel_common, &mut rear_wheel_phys_common, "Rear Wheels");

		let front_axles_changed 	=
			draw_axle_params_ui		(ui, &mut front_axle_common, &mut front_axle_phys_common, "Front Axles");

		let rear_axles_changed		=
			draw_axle_params_ui		(ui, &mut rear_axle_common, &mut rear_axle_phys_common, "Rear Axles");

		let wheels_changed			= front_wheels_changed || rear_wheels_changed;
		let axles_changed			= front_axles_changed || rear_axles_changed;

		for (wheel, mut wheel_cfg, _sidex, sidez) in q_wheel_cfg.iter_mut() {
			let mut phys_cfg		= q_phys_cfg.get_mut(wheel).unwrap();
			if *sidez == SideZ::Front && front_wheels_changed {
				*wheel_cfg.as_mut() = front_wheel_common;
				*phys_cfg.as_mut() = front_wheel_phys_common;
			} else if *sidez == SideZ::Rear && rear_wheels_changed {
				*wheel_cfg.as_mut() = rear_wheel_common;
				*phys_cfg.as_mut() = rear_wheel_phys_common;
			}
		}

		for (axle, mut axle_cfg, _sidex, sidez) in q_axle_cfg.iter_mut() {
			let mut phys_cfg		= q_phys_cfg.get_mut(axle).unwrap();
			if *sidez == SideZ::Front && front_axles_changed {
				*axle_cfg.as_mut() 	= front_axle_common;
				*phys_cfg.as_mut() 	= front_axle_phys_common;
			}
			if *sidez == SideZ::Rear && rear_axles_changed {
				*axle_cfg.as_mut() 	= rear_axle_common;
				*phys_cfg.as_mut() 	= rear_axle_phys_common;
			}
		}

		let writeback_axle_collider = |
			  cfg					: &AxleConfig
			, phys					: &PhysicsConfig
			, collider				: &mut Mut<Collider>
			, mass_props_co			: &mut Mut<ColliderMassProperties>
		| {
			set_box_half_size		(cfg.half_size, collider);
			set_density				(phys.density, mass_props_co);
		};

		let writeback_wheel_collider = |
			  cfg					: &WheelConfig
			, phys					: &PhysicsConfig
			, collider				: &mut Mut<Collider>
			, mass_props_co			: &mut Mut<ColliderMassProperties>
			, friction				: &mut Mut<Friction>
			, restitution			: &mut Mut<Restitution>
			, damping				: &mut Mut<Damping>
		| {
			set_cylinder_hh			(cfg.hh, collider);
			set_cylinder_r			(cfg.r, collider);
			set_density				(phys.density, mass_props_co);
			set_friction			(phys.friction, friction);
			set_restitution			(phys.restitution, restitution);
			set_damping				(phys.lin_damping, phys.ang_damping, damping);
		};

		// write changes back to physics + per component ui 
		for (parent, mut collider, mut mass_props_co, mut friction, mut restitution) in q_child.iter_mut() {
			let (vehicle_part, sidez, name_comp, mass_props_rb, mut damping) = q_parent.get_mut(parent.0).unwrap();
			let name 				= &name_comp.name;
			let vp 					= *vehicle_part;

			if (vp == Vehicle::Part::Wheel && wheels_changed) || (vp == Vehicle::Part::Axle && axles_changed) {
				let mut phys		= q_phys_cfg.get_mut(parent.0).unwrap();
				phys.mass			= mass_props_rb.mass;
			}

			let mut body_changed	= false;
			let mut body_cfg 		= q_body_cfg.get_mut(body).unwrap();
			let 	body_cfg_cache 	= body_cfg.clone();
			let mut body_phys_cfg 	= q_phys_cfg.get_mut(body).unwrap();
			let		body_phys_cfg_cache = body_phys_cfg.clone();

			if vp == Vehicle::Part::Body {
				let mass			= mass_props_rb.mass;
				body_changed 		= draw_body_params_ui_collapsing(ui, name, [0.05, 100.0], mass, body_cfg.as_mut(), body_phys_cfg.as_mut(), "Body");

				draw_acceleration_params_ui	(ui, accel_cfg.as_mut());
				draw_steering_params_ui		(ui, steer_cfg.as_mut());
			} else if vp == Vehicle::Part::Wheel && *sidez == SideZ::Front && front_wheels_changed {
				writeback_wheel_collider(&front_wheel_common, &front_wheel_phys_common, &mut collider, &mut mass_props_co, &mut friction, &mut restitution, &mut damping);
			} else if vp == Vehicle::Part::Wheel && *sidez == SideZ::Rear && rear_wheels_changed {
				writeback_wheel_collider(&rear_wheel_common, &rear_wheel_phys_common, &mut collider, &mut mass_props_co, &mut friction, &mut restitution, &mut damping);
			} else if vp == Vehicle::Part::Axle && *sidez == SideZ::Front && front_axles_changed {
				writeback_axle_collider(&front_axle_common, &front_axle_phys_common, &mut collider, &mut mass_props_co);
			} else if vp == Vehicle::Part::Axle && *sidez == SideZ::Rear && rear_axles_changed {
				writeback_axle_collider(&rear_axle_common, &rear_axle_phys_common, &mut collider, &mut mass_props_co);
			}

			if axles_changed {
				// respawn child wheel
				for side_ref in WHEEL_SIDES {
					let side 		= *side_ref;
					game.wheels[side] = Some(RespawnableEntity{ entity : game.wheels[side].unwrap().entity, respawn: true });
				}
			}

			if body_changed {
				*mass_props_co	 	= ColliderMassProperties::Density(body_phys_cfg.density);
				body_phys_cfg.mass	= mass_props_rb.mass;

				damping.as_mut().linear_damping = body_phys_cfg.lin_damping;
				damping.as_mut().angular_damping = body_phys_cfg.ang_damping;

				let cuboid 			= collider.as_cuboid_mut().unwrap();
				cuboid.raw.half_extents = body_cfg.half_size.into();

				for side_ref in WHEEL_SIDES {
					let side 		= *side_ref;
					// respawn
					game.axles[side] = Some(RespawnableEntity{ entity : game.axles[side].unwrap().entity, respawn: true }); // TODO: hide the ugly
					game.wheels[side] = Some(RespawnableEntity{ entity : game.wheels[side].unwrap().entity, respawn: true });
				}

				// respawn
				if body_phys_cfg_cache.fixed != body_phys_cfg.fixed {
					game.body 		= Some(RespawnableEntity{ entity : game.body.unwrap().entity, respawn: true })
				}
			}
		}

		ui.separator();

		if (ui.button("Save Vehicle")).clicked() {
			let mut dialog			= FileDialog::save_file(None);
			dialog.open				();
			game.save_veh_dialog	= Some(dialog);
		}

		if (ui.button("Load Vehicle")).clicked() {
			let mut dialog			= FileDialog::open_file(None);
			dialog.open				();
			game.load_veh_dialog 	= Some(dialog);
		}

		if let Some(dialog) = &mut game.save_veh_dialog {
			if dialog.show(&ui.ctx()).selected() {
				game.save_veh_file 	= dialog.path();
			}
		}

		if let Some(dialog) = &mut game.load_veh_dialog {
			if dialog.show(&ui.ctx()).selected() {
				game.load_veh_file 	= dialog.path();
			}
		}

		ui.separator();

		if ui.button("Respawn Vehicle").clicked() {
			game.body 				= Some(RespawnableEntity{ entity : game.body.unwrap().entity, respawn: true });
		}

	});

// uncomment when we need to catch a closed window
//	match out {
//		Some(response) => {
//			if response.inner == None { println!("PEWPEWPEWPEW") }; 
//		}
//		_ => ()
//	}
}