use std::collections::HashMap;

use crate::quantum_chp_state::QuantumCHPState;
use crate::quantum_graph_state::QuantumGraphState;
use crate::quantum_vector_state::QuantumVectorState;
use crate::quantum_state::{QuantumState, Entropy};
use crate::dataframe::{DataFrame, DataSlide, Simulator, RunConfig, ParallelCompute};

use serde::{Serialize, Deserialize};
use rand::rngs::ThreadRng;
use rayon::prelude::*;
use rand::Rng;

const fn _true() -> bool { true }
const fn _false() -> bool { false }
const fn _zero() -> usize { 0 }
const fn _one() -> usize { 1 }

#[derive(Debug, Serialize, Deserialize, Clone)]
struct EntropyJSONConfig {
    simulator_type: u8,

    system_sizes: Vec<usize>,
    partition_sizes: Vec<usize>,
    mzr_probs: Vec<f32>,
    timesteps: Vec<usize>,

    #[serde(default = "_zero")]
    measurement_freq: usize,

    #[serde(default = "_one")]
    num_runs: usize,

    #[serde(default = "_false")]
    space_avg: bool,
    #[serde(default = "_one")]
    spacing: usize,

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


struct EntropyConfig {
    simulator_type: u8,

    system_size: usize,
    partition_size: usize,
    mzr_prob: f32,
    timesteps: usize,

    measurement_freq: usize,
    num_runs: usize,
    
    space_avg: bool,
    spacing: usize,
    save_state: bool,
    load_state: bool,
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




impl EntropyConfig {
    pub fn from(json_config: &EntropyJSONConfig, system_size_idx: usize, timesteps_idx: usize, 
                                                 partition_size_idx: usize, mzr_idx: usize) -> Self {
        return EntropyConfig{
            simulator_type: json_config.simulator_type,
            system_size: json_config.system_sizes[system_size_idx],
            partition_size: json_config.partition_sizes[partition_size_idx],
            mzr_prob: json_config.mzr_probs[mzr_idx],
            timesteps: json_config.timesteps[timesteps_idx],

            measurement_freq: json_config.measurement_freq,
            num_runs: json_config.num_runs,

            space_avg: json_config.space_avg,
            spacing: json_config.spacing,
            save_state: json_config.save_state,
            load_state: json_config.load_state,
        }
    }

    fn compute_entropy<Q: QuantumState + Entropy>(&self, quantum_state: &mut Q) -> Vec<f32> {
        let system_size = quantum_state.system_size();
        let qubits: Vec<usize> = (0..self.partition_size).collect();
        let mut entropy: Vec<f32> = Vec::new();

        // Intially polarize in x-direction
        polarize(quantum_state);

        // Do timesteps
        let (num_timesteps, num_intervals): (usize, usize) = if self.measurement_freq == 0 { 
            (self.timesteps, 1) 
        } else { 
            (self.measurement_freq, self.timesteps/self.measurement_freq)
        };
        
        for t in 0..num_intervals {
            do_timesteps(quantum_state, num_timesteps, self.mzr_prob);

            let s: f32 = 
            if self.space_avg {
                let mut tmp: f32 = 0.;
                let num_partitions = (self.system_size - self.partition_size)/self.spacing;

                for i in (0..num_partitions).step_by(self.spacing) {
                    let offset_qubits: Vec<usize> = qubits.iter().map(|x| x + i).collect();
                    tmp += quantum_state.renyi_entropy(&offset_qubits);
                }
                tmp/(num_partitions as f32)
            } else {
                quantum_state.renyi_entropy(&qubits)
            };

            entropy.push(s);
        }

        return entropy;
    }

    fn save_to_dataslide<Q: QuantumState + Entropy + Clone>(&self, dataslide: &mut DataSlide, quantum_state: &mut Q) {
        let num_datapoints = if self.measurement_freq == 0 { 1 } else { self.timesteps / self.measurement_freq };
        let mut entropy: Vec<f32> = vec![0.; num_datapoints];
        let mut state: &Q;

        let mut entropy_tmp: Vec<f32> = vec![0.; num_datapoints];
        let init_quantum_state = quantum_state.clone();
        for run in 0..self.num_runs {
            if run > 0 { *quantum_state = init_quantum_state.clone() } // If doing multiple runs, reset each time
            entropy_tmp = self.compute_entropy::<Q>(quantum_state);
            entropy = entropy.iter().zip(entropy_tmp.iter()).map(|(a, b)| a + b).collect();
        }


        for s in entropy {
            dataslide.push_data("entropy", s/(self.num_runs as f32));
        }

        if self.save_state {
            dataslide.add_state("state", quantum_state);
        }
    }

}

impl RunConfig for EntropyConfig {
    fn init_state(&self) -> Simulator {
        if self.load_state {
            // TODO implement loading states
            return Simulator::CHP(QuantumCHPState::new(self.system_size))
        } else {
            return match self.simulator_type {
                0 => Simulator::CHP(QuantumCHPState::new(self.system_size)),
                1 => Simulator::Graph(QuantumGraphState::new(self.system_size)),
                2 => Simulator::Vector(QuantumVectorState::new(self.system_size)),
                _ => {
                    println!("Error: simulator type provided not supported.");
                    panic!()
                }
            }       
        }
    }

    fn gen_dataslide(&self, mut simulator: Simulator) -> DataSlide {
        assert!(self.num_runs > 0);
        let mut dataslide: DataSlide = DataSlide::new();

        let system_size = self.system_size;
        let timesteps = self.timesteps;
        let measurement_freq = self.measurement_freq;
        let save_state = self.save_state;

        let partition_size: usize = self.partition_size;
        let mzr_prob: f32 = self.mzr_prob;

        dataslide.add_int_param("system_size", system_size as i32);
        dataslide.add_int_param("timesteps", timesteps as i32);
        dataslide.add_int_param("partition_size", partition_size as i32);
        dataslide.add_float_param("mzr_prob", mzr_prob);
        dataslide.add_data("entropy");
        
        match simulator {
            Simulator::CHP(mut state) => self.save_to_dataslide(&mut dataslide, &mut state),
            Simulator::Graph(mut state) => self.save_to_dataslide(&mut dataslide, &mut state),
            Simulator::Vector(mut state) => self.save_to_dataslide(&mut dataslide, &mut state),
            Simulator::None => {
                println!("State not initialized!");
                panic!();
            }
        };

        return dataslide;
            
    }
}


fn load_json_config(json_config: &EntropyJSONConfig) -> Vec<EntropyConfig> {
    let mut configs: Vec<EntropyConfig> = Vec::new();
    for system_size_idx in 0..json_config.system_sizes.len() {
        for timesteps_idx in 0..json_config.timesteps.len() {
            for partition_size_idx in 0..json_config.partition_sizes.len() {
                for mzr_prob_idx in 0..json_config.mzr_probs.len() {
                    configs.push(EntropyConfig::from(&json_config, system_size_idx, 
                                                                   timesteps_idx, 
                                                                   partition_size_idx, 
                                                                   mzr_prob_idx));
                }
            }
        }
    }

    return configs;
}

/*
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
*/

pub fn take_data(num_threads: usize, cfg_filename: &String) {
    let cfg_path: String = String::from("configs/") + cfg_filename;
    let json_config: EntropyJSONConfig = EntropyJSONConfig::load_json(&cfg_path);
    json_config.print();

    let data_filename: String = String::from("data/") + &json_config.filename;

    let configs: Vec<EntropyConfig> = load_json_config(&json_config);

    let mut pc: ParallelCompute<EntropyConfig> = ParallelCompute::new(num_threads, configs);
    let dataframe = pc.compute();

    if json_config.save_data {
        dataframe.save_json(data_filename);
    }
}

