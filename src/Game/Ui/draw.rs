use bevy			::	prelude :: *;
use bevy_egui		::	egui :: { Slider, Ui };

use super			::	{ * };

pub fn density_param(
		ui					: &mut Ui,
		range				: [f32; 2],
		density				: &mut f32,
		mass				: f32,
) -> bool {
	ui.add(
		Slider::new			(density, std::ops::RangeInclusive::new(range[0], range[1])).text(format!("Density (Mass {:.3})", mass))
	).changed()
}

pub fn phys_params(
	  ui					: &mut Ui
	, cfg					: &mut PhysicsConfig
) -> bool {

	let mut changed	= false;

	ui.collapsing("Physics", |ui| {
	ui.vertical(|ui| {

	changed |= density_param(ui, [0.05, 2000.0], &mut cfg.density, cfg.mass);

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

pub fn body_params_collapsing(
	ui						: &mut Ui,
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

			changed 		|= phys_params(ui, phys);
		}); // ui.vertical
	}); // ui.collapsing

	changed
}

pub fn axle_params(
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

	changed |= phys_params(ui, phys);

	}); // ui.vertical
	}); // ui.collapsing

	changed
}

pub fn wheel_params(
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

	changed |= phys_params(ui, cfg_phys);

	}); // ui.vertical
	}); // ui.collapsing

	changed
}

pub fn acceleration_params(
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

pub fn steering_params(
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