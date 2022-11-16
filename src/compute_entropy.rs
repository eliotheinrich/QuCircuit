use crate::quantum_chp_state::QuantumCHPState;
use crate::quantum_graph_state::QuantumGraphState;
use crate::quantum_vector_state::QuantumVectorState;
use crate::quantum_state::{QuantumState, Entropy};
use crate::dataframe::{DataFrame, DataSlide, DataField};

use serde::{Serialize, Deserialize};
use rand::rngs::ThreadRng;
use rayon::prelude::*;
use rand::Rng;

const fn _true() -> bool { true }
const fn _false() -> bool { false }

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EntropyConfig {
    simulator_type: u8,
    system_size: usize,
    partition_sizes: Vec<usize>,
    mzr_probs: Vec<f32>,
    timesteps: usize,
    measurement_freq: usize,

    #[serde(default = "_false")]
    save_state: bool,

    #[serde(default = "_false")]
    load_state: bool,

    #[serde(default = "_true")]
    save_data: bool, 

    filename: String
}

impl EntropyConfig {
    pub fn load_json(cfg_filename: &String) -> Self {
        let data = std::fs::read_to_string(cfg_filename).unwrap();
        let cfg: EntropyConfig = serde_json::from_str(&data).unwrap();
        return cfg;
    }

    pub fn print(&self) {
        println!("{:?}", self);
    }
}

enum ParamSet {
    CHP(QuantumCHPState, f32, usize),
    Graph(QuantumGraphState, f32, usize),
    Vector(QuantumVectorState, f32, usize)
}

impl ParamSet {
    pub fn get_p(&self) -> f32 {
        match self {
            Self::CHP(_, x, _) => *x,
            Self::Graph(_, x, _) => *x,
            Self::Vector(_, x, _) => *x,
        }
    }

    pub fn get_partition_size(&self) -> usize {
        match self {
            Self::CHP(_, _, x) => *x,
            Self::Graph(_, _, x) => *x,
            Self::Vector(_, _, x) => *x,
        }
    }
}

enum Gate {
    CZ,
    CX,
}


fn apply_layer<Q: QuantumState>(quantum_state: &mut Q, rng: &mut ThreadRng, offset: bool, gate_type: &Gate) {
    let system_size = quantum_state.system_size();
    for i in 0..system_size/2 {
        let mut qubit1 = if offset { (2*i + 1) % system_size } else { 2*i };
        let mut qubit2 = if offset { (2*i + 2) % system_size } else { (2*i + 1) % system_size };
        if rng.gen::<u8>() % 2 == 0 {
            std::mem::swap(&mut qubit1, &mut qubit2);
        }

        match gate_type {
            Gate::CZ => quantum_state.cz_gate(qubit1, qubit2),
            Gate::CX => quantum_state.cx_gate(qubit1, qubit2),
        };
    }
}

pub fn compute_entropy<Q: QuantumState + Entropy>(mut quantum_state: Q, subsystem_size: usize, mzr_prob: f32, 
                                                  timesteps: usize, measurement_freq: usize) -> (Q, Vec<f32>) {
    let system_size = quantum_state.system_size();
    let mut rng = rand::thread_rng();
    let qubits: Vec<usize> = (0..subsystem_size).collect();
    let mut entropy: Vec<f32> = Vec::new();

    for t in 0..timesteps {

        apply_layer(&mut quantum_state, &mut rng, false, &Gate::CX);
        apply_layer(&mut quantum_state, &mut rng, false, &Gate::CZ);

        apply_layer(&mut quantum_state, &mut rng, true, &Gate::CX);
        apply_layer(&mut quantum_state, &mut rng, true, &Gate::CZ);

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

    return (quantum_state, entropy);
}



fn gen_dataslide(config: EntropyConfig, param: ParamSet) -> DataSlide {
	let mut dataslide: DataSlide = DataSlide::new();

    let system_size = config.system_size;
    let timesteps = config.timesteps;
    let measurement_freq = config.measurement_freq;
    let save_state = config.save_state;

	dataslide.add_int_param("L", system_size as i32);
	dataslide.add_int_param("LA", param.get_partition_size() as i32);
	dataslide.add_float_param("p", param.get_p());
	dataslide.add_data("entropy");

    match param {
        ParamSet::CHP(state, p, LA) => {
            let (state, entropy) = compute_entropy::<QuantumCHPState>(state, LA, p, timesteps, measurement_freq);
            for s in entropy {
                dataslide.push_data("entropy", s);
            }
            if save_state {
                dataslide.add_state("state", state);
            }
        } ParamSet::Graph(state, p, LA) => {
            let (state, entropy) = compute_entropy::<QuantumGraphState>(state, LA, p, timesteps, measurement_freq);
            for s in entropy {
                dataslide.push_data("entropy", s);
            }
            if save_state {
                dataslide.add_state("state", state);
            }

        } ParamSet::Vector(state, p, LA) => {
            let (state, entropy) = compute_entropy::<QuantumVectorState>(state, LA, p, timesteps, measurement_freq);
            for s in entropy {
                dataslide.push_data("entropy", s);
            }
            if save_state {
                dataslide.add_state("state", state);
            }
        }
    }

	return dataslide;
}

fn get_params_from_file(data_filename: String) -> Vec<ParamSet> {
    let mut params: Vec<ParamSet> = Vec::new();
    let data_json: String = std::fs::read_to_string(data_filename).unwrap();
    let dataframe: DataFrame = serde_json::from_str(&data_json).unwrap();
    for slide in dataframe.slides {
        match slide.get_val("state") {
            DataField::QuantumCHPState(state) => params.push(ParamSet::CHP(state.clone(), slide.unwrap_float("p"), slide.unwrap_int("LA") as usize)),
            DataField::QuantumGraphState(state) => params.push(ParamSet::Graph(state.clone(), slide.unwrap_float("p"), slide.unwrap_int("LA") as usize)),
            DataField::QuantumVectorState(state) => params.push(ParamSet::Vector(state.clone(), slide.unwrap_float("p"), slide.unwrap_int("LA") as usize)),
            _ => panic!()
        }
    }

    return params;
}

fn get_params_from_cfg(config: EntropyConfig) -> Vec<ParamSet> {
    let num_sizes: usize = config.partition_sizes.len();
    let num_probs: usize = config.mzr_probs.len();

    let mut params: Vec<ParamSet> = Vec::new();
    for i in 0..config.mzr_probs.len() {
        for j in 0..config.partition_sizes.len() {
            match config.simulator_type {
                0 => params.push(ParamSet::CHP(QuantumCHPState::new(config.system_size), config.mzr_probs[i], config.partition_sizes[j])),
                1 => params.push(ParamSet::Graph(QuantumGraphState::new(config.system_size), config.mzr_probs[i], config.partition_sizes[j])),
                2 => params.push(ParamSet::Vector(QuantumVectorState::new(config.system_size), config.mzr_probs[i], config.partition_sizes[j])),
                _ => {
                    println!("Simulator type not supported; must be: \n0: CHP simulator\n1: Graph state simulator\n2: Vector simulator");
                    panic!();
                }
            }
        }
    }

    return params;
}

pub fn take_data(cfg_filename: &String) {
    let cfg_path: String = String::from("configs/") + cfg_filename;
    let config: EntropyConfig = EntropyConfig::load_json(&cfg_path);
    config.print();

    let data_filename: String = String::from("data/") + &config.filename;

    let params: Vec<ParamSet> = if config.load_state { get_params_from_file(data_filename.clone()) } else { get_params_from_cfg(config.clone()) };

    let mut slides: Vec<DataSlide> = params.into_par_iter().map(|param| {
        gen_dataslide(config.clone(), param)
    }).collect();

    let dataframe: DataFrame = DataFrame::from(slides);
    if config.save_data {
        dataframe.save_json(data_filename);
    }
}

