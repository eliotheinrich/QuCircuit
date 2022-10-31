use std::collections::HashMap;
use std::fs;


pub struct DataFrame {
	int_params: HashMap<String, i32>,
	float_params: HashMap<String, f32>,
	data: HashMap<String, Vec<f32>>,
}

impl DataFrame {
	pub fn new() -> DataFrame {
		let int_params: HashMap<String, i32> = HashMap::new();
		let float_params: HashMap<String, f32> = HashMap::new();
		let data: HashMap<String, Vec<f32>> = HashMap::new();
		return DataFrame { int_params: int_params, float_params: float_params, data: data };
	}

	pub fn add_int_param(&mut self, key: &str, val: i32) {
		self.int_params.insert(String::from(key), val);
	}

	pub fn add_float_param(&mut self, key: &str, val: f32) {
		self.float_params.insert(String::from(key), val);
	}

	pub fn push_data(&mut self, key: &str, val: f32) {
		self.data.get_mut(key).unwrap().push(val);
	}

	pub fn add_data(&mut self, key: &str) {
		self.data.insert(String::from(key), Vec::new());
	}

	pub fn to_string(&self) -> String {
		let mut s: String = String::new();
		if !self.int_params.is_empty() {
			for (key, val) in self.int_params.iter() {
				s.push_str(&format!("{}: {}\t", key, val));
			}
		}
		if !self.float_params.is_empty() {
			for (key, val) in self.float_params.iter() {
				s.push_str(&format!("{}: {} \t", key, val));
			}
		}

		if !self.data.is_empty() {
			let mut vs: String;
			for (key, vec) in self.data.iter() {
				vs = String::new();
				if !vec.is_empty() {
					for f in vec {
						vs.push_str(&format!("{} ", f.to_string()));
					}
					vs.pop();
				}
				s.push_str(&format!("{}: {}\t", key, vs));
			} 
			s.pop();
		}

		return s;
	}

	pub fn write_dataframes(filename: &str, dataframes: Vec<DataFrame>) {
		let mut s: String = String::new();
		for df in &dataframes {
			s.push_str(&format!("{}\n", df.to_string()));
		}
		fs::write(filename, s);
	}
}