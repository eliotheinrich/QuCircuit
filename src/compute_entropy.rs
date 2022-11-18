use std::collections::HashMap;

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

enum Simulator {
    CHP(QuantumCHPState),
    Graph(QuantumGraphState),
    Vector(QuantumVectorState),
}

enum Param {
    Int(i32),
    Float(f32),
}

impl Param {
    pub fn unwrap_int(&self) -> i32 {
        match self {
            Param::Int(x) => *x,
            _ => panic!(),
        }
    }

    pub fn unwrap_float(&self) -> f32 {
        match self {
            Param::Float(x) => *x,
            _ => panic!(),
        }
    }
}


struct ParamSet {
    pub state: Simulator,
    params: HashMap<String, Param>,
}

impl ParamSet {
    pub fn get_int(&self, key: &str) -> i32 {
        return self.params[key].unwrap_int();
    }
    pub fn get_float(&self, key: &str) -> f32 {
        return self.params[key].unwrap_float();
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

fn timestep<Q: QuantumState>(quantum_state: &mut Q, mzr_prob: f32) {
    let mut rng = rand::thread_rng();
    apply_layer(quantum_state, &mut rng, false, &Gate::CX);
    apply_layer(quantum_state, &mut rng, false, &Gate::CZ);

    apply_layer(quantum_state, &mut rng, true, &Gate::CX);
    apply_layer(quantum_state, &mut rng, true, &Gate::CZ);

    for i in 0..quantum_state.system_size() {
        if rng.gen::<f32>() < mzr_prob {
            quantum_state.mzr_qubit(i);
            quantum_state.h_gate(i);
        }
    }
}

fn polarize<Q: QuantumState>(quantum_state: &mut Q) {
    for i in 0..quantum_state.system_size() {
        quantum_state.h_gate(i);
    }
}

fn evolve_quantum_state<Q: QuantumState>(quantum_state: &mut Q, mzr_prob: f32, timesteps: usize) {
    for t in 0..timesteps {
        timestep(quantum_state, mzr_prob);
    }
}

fn compute_entropy<Q: QuantumState + Entropy>(quantum_state: &mut Q, subsystem_size: usize, mzr_prob: f32, 
                                                  timesteps: usize, measurement_freq: usize) -> (&mut Q, Vec<f32>) {
    let system_size = quantum_state.system_size();
    let qubits: Vec<usize> = (0..subsystem_size).collect();
    let mut entropy: Vec<f32> = Vec::new();

    // Intially polarize in x-direction
    polarize(quantum_state);

    for t in 0..timesteps/measurement_freq {
        evolve_quantum_state(quantum_state, mzr_prob, measurement_freq);
        entropy.push(quantum_state.renyi_entropy(&qubits));
    }

    return (quantum_state, entropy);
}

fn save_to_dataslide<Q: QuantumState + Entropy>(dataslide: &mut DataSlide, mut quantum_state: Q, LA: usize, p: f32, timesteps: usize, 
                                                measurement_freq: usize, save_state: bool) {
    let (state, entropy) = compute_entropy::<Q>(&mut quantum_state, LA, p, timesteps, measurement_freq);
    for s in entropy {
        dataslide.push_data("entropy", s);
    }

    if save_state {
        dataslide.add_state("state", quantum_state);
    }
}

fn gen_dataslide(config: EntropyConfig, params: ParamSet) -> DataSlide {
	let mut dataslide: DataSlide = DataSlide::new();

    let system_size = config.system_size;
    let timesteps = config.timesteps;
    let measurement_freq = config.measurement_freq;
    let save_state = config.save_state;

    let LA: usize = params.get_int("LA") as usize;
    let p: f32 = params.get_float("p");


	dataslide.add_int_param("L", system_size as i32);
	dataslide.add_int_param("LA", LA as i32);
	dataslide.add_float_param("p", p);
	dataslide.add_data("entropy");

    match params.state {
        Simulator::CHP(state) => save_to_dataslide::<QuantumCHPState>(&mut dataslide, state, LA, p, timesteps, measurement_freq, save_state),
        Simulator::Graph(state) => save_to_dataslide::<QuantumGraphState>(&mut dataslide, state, LA, p, timesteps, measurement_freq, save_state),
        Simulator::Vector(state) => save_to_dataslide::<QuantumVectorState>(&mut dataslide, state, LA, p, timesteps, measurement_freq, save_state),
    }

	return dataslide;
}

fn get_params_from_file(data_filename: String) -> Vec<ParamSet> {
    let mut params: Vec<ParamSet> = Vec::new();
    let data_json: String = std::fs::read_to_string(data_filename).unwrap();
    let dataframe: DataFrame = serde_json::from_str(&data_json).unwrap();
    for slide in dataframe.slides {
        let mut param_data: HashMap<String, Param> = HashMap::new();
        param_data.insert(String::from("p"), Param::Float(slide.unwrap_float("p")));
        param_data.insert(String::from("LA"), Param::Int(slide.unwrap_int("LA")));

        match slide.get_val("state") {
            DataField::QuantumCHPState(state) => params.push(ParamSet { state : Simulator::CHP(state.clone()), params : param_data } ),
            DataField::QuantumGraphState(state) => params.push(ParamSet { state : Simulator::Graph(state.clone()), params : param_data } ),
            DataField::QuantumVectorState(state) => params.push(ParamSet { state : Simulator::Vector(state.clone()), params : param_data } ),
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
            let mut param_data: HashMap<String, Param> = HashMap::new();
            param_data.insert(String::from("p"), Param::Float(config.mzr_probs[i]));
            param_data.insert(String::from("LA"), Param::Int(config.partition_sizes[j] as i32));

            match config.simulator_type {
                0 => params.push(ParamSet { state : Simulator::CHP(QuantumCHPState::new(config.system_size)), params : param_data } ),
                1 => params.push(ParamSet { state : Simulator::Graph(QuantumGraphState::new(config.system_size)), params : param_data } ),
                2 => params.push(ParamSet { state : Simulator::Vector(QuantumVectorState::new(config.system_size)), params : param_data } ),
                _ => {
                    println!("Simulator type not supported; must be: \n0: CHP simulator\n1: Graph state simulator\n2: Vector simulator");
                    panic!();
                }
            }
        }
    }

    return params;
}

fn parallel_compute(config: &EntropyConfig, params: Vec<ParamSet>) -> Vec<DataSlide> {
    return params.into_par_iter().map(|param| {
        gen_dataslide(config.clone(), param)
    }).collect();
}



pub fn time_series() {
    let L: usize = 800;
    let LA: usize = L/2;
    let mzr_prob = 0.138;
    let num_runs: usize = 10;

    let times: Vec<i32> = (5..1000).step_by(20).collect();

    let mut slides: Vec<DataSlide> = times.into_par_iter().map(|x| {
        println!("time = {x}");
        let mut ds: DataSlide = DataSlide::new();
        ds.add_int_param("t", x);
        ds.add_int_param("L", L as i32);
        ds.add_int_param("LA", LA as i32);
        ds.add_float_param("p", mzr_prob);
        let mut S: f32 = 0.;
        let qubits: Vec<usize> = (0..LA).collect();

        for i in 0..num_runs {
            let mut quantum_state: QuantumCHPState = QuantumCHPState::new(L);
            polarize(&mut quantum_state);
            evolve_quantum_state(&mut quantum_state, mzr_prob, x as usize);
            S += quantum_state.renyi_entropy(&qubits);
        }
        S /= num_runs as f32;

        ds.add_float_param("entropy", S);
        return ds;
    }).collect();

    let dataframe: DataFrame = DataFrame::from(slides);
    dataframe.save_json(String::from("data/timeseries.json"));
    

}


pub fn take_data(cfg_filename: &String) {
    let cfg_path: String = String::from("configs/") + cfg_filename;
    let config: EntropyConfig = EntropyConfig::load_json(&cfg_path);
    config.print();

    let data_filename: String = String::from("data/") + &config.filename;

    let params: Vec<ParamSet> = if config.load_state { 
        get_params_from_file(data_filename.clone()) 
    } else { 
        get_params_from_cfg(config.clone()) 
    };

    let mut slides: Vec<DataSlide> = parallel_compute(&config, params);

    let dataframe: DataFrame = DataFrame::from(slides);
    if config.save_data {
        dataframe.save_json(data_filename);
    }
}

