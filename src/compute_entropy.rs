use crate::quantum_chp_state::QuantumCHPState;
use crate::quantum_state::{QuantumState, Entropy};
use crate::dataframe::{DataFrame, DataSlide};

use serde::{Serialize, Deserialize};
use rand::rngs::ThreadRng;
use rayon::prelude::*;
use rand::Rng;

#[derive(Serialize, Deserialize)]
pub struct EntropyConfig {
    system_size: usize,
    partition_sizes: Vec<usize>,
    mzr_probs: Vec<f32>,
    timesteps: usize,
    measurement_freq: usize,
    filename: String
}

impl EntropyConfig {
    pub fn load_json(cfg_filename: &String) -> Self {
        let data = std::fs::read_to_string(cfg_filename).unwrap();
        let cfg: EntropyConfig = serde_json::from_str(&data).unwrap();
        return cfg;
    }

    pub fn print(&self) {
        println!("config: \nsystem_size: {}, partition_sizes:: {:?}, mzr_probs: {:?}, timesteps: {}, measurement_freq: {}, filename: {}",
                  self.system_size, self.partition_sizes, self.mzr_probs, self.timesteps, self.measurement_freq, self.filename);
    }
}


fn apply_layer<Q: QuantumState>(quantum_state: &mut Q, rng: &mut ThreadRng, offset: bool) {
    let system_size = quantum_state.system_size();
    for i in 0..system_size/2 {
        let mut qubit1 = if offset { (2*i + 1) % system_size } else { 2*i };
        let mut qubit2 = if offset { (2*i + 2) % system_size } else { (2*i + 1) % system_size };
        if rng.gen::<u8>() % 2 == 0 {
            std::mem::swap(&mut qubit1, &mut qubit2);
        }

        if rng.gen::<u8>() % 2 == 0 {
            quantum_state.cz_gate(qubit1, qubit2);
        } else {
            quantum_state.cx_gate(qubit1, qubit2);
        }
    }
}

pub fn compute_entropy<Q: QuantumState + Entropy>(system_size: usize, subsystem_size: usize, mzr_prob: f32, 
                                                  timesteps: usize, measurement_freq: usize) -> Vec<f32> {
    assert!(0. < mzr_prob && mzr_prob < 1.);
    let mut quantum_state: Q = Q::new(system_size);
    let mut rng = rand::thread_rng();
    let qubits: Vec<usize> = (0..subsystem_size).collect();
    let mut entropy: Vec<f32> = Vec::new();

    for t in 0..timesteps {

        apply_layer(&mut quantum_state, &mut rng, false);
        apply_layer(&mut quantum_state, &mut rng, false);

        apply_layer(&mut quantum_state, &mut rng, true);
        apply_layer(&mut quantum_state, &mut rng, true);

        for i in 0..system_size {
            if rng.gen::<f32>() < mzr_prob {
                quantum_state.mzr_qubit(i);
                quantum_state.h_gate(i);
            }
        }
        if t % measurement_freq == 0 {
            entropy.push(quantum_state.renyi_entropy(&qubits));
        }

    }

    return entropy;
}


fn gen_dataslide<Q: QuantumState + Entropy>(system_size: usize, partition_size: usize, prob: f32, timesteps: usize, measurement_freq: usize) -> DataSlide {
	let mut df: DataSlide = DataSlide::new();
	df.add_int_param("L", system_size as i32);
	df.add_int_param("LA", partition_size as i32);
	df.add_float_param("p", prob);
	df.add_data("entropy");

    let entropy = compute_entropy::<Q>(system_size, partition_size, prob, timesteps, measurement_freq);
    for s in entropy {
        df.push_data("entropy", s);
    }

	return df;
}



pub fn take_data<Q: QuantumState + Entropy>(cfg_filename: &String) {
    
    let config = EntropyConfig::load_json(cfg_filename);
    config.print();
    let num_sizes: usize = config.partition_sizes.len();
    let num_probs: usize = config.mzr_probs.len();

    let mut params: Vec<(f32, usize)> = Vec::new();
    for i in 0..num_probs {
        for j in 0..num_sizes {
            params.push((config.mzr_probs[i], config.partition_sizes[j]));
        }
    }

    let slides: Vec<DataSlide> = params.into_par_iter().map(|x| {
                                        gen_dataslide::<Q>(config.system_size, x.1, x.0, config.timesteps, config.measurement_freq)
                                   }).collect();

    let data: DataFrame = DataFrame::from(slides);
    println!("done!");
    data.save_json(config.filename);
}

