use bevy::prelude::*;

use std::io::prelude::*;
use std::fs::File;
use std::path::{ Path, PathBuf };

use ron::ser::{ to_string_pretty, PrettyConfig };

use directories :: { BaseDirs, UserDirs, ProjectDirs };

use super::*;

const VERSION_STR : &str = "version";

pub fn save_vehicle_config_system(
	mut game: ResMut<GameState>,

	q_body	: Query	<(Entity, &BodyConfig, &PhysicsConfig)>,
	q_axle	: Query	<(Entity, &AxleConfig, &PhysicsConfig)>,
	q_wheel	: Query	<(Entity, &WheelConfig, &PhysicsConfig)>,
	q_accel	: Query <(Entity, &AcceleratorConfig)>,
	q_steer	: Query <(Entity, &SteeringConfig)>,
) {
	if game.save_veh_file.is_none() { return; }

	let mut veh_cfg = self::Config::default();

	match game.body {
		Some(re) => {
			let (_, accel) = q_accel.get(re.entity).unwrap();
			veh_cfg.accel = Some(*accel);
			let (_, steer) = q_steer.get(re.entity).unwrap();
			veh_cfg.steer = Some(*steer);

			let (_, body, phys) = q_body.get(re.entity).unwrap();
			veh_cfg.body = Some(*body);
			veh_cfg.bophys = Some(*phys);
		},
		_ => (),
	};

	for i in 0..WHEELS_MAX {
		match game.axles[i] {
			Some(re) => {
				let (_, axle, phys) = q_axle.get(re.entity).unwrap();
				veh_cfg.axles[i] = Some(*axle);
				veh_cfg.axphys[i] = Some(*phys)
			},
			_ => ()
		};

		match game.wheels[i] {
			Some(re) => {
				let (_, wheel, phys) = q_wheel.get(re.entity).unwrap();
				veh_cfg.wheels[i] = Some(*wheel);
				veh_cfg.whphys[i] = Some(*phys)
			},
			_ => ()
		}
	}

	let pretty = PrettyConfig::new()
		.depth_limit(5)
		.enumerate_arrays(true)
		.separate_tuple_members(true);

	let version_str = format!("{}: {}\n", VERSION_STR, self::Config::version());
	let serialized	= to_string_pretty(&veh_cfg, pretty).expect("Serialization failed");
	let save_content = [
		  version_str
		, serialized
	].concat();

	let mut save_name = file_path_to_string(&game.save_veh_file);
	// if let Some(proj_dirs) = ProjectDirs::from("lol", "Gryazevicki Inc",  "Gryazevichki") {
	// 	save_name = [ proj_dirs.config_dir(), &save_name ].concat();
	// 	// Lin: /home/user/.config/gryazevichki
	// 	// Win: C:\Users\User\AppData\Roaming\Gryazevicki Inc\Gryazevicki\config
	// 	// Mac: /Users/User/Library/Application Support/lol.Gryazevicki-Inc.Gryazevicki
	// }

	let path = Path::new(&save_name);
	let display = path.display();

	let mut file = match File::create(&path) {
		Err(why) => panic!("couldn't create {}: {}", display, why),
		Ok(file) => file,
	};

	match file.write_all(save_content.as_bytes()) {
		Err(why) => panic!("couldn't write to {}: {}", display, why),
		Ok(_) => println!("successfully wrote to {}", display),
	}

	game.save_veh_file = None;
}

pub fn load_vehicle_config_system(
	mut game	: ResMut<GameState>,

	mut q_phys	: Query <&mut PhysicsConfig>,
	mut q_body	: Query	<&mut BodyConfig>,
	mut q_axle	: Query	<&mut AxleConfig>,
	mut q_wheel	: Query <&mut WheelConfig>,
	mut q_accel	: Query <&mut AcceleratorConfig>,
	mut q_steer	: Query <&mut SteeringConfig>,
) {
	if game.load_veh_file.is_none() { return; }

	let mut veh_cfg	= load_vehicle_config(&game.load_veh_file).unwrap();

	game.load_veh_file = None;

	match game.body {
		Some(re) => {
			match q_body.get_mut(re.entity) {
				Ok(mut body) => *body = veh_cfg.body.unwrap_or_default(), _ => (),
			}
			match q_phys.get_mut(re.entity) {
				Ok(mut phys) => *phys = veh_cfg.bophys.unwrap_or_default(), _ => (),
			}
			match q_accel.get_mut(re.entity) {
				Ok(mut accel) => *accel = veh_cfg.accel.unwrap_or_default(), _ => (),
			}
			match q_steer.get_mut(re.entity) {
				Ok(mut steer) => *steer = veh_cfg.steer.unwrap_or_default(), _ => (),
			}
		},
		_ => (),
	};

	for i in 0..WHEELS_MAX {
		match game.axles[i] {
			Some(re) => {
				match q_axle.get_mut(re.entity) {
					Ok(mut axle) => *axle = veh_cfg.axles[i].unwrap_or_default(), _ => (),
				}
				match q_phys.get_mut(re.entity) {
					Ok(mut phys) => *phys = veh_cfg.axphys[i].unwrap_or_default(), _ => (),
				}
			},
			_ => ()
		};

		match game.wheels[i] {
			Some(re) => {
				match q_wheel.get_mut(re.entity) {
					Ok(mut wheel) => *wheel = veh_cfg.wheels[i].unwrap_or_default(), _ => (),
				}
				match q_phys.get_mut(re.entity) {
					Ok(mut phys) => *phys = veh_cfg.whphys[i].unwrap_or_default(), _ => (),
				}
			},
			_ => ()
		}
	}

	// respawn
	game.body = Some(RespawnableEntity{ entity : game.body.unwrap().entity, respawn: true });
}

pub fn load_vehicle_config(
	vehicle_config_file : &Option<PathBuf> 
) -> Option<self::Config> {
	let load_name 	= file_path_to_string(vehicle_config_file);
	let path 		= Path::new(&load_name);
	let display 	= path.display();

	let mut file = match File::open(&path) {
		Err(why) 	=> { println!("couldn't open {}: {}", display, why); return None; },
		Ok(file) 	=> file,
	};

	let mut save_content = String::new();
	match file.read_to_string(&mut save_content) {
		Err(why)	=> { println!("couldn't read {}: {}", display, why); return None; },
		Ok(_) 		=> println!("Opened file {} for reading", display.to_string()),
	}

	let mut lines	= save_content.lines();
	let line 		= match lines.next() {
		Some(l)		=> l,
		None		=> { println!("{0} not found! Config should start with {0}", VERSION_STR); return None; }
	};
	
	let version_value : String = match line.split_terminator(':').last() {
		Some(v)		=> v.chars().filter(|c| c.is_digit(10)).collect(),
		None		=> { println!("{0} value not found! Config should start with \"{0}: {1}\"", VERSION_STR, self::Config::version()); return None; }
	};

	let version 	= match version_value.parse::<u32>() {
		Ok(v) 		=> v,
		Err(why) 	=> { println!("Failed to parse version value ({})! Reason: {}", version_value, why); return None; },
	};

	if version > self::Config::version() {
		println!	("Invalid config version! Expected: <={} found: {}", self::Config::version(), version);
		return 		None;
	}

	let pos			= match save_content.find('(') {
		Some(p)		=> p - 1, // -1 to capture the brace as well lower, see save_content.get
		None		=> { println!("Failed to find first opening brace \"(\". Most likely invalid format or corrupted file!"); return None; }
	};

	save_content	= match save_content.get(pos..) {
		Some(c)		=> c.to_string(),
		None		=> { return None; }
	};

	let veh_cfg: self::Config = ron::from_str(save_content.as_str()).unwrap();
	
	Some(veh_cfg)
}

fn file_path_to_string(buf: &Option<PathBuf>) -> String {
	match buf {
		Some(path) => path.display().to_string(),
		None => String::from(""),
	}
}