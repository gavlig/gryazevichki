use bevy			:: prelude :: *;
use bevy_rapier3d	:: prelude :: *;
use bevy_egui		:: { egui, EguiContext };
use bevy_mod_picking:: { * };

use super           :: egui_ext	:: FileDialog;
use super           :: egui_ext	:: toggle_switch;

use crate			:: game :: *;
use crate			:: vehicle;
use crate			:: vehicle :: { WHEEL_SIDES, FRONT_LEFT, REAR_LEFT };
use crate			:: herringbone;
use crate			:: herringbone :: Herringbone2TileFilterInfo;
use super			:: { writeback };
use super			:: { draw };

pub fn vehicle_params_ui_system(
	// mut ui_context	: ResMut<EguiContext>,
	// mut	game		: ResMut<GameState>,

	// mut q_child		: Query<(
	// 	&Parent,
	// 	&mut Collider,
	// 	&mut ColliderMassProperties,
	// 	&mut Friction,
	// 	&mut Restitution,
	// )>,
	// mut	q_parent	: Query<(
	// 	&Vehicle::Part,
	// 	&SideZ,
	// 	&MassProperties,
	// 	&mut Damping,
	// )>,
	// mut q_phys_cfg	: Query<
	// 	&mut PhysicsConfig
	// >,
	// mut q_body_cfg	: Query<
	// 	&mut Vehicle::BodyConfig
	// >,
	// mut q_wheel_cfg	: Query<(
	// 	Entity,
	//  	&mut Vehicle::WheelConfig,
	//  	&SideX,
	// 	&SideZ
	// )>,
	// mut q_axle_cfg	: Query<(
	// 	Entity,
	//  	&mut Vehicle::AxleConfig,
	//  	&SideX,
	//  	&SideZ
	// )>,
	// mut q_accel_cfg	: Query<
	//  	&mut Vehicle::AcceleratorConfig
	// >,
	// mut q_steer_cfg	: Query<
	//  	&mut Vehicle::SteeringConfig
	// >,
) {
	// if true {
	// 	return;
	// }

	// let body			= game.body.unwrap().entity;

	// let mut accel_cfg	= q_accel_cfg.get_mut(body).unwrap();
	// let mut steer_cfg	= q_steer_cfg.get_mut(body).unwrap();

	// let window 			= egui::Window::new("Parameters").vscroll(true);
	// //let out = 
	// window.show(ui_context.ctx_mut(), |ui| {
	// 	ui.horizontal(|ui| {
	// 		let mut body_phys_cfg = q_phys_cfg.get_mut(body).unwrap();
	// 		if ui.add(toggle_switch::toggle(&mut body_phys_cfg.fixed))
	// 			.on_hover_text("Put vehicle in the air and keep it fixed there.")
	// 			.clicked()
	// 		{
	// 			game.body 			= Some(RespawnableEntity{ entity : game.body.unwrap().entity, respawn: true });
	// 		}
	// 		ui.label("Lifted Car Mode");
	// 	});
		
	// 	ui.separator();

	// 	// common wheel/axle configs
	// 	// ^^
	// 	let (_, fl_wheel, _, _)		= q_wheel_cfg.get(game.wheels[FRONT_LEFT].unwrap().entity).unwrap();
	// 	let (_, rl_wheel, _, _)		= q_wheel_cfg.get(game.wheels[REAR_LEFT].unwrap().entity).unwrap();
	// 	let (_, fl_axle, _, _)		= q_axle_cfg.get(game.axles[FRONT_LEFT].unwrap().entity).unwrap();
	// 	let (_, rl_axle, _, _)		= q_axle_cfg.get(game.axles[REAR_LEFT].unwrap().entity).unwrap();

	// 	let mut front_wheel_phys_common = q_phys_cfg.get_mut(game.wheels[FRONT_LEFT].unwrap().entity).unwrap().clone();
	// 	let mut rear_wheel_phys_common	= q_phys_cfg.get_mut(game.wheels[REAR_LEFT].unwrap().entity).unwrap().clone();
	// 	let mut front_axle_phys_common	= q_phys_cfg.get_mut(game.axles[FRONT_LEFT].unwrap().entity).unwrap().clone();
	// 	let mut rear_axle_phys_common	= q_phys_cfg.get_mut(game.axles[REAR_LEFT].unwrap().entity).unwrap().clone();

	// 	let mut front_wheel_common 	= fl_wheel.clone();
	// 	let mut rear_wheel_common 	= rl_wheel.clone();

	// 	let mut front_axle_common 	= fl_axle.clone();
	// 	let mut rear_axle_common 	= rl_axle.clone();

	// 	let front_wheels_changed 	=
	// 		draw::wheel_params		(ui, &mut front_wheel_common, &mut front_wheel_phys_common, "Front Wheels");

	// 	let rear_wheels_changed		=
	// 		draw::wheel_params		(ui, &mut rear_wheel_common, &mut rear_wheel_phys_common, "Rear Wheels");

	// 	let front_axles_changed 	=
	// 		draw::axle_params		(ui, &mut front_axle_common, &mut front_axle_phys_common, "Front Axles");

	// 	let rear_axles_changed		=
	// 		draw::axle_params		(ui, &mut rear_axle_common, &mut rear_axle_phys_common, "Rear Axles");

	// 	let wheels_changed			= front_wheels_changed || rear_wheels_changed;
	// 	let axles_changed			= front_axles_changed || rear_axles_changed;

	// 	for (wheel, mut wheel_cfg, _sidex, sidez) in q_wheel_cfg.iter_mut() {
	// 		let mut phys_cfg		= q_phys_cfg.get_mut(wheel).unwrap();
	// 		if *sidez == SideZ::Front && front_wheels_changed {
	// 			*wheel_cfg.as_mut() = front_wheel_common;
	// 			*phys_cfg.as_mut() 	= front_wheel_phys_common;
	// 		} else if *sidez == SideZ::Rear && rear_wheels_changed {
	// 			*wheel_cfg.as_mut() = rear_wheel_common;
	// 			*phys_cfg.as_mut() 	= rear_wheel_phys_common;
	// 		}
	// 	}

	// 	for (axle, mut axle_cfg, _sidex, sidez) in q_axle_cfg.iter_mut() {
	// 		let mut phys_cfg		= q_phys_cfg.get_mut(axle).unwrap();
	// 		if *sidez == SideZ::Front && front_axles_changed {
	// 			*axle_cfg.as_mut() 	= front_axle_common;
	// 			*phys_cfg.as_mut() 	= front_axle_phys_common;
	// 		}
	// 		if *sidez == SideZ::Rear && rear_axles_changed {
	// 			*axle_cfg.as_mut() 	= rear_axle_common;
	// 			*phys_cfg.as_mut() 	= rear_axle_phys_common;
	// 		}
	// 	}

	// 	let writeback_axle_collider = |
	// 		  cfg					: &Vehicle::AxleConfig
	// 		, phys					: &PhysicsConfig
	// 		, collider				: &mut Mut<Collider>
	// 		, mass_props_co			: &mut Mut<ColliderMassProperties>
	// 	| {
	// 		writeback::box_half_size		(cfg.half_size, collider);
	// 		writeback::density				(phys.density, mass_props_co);
	// 	};

	// 	let writeback_wheel_collider = |
	// 		  cfg					: &Vehicle::WheelConfig
	// 		, phys					: &PhysicsConfig
	// 		, collider				: &mut Mut<Collider>
	// 		, mass_props_co			: &mut Mut<ColliderMassProperties>
	// 		, friction				: &mut Mut<Friction>
	// 		, restitution			: &mut Mut<Restitution>
	// 		, damping				: &mut Mut<Damping>
	// 	| {
	// 		writeback::cylinder_hh	(cfg.hh, collider);
	// 		writeback::cylinder_r	(cfg.r, collider);
	// 		writeback::density		(phys.density, mass_props_co);
	// 		writeback::friction		(phys.friction, friction);
	// 		writeback::restitution	(phys.restitution, restitution);
	// 		writeback::damping		(phys.lin_damping, phys.ang_damping, damping);
	// 	};

	// 	// write changes back to physics + per component ui 
	// 	for (parent, mut collider, mut mass_props_co, mut friction, mut restitution) in q_child.iter_mut() {
	// 		let (vehicle_part, sidez, mass_props_rb, mut damping) = q_parent.get_mut(parent.0).unwrap();
	// 		let vp 					= *vehicle_part;

	// 		if (vp == Vehicle::Part::Wheel && wheels_changed) || (vp == Vehicle::Part::Axle && axles_changed) {
	// 			let mut phys		= q_phys_cfg.get_mut(parent.0).unwrap();
	// 			phys.mass			= mass_props_rb.mass;
	// 		}

	// 		let mut body_changed	= false;
	// 		let mut body_cfg 		= q_body_cfg.get_mut(body).unwrap();
	// 		let mut body_phys_cfg 	= q_phys_cfg.get_mut(body).unwrap();
	// 		let		body_phys_cfg_cache = body_phys_cfg.clone();

	// 		if vp == Vehicle::Part::Body {
	// 			body_changed 		= draw::body_params_collapsing(ui, body_cfg.as_mut(), body_phys_cfg.as_mut(), "Body");

	// 			draw::acceleration_params	(ui, accel_cfg.as_mut());
	// 			draw::steering_params		(ui, steer_cfg.as_mut());
	// 		} else if vp == Vehicle::Part::Wheel && *sidez == SideZ::Front && front_wheels_changed {
	// 			writeback_wheel_collider(&front_wheel_common, &front_wheel_phys_common, &mut collider, &mut mass_props_co, &mut friction, &mut restitution, &mut damping);
	// 		} else if vp == Vehicle::Part::Wheel && *sidez == SideZ::Rear && rear_wheels_changed {
	// 			writeback_wheel_collider(&rear_wheel_common, &rear_wheel_phys_common, &mut collider, &mut mass_props_co, &mut friction, &mut restitution, &mut damping);
	// 		} else if vp == Vehicle::Part::Axle && *sidez == SideZ::Front && front_axles_changed {
	// 			writeback_axle_collider(&front_axle_common, &front_axle_phys_common, &mut collider, &mut mass_props_co);
	// 		} else if vp == Vehicle::Part::Axle && *sidez == SideZ::Rear && rear_axles_changed {
	// 			writeback_axle_collider(&rear_axle_common, &rear_axle_phys_common, &mut collider, &mut mass_props_co);
	// 		}

	// 		if axles_changed {
	// 			// respawn child wheel
	// 			for side_ref in WHEEL_SIDES {
	// 				let side 		= *side_ref;
	// 				game.wheels[side] = Some(RespawnableEntity{ entity : game.wheels[side].unwrap().entity, respawn: true });
	// 			}
	// 		}

	// 		if body_changed {
	// 			*mass_props_co	 	= ColliderMassProperties::Density(body_phys_cfg.density);
	// 			body_phys_cfg.mass	= mass_props_rb.mass;

	// 			damping.as_mut().linear_damping = body_phys_cfg.lin_damping;
	// 			damping.as_mut().angular_damping = body_phys_cfg.ang_damping;

	// 			let cuboid 			= collider.as_cuboid_mut().unwrap();
	// 			cuboid.raw.half_extents = body_cfg.half_size.into();

	// 			for side_ref in WHEEL_SIDES {
	// 				let side 		= *side_ref;
	// 				// respawn
	// 				game.axles[side] = Some(RespawnableEntity{ entity : game.axles[side].unwrap().entity, respawn: true }); // TODO: hide the ugly
	// 				game.wheels[side] = Some(RespawnableEntity{ entity : game.wheels[side].unwrap().entity, respawn: true });
	// 			}

	// 			// respawn
	// 			if body_phys_cfg_cache.fixed != body_phys_cfg.fixed {
	// 				game.body 		= Some(RespawnableEntity{ entity : game.body.unwrap().entity, respawn: true })
	// 			}
	// 		}
	// 	}

	// 	ui.separator();

	// 	if (ui.button("Save Vehicle")).clicked() {
	// 		let mut dialog			= FileDialog::save_file(None);
	// 		dialog.open				();
	// 		game.save_veh_dialog	= Some(dialog);
	// 	}

	// 	if (ui.button("Load Vehicle")).clicked() {
	// 		let mut dialog			= FileDialog::open_file(None);
	// 		dialog.open				();
	// 		game.load_veh_dialog 	= Some(dialog);
	// 	}

	// 	if let Some(dialog) = &mut game.save_veh_dialog {
	// 		if dialog.show(&ui.ctx()).selected() {
	// 			game.save_veh_file 	= dialog.path();
	// 		}
	// 	}

	// 	if let Some(dialog) = &mut game.load_veh_dialog {
	// 		if dialog.show(&ui.ctx()).selected() {
	// 			game.load_veh_file 	= dialog.path();
	// 		}
	// 	}

	// 	ui.separator();

	// 	if ui.button("Respawn Vehicle").clicked() {
	// 		game.body 				= Some(RespawnableEntity{ entity : game.body.unwrap().entity, respawn: true });
	// 	}

	// });

// uncomment when we need to catch a closed window
//	match out {
//		Some(response) => {
//			if response.inner == None { println!("PEWPEWPEWPEW") }; 
//		}
//		_ => ()
//	}
}

pub fn coords_on_hover_ui_system(
	mut windows			: ResMut<Windows>,
	mut ui_context		: ResMut<EguiContext>,
		q_hover_tile	: Query<(&Hover, &herringbone::BrickRoadProgressState, Option<&Herringbone2TileFilterInfo>, &Transform)>,
		q_hover			: Query<(&Hover, &Transform, &GlobalTransform), Without<Tile>>,
) {
	if q_hover.is_empty() && q_hover_tile.is_empty() {
		return;
	}

	let window = windows.get_primary_mut().unwrap();

	for (hover, state, filter_info, tform) in q_hover_tile.iter() {
		if !hover.hovered() {
			continue;
		}

		// crashes randomly when we drag something outside window
		if !window.physical_cursor_position().is_none() {
			egui::show_tooltip_at_pointer(ui_context.ctx_mut(), egui::Id::new("herr"), |ui| {
				ui.label(format!("#[{} {}] t: {:.3} x: {:.3} z: {:.3}", state.column_id, state.row_id, state.t, tform.translation.x, tform.translation.z));

				match filter_info {
					Some(ref fi) => {
						ui.label(
							egui::RichText::new(format!(
								"\n\
								Filter Info:\n\
								spline_p               : [{:>6.3} {:>6.3} {:>6.3}]\n\
								road_halfwidth_rotated : [{:>6.3} {:>6.3} {:>6.3}]\n\
								left_border            : [{:>6.3} {:>6.3} {:>6.3}]\n\
								x                      : [{:.3}]\n\
								right_border           : [{:>6.3} {:>6.3} {:>6.3}]\n\
								",
								fi.spline_p.x, fi.spline_p.y, fi.spline_p.z,
								fi.road_halfwidth_rotated.x, fi.road_halfwidth_rotated.y, fi.road_halfwidth_rotated.z,
								fi.left_border.x, fi.left_border.y, fi.left_border.z,
								fi.right_border.x, fi.right_border.y, fi.right_border.z,
								fi.x
							))
						.text_style(egui::TextStyle::Monospace));	
					},
				_ => (),
				}
			});
		}
	}

	for (hover, tform, gtform) in q_hover.iter() {
		if !hover.hovered() {
			continue;
		}
	
		// crashes randomly when we drag something outside window
		if !window.physical_cursor_position().is_none() {
			egui::show_tooltip_at_pointer(ui_context.ctx_mut(), egui::Id::new("hover"), |ui| {
				ui.label(format!("lx: {:.3} lz: {:.3} gx: {:.3} gz: {:.3}", tform.translation.x, tform.translation.z, gtform.translation.x, gtform.translation.z));
			});
		}
	}
}