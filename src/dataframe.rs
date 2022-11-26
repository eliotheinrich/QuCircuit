use std::collections::HashMap;
use std::fs;
use serde::{Serialize, Deserialize};
use crate::{quantum_state::QuantumState, 
		 	quantum_chp_state::QuantumCHPState, 
			quantum_graph_state::QuantumGraphState, 
			quantum_vector_state::QuantumVectorState};

use rayon::prelude::*;

#[derive(Serialize, Deserialize)]
pub enum DataField {
	Int(i32),
	Float(f32),
	Data(Vec<f32>),
	QuantumCHPState(QuantumCHPState),
	QuantumGraphState(QuantumGraphState),
	QuantumVectorState(QuantumVectorState),
}

#[derive(Serialize, Deserialize)]
pub struct DataFrame {
	pub slides: Vec<DataSlide>
}

impl DataFrame {
	pub fn new() -> Self {
		return DataFrame { slides: Vec::new() };
	}

	pub fn from(mut slides: Vec<DataSlide>) -> Self {
		let mut df = DataFrame::new();
		for i in 0..slides.len() {
			df.add_slide(slides.pop().unwrap());
		}
		return df;
	}

	pub fn add_slide(&mut self, slide: DataSlide) {
		self.slides.push(slide);
	}

	pub fn save_json(&self, filename: String) {
		let json = serde_json::to_string(&self).unwrap();
		fs::write(filename, json);
	}
}

#[derive(Serialize, Deserialize)]
pub struct DataSlide {
	data: HashMap<String, DataField>,
}

impl DataSlide {
	pub fn new() -> DataSlide {
		let data: HashMap<String, DataField> = HashMap::new(); 
		return DataSlide { data: data };
	}

	pub fn add_int_param(&mut self, key: &str, val: i32) {
		self.data.insert(String::from(key), DataField::Int(val));
	}

	pub fn add_float_param(&mut self, key: &str, val: f32) {
		self.data.insert(String::from(key), DataField::Float(val));
	}

	pub fn push_data(&mut self, key: &str, val: f32) {
		match self.data.get_mut(key).unwrap() {
			DataField::Data(v) => v.push(val),
			_ => ()
		}
	}

	pub fn add_data(&mut self, key: &str) {
		self.data.insert(String::from(key), DataField::Data(Vec::new()));
	}


	pub fn add_state<Q: QuantumState>(&mut self, key: &str, state: &Q) {
		self.data.insert(String::from(key), state.to_datafield());
	}

	pub fn get_val(&self, key: &str) -> &DataField {
		return &self.data[key];
	}

	pub fn contains_key(&self, key: &str) -> bool {
		return self.data.contains_key(key);
	}

	pub fn unwrap_int(&self, key: &str) -> i32 {
		match self.data[key] {
			DataField::Int(x) => x,
			_ => panic!()
		}
	}

	pub fn unwrap_float(&self, key: &str) -> f32 {
		match self.data[key] {
			DataField::Float(x) => x,
			_ => panic!()
		}
	}

	pub fn unwrap_data(&self, key: &str) -> &Vec<f32> {
		match &self.data[key] {
			DataField::Data(x) => x,
			_ => panic!()
		}
	}
}