use std::collections::HashMap;
use std::fs;
use serde::{Serialize, Deserialize, Serializer, ser::SerializeTuple};
use crate::{quantum_chp_state::QuantumCHPState, 
			quantum_graph_state::QuantumGraphState, 
			quantum_vector_state::QuantumVectorState};

use rayon::prelude::*;

#[derive(Deserialize, Clone)]
pub struct Sample {
	pub mean: f32,
	pub std: f32,
	pub num_samples: usize,
}

impl Sample {
	pub fn new(s: f32) -> Sample {
		return Sample { mean: s, std: 0., num_samples: 1 };
	}

	pub fn from(vec: &Vec<f32>) -> Sample {
		let num_samples: usize = vec.len();
		let mut s: f32 = 0.;
		let mut s2: f32 = 0.;

		for v in vec {
			s += v;
			s2 += v*v;
		}
	
		s /= num_samples as f32;
		s2 /= num_samples as f32;

		let mean: f32 = s;
		let std: f32 = (s2 - s*s).powf(0.5);

		return Sample { mean: mean, std: std, num_samples: num_samples }
	}

	pub fn combine(&self, other: &Sample) -> Sample {
		let combined_samples: usize = self.num_samples + other.num_samples;
		let combined_mean: f32 = ((self.num_samples as f32)*self.mean
								+ (other.num_samples as f32)*other.mean)
								/ (combined_samples as f32);
		let combined_std: f32 = (((self.num_samples as f32)* (self.std.powi(2) +  (self.mean -  combined_mean).powi(2)) 
							    + (other.num_samples as f32)*(other.std.powi(2) + (other.mean - combined_mean).powi(2)))
							    / (combined_samples as f32)).powf(0.5);
		return Sample { mean: combined_mean, std: combined_std, num_samples: combined_samples };
	}
}

impl Serialize for Sample {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_tuple(3)?;
		seq.serialize_element(&self.mean)?;
		seq.serialize_element(&self.std)?;
		seq.serialize_element(&self.num_samples)?;
		seq.end()
    }
}

// Code for managing output data in runs
#[derive(Serialize, Deserialize, Clone)]
pub enum DataField {
	Int(i32),
	Float(f32),
	Data(Vec<Sample>),
}

#[derive(Serialize, Deserialize)]
pub struct DataFrame {
	pub params: HashMap<String, DataField>,
	pub slides: Vec<DataSlide>
}

impl DataFrame {
	pub fn new() -> Self {
		return DataFrame { params: HashMap::new(), slides: Vec::new() };
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

#[derive(Serialize, Deserialize, Clone)]
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

	pub fn push_data(&mut self, key: &str, val: Sample) {
		match self.data.get_mut(key).unwrap() {
			DataField::Data(v) => v.push(val),
			_ => ()
		}
	}

	pub fn add_data(&mut self, key: &str) {
		self.data.insert(String::from(key), DataField::Data(Vec::new()));
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

	pub fn unwrap_data(&self, key: &str) -> &Vec<Sample> {
		match &self.data[key] {
			DataField::Data(x) => x,
			_ => panic!()
		}
	}
}

// Code for managing parallel computation of many configurable runs
pub enum Simulator {
    None,
    CHP(QuantumCHPState),
    Graph(QuantumGraphState),
    Vector(QuantumVectorState),
}

pub trait RunConfig {
    fn init_state(&self) -> Simulator;
    fn gen_dataslide(&self, sim: Simulator) -> DataSlide;
}

pub struct ParallelCompute<C: RunConfig + std::marker::Sync> {
    num_threads: usize,
    configs: Vec<C>,
	params: HashMap<String, DataField>,

    initialized: bool,
}

impl<C: RunConfig + std::marker::Sync> ParallelCompute<C> {
    pub fn new(num_threads: usize, configs: Vec<C>) -> Self {
        Self { num_threads: num_threads, configs: configs, params: HashMap::new(), initialized: false }
    }

	pub fn add_int_param(&mut self, key: &str, val: i32) {
		self.params.insert(String::from(key), DataField::Int(val));
	}

	pub fn add_float_param(&mut self, key: &str, val: f32) {
		self.params.insert(String::from(key), DataField::Float(val));
	}

    pub fn compute(&self) -> DataFrame {
        if !self.initialized {
            rayon::ThreadPoolBuilder::new().num_threads(self.num_threads).build_global().unwrap();
        }

        let slides: Vec<DataSlide> = (0..self.configs.len()).into_par_iter().map(|i| {
            let mut state: Simulator = self.configs[i].init_state();
            self.configs[i].gen_dataslide(state)
        }).collect();

		return DataFrame { params: self.params.clone(), slides: slides };
    }
}