use std::{fs, str::FromStr};

pub struct ImageArrayState {
	pub path: String,
	pub textures: Vec<String>,
}

impl ImageArrayState {
	pub fn load(
		path_abs: &str,
	) -> Self {
		let mut ias_path = String::from_str(path_abs).unwrap();
		ias_path += "/ias.txt";
		let ias = fs::read(ias_path).unwrap();
		let mut mid_buffer = String::with_capacity(32);
		let mut textures = Vec::with_capacity(1024);
		for c in ias {
			let c = c as char;
			match c {
				'\n' => {
					textures.push(mid_buffer.clone());
					mid_buffer.clear();
				},
				_ => {
					mid_buffer.extend_one(c);
				},
			}
		}
		Self {
			path: String::from_str(path_abs).unwrap(),
			textures,
		}
	}
	
	pub fn store(
		self,
	) {
		let mut ias_path = String::from_str(&self.path).unwrap();
		ias_path += "/ias.txt";
		let mut ias: Vec<u8> = Vec::with_capacity(4096);
		for texture in self.textures {
			ias.extend_from_slice(texture.as_bytes());
			ias.extend_one('\n' as u8);
		}
		fs::write(ias_path, ias).expect("failed to write IAS");
	}
}