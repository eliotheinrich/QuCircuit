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
struct EntropyJSONConfig {
    simulator_type: u8,
    system_size: usize,
    partition_sizes: Vec<usize>,
    mzr_probs: Vec<f32>,
    timesteps: usize,
    measurement_freq: usize,
    
    #[serde(default = "_false")]
    space_avg: bool,

    #[serde(default = "_false")]
    save_state: bool,

    #[serde(default = "_false")]
    load_state: bool,

    #[serde(default = "_true")]
    save_data: bool, 

    filename: String
}

impl EntropyJSONConfig {
    pub fn load_json(cfg_filename: &String) -> Self {
        let data = std::fs::read_to_string(cfg_filename).unwrap();
        let cfg: EntropyJSONConfig = serde_json::from_str(&data).unwrap();
        return cfg;
    }

    pub fn print(&self) {
        println!("{:?}", self);
    }
}

#[derive(Debug)]
struct EntropyConfig {
    simulator_type: u8,
    system_size: usize,
    partition_size: usize,
    mzr_prob: f32,
    timesteps: usize,
    measurement_freq: usize,
    
    space_avg: bool,
    save_state: bool,
    load_state: bool,
}

impl EntropyConfig {
    pub fn from(json_config: &EntropyJSONConfig, partition_idx: usize, mzr_idx: usize) -> Self {
        return EntropyConfig{ 
            simulator_type: json_config.simulator_type,
            system_size: json_config.system_size,
            partition_size: json_config.partition_sizes[partition_idx],
            mzr_prob: json_config.mzr_probs[mzr_idx],
            timesteps: json_config.timesteps,
            measurement_freq: json_config.measurement_freq,

            space_avg: json_config.space_avg,
            save_state: json_config.save_state,
            load_state: json_config.load_state,
        }
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

fn do_timesteps<Q: QuantumState>(quantum_state: &mut Q, timesteps: usize, mzr_prob: f32) {
    for t in 0..timesteps {
        timestep(quantum_state, mzr_prob);
    }
}


fn compute_entropy<'a, Q: QuantumState + Entropy>(quantum_state: &'a mut Q, config: &EntropyConfig) -> (&'a mut Q, Vec<f32>) {
    let system_size = quantum_state.system_size();
    let qubits: Vec<usize> = (0..config.partition_size).collect();
    let mut entropy: Vec<f32> = Vec::new();

    // Intially polarize in x-direction
    polarize(quantum_state);

    // Do timesteps
    for t in 0..config.timesteps/config.measurement_freq {
        do_timesteps(quantum_state, config.measurement_freq, config.mzr_prob);

        let s: f32 = 
        if config.space_avg {
            let mut tmp: f32 = 0.;
            let num_partitions = config.system_size - config.partition_size;

            for i in 0..num_partitions {
                let offset_qubits: Vec<usize> = qubits.iter().map(|x| x + i).collect();
                tmp += quantum_state.renyi_entropy(&offset_qubits);
            }
            tmp
        } else {
            quantum_state.renyi_entropy(&qubits)
        };

        entropy.push(s);
    }

    return (quantum_state, entropy);
}

fn save_to_dataslide<Q: QuantumState + Entropy>(dataslide: &mut DataSlide, quantum_state: &mut Q, config: &EntropyConfig) {
    let (state, entropy) = compute_entropy::<Q>(quantum_state, config);

    for s in entropy {
        dataslide.push_data("entropy", s);
    }

    if config.save_state {
        dataslide.add_state("state", quantum_state);
    }
}

fn gen_dataslide(simulator: &mut Simulator, config: &EntropyConfig) -> DataSlide {
	let mut dataslide: DataSlide = DataSlide::new();

    let system_size = config.system_size;
    let timesteps = config.timesteps;
    let measurement_freq = config.measurement_freq;
    let save_state = config.save_state;

    let partition_size: usize = config.partition_size;
    let mzr_prob: f32 = config.mzr_prob;

	dataslide.add_int_param("L", system_size as i32);
	dataslide.add_int_param("LA", partition_size as i32);
	dataslide.add_float_param("p", mzr_prob);
	dataslide.add_data("entropy");
    
    match simulator {
        Simulator::CHP(state) => save_to_dataslide(&mut dataslide, state, config),
        Simulator::Graph(state) => save_to_dataslide(&mut dataslide, state, config),
        Simulator::Vector(state) => save_to_dataslide(&mut dataslide, state, config),
    };

	return dataslide;
}

fn load_config(json_config: &EntropyJSONConfig) -> Vec<EntropyConfig> {
    let mut configs: Vec<EntropyConfig> = Vec::new();
    for i in 0..json_config.partition_sizes.len() {
        for j in 0..json_config.mzr_probs.len() {
            configs.push(EntropyConfig::from(&json_config, i, j));
        }
    }

    return configs;
}

fn parallel_compute(configs: Vec<EntropyConfig>, states: Vec<Simulator>) -> Vec<DataSlide> {
    return configs.into_iter().zip(states.into_iter()).collect::<Vec<(EntropyConfig, Simulator)>>().into_par_iter().map(|(config, mut state)| {
        gen_dataslide(&mut state, &config)
    }).collect();
}



pub fn time_series() {
    let L: usize = 800;
    let LA: usize = L/2;
    let mzr_prob = 0.138;
    let num_runs: usize = 100;

    let mut times: Vec<i32> = (5..1000).step_by(20).collect();
    times.push(1);
    times.push(2);
    times.push(3);
    times.push(4);
    times.push(6);
    times.push(7);
    times.push(8);

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
            //evolve_quantum_state(&mut quantum_state, );
            S += quantum_state.renyi_entropy(&qubits);
        }
        S /= num_runs as f32;

        ds.add_float_param("entropy", S);
        return ds;
    }).collect();

    let dataframe: DataFrame = DataFrame::from(slides);
    dataframe.save_json(String::from("data/timeseries.json"));
    

}

fn load_states(data_filename: String) -> Vec<Simulator> {
    let data_json: String = std::fs::read_to_string(data_filename).unwrap();
    let dataframe: DataFrame = serde_json::from_str(&data_json).unwrap();

    let mut states: Vec<Simulator> = Vec::new();
    for slide in dataframe.slides {
        match slide.get_val("state") {
            DataField::QuantumCHPState(state) => states.push(Simulator::CHP(state.clone())),
            DataField::QuantumGraphState(state) => states.push(Simulator::Graph(state.clone())),
            DataField::QuantumVectorState(state) => states.push(Simulator::Vector(state.clone())),
            _ => panic!()
        }
    }

    return states;
}

fn gen_new_states(configs: &Vec<EntropyConfig>) -> Vec<Simulator> {
    let mut states: Vec<Simulator> = Vec::new();
    for config in configs {
        let simulator: Simulator = 
            match config.simulator_type {
                0 => Simulator::CHP(QuantumCHPState::new(config.system_size)),
                1 => Simulator::Graph(QuantumGraphState::new(config.system_size)),
                2 => Simulator::Vector(QuantumVectorState::new(config.system_size)),
                _ => {
                    println!("Simulator type not supported!");
                    panic!();
                }
            };

        states.push(simulator);
    }

    return states;
}

pub fn take_data(cfg_filename: &String) {
    let cfg_path: String = String::from("configs/") + cfg_filename;
    let json_config: EntropyJSONConfig = EntropyJSONConfig::load_json(&cfg_path);
    json_config.print();

    let data_filename: String = String::from("data/") + &json_config.filename;

    let configs: Vec<EntropyConfig> = load_config(&json_config);

    let mut states: Vec<Simulator> = if json_config.load_state {
        load_states(json_config.filename)
    } else {
        gen_new_states(&configs)
    };

    let mut slides: Vec<DataSlide> = parallel_compute(configs, states);


    let dataframe: DataFrame = DataFrame::from(slides);
    if json_config.save_data {
        dataframe.save_json(data_filename);
    }
}

