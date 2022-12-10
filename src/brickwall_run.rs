use std::collections::HashMap;
use std::os::unix::fs::symlink;

use crate::quantum_chp_state::QuantumCHPState;
use crate::quantum_graph_state::QuantumGraphState;
use crate::quantum_vector_state::QuantumVectorState;
use crate::quantum_state::{QuantumState, Entropy};
use crate::dataframe::{Sample, DataFrame, DataSlide, Simulator, RunConfig, ParallelCompute};

use serde::{Serialize, Deserialize};
use rand::rngs::ThreadRng;
use rayon::prelude::*;
use rand::Rng;

const fn _true() -> bool { true }
const fn _false() -> bool { false }
const fn _zero() -> usize { 0 }
const fn _one() -> usize { 1 }
const fn _two() -> usize { 2 }

#[derive(Debug, Serialize, Deserialize, Clone)]
struct EntropyJSONConfig {
    run_name: String,

    circuit_type: String,

    #[serde(default = "_two")]
    gate_width: usize,

    simulator_type: String,

    system_sizes: Vec<usize>,
    partition_sizes: Vec<usize>,
    mzr_probs: Vec<f32>,
    timesteps: Vec<usize>,

    #[serde(default = "_one")]
    num_runs: usize,

    #[serde(default = "_zero")]
    equilibration_steps: usize,

    #[serde(default = "_false")]
    temporal_avg: bool,
    #[serde(default = "_zero")]
    measurement_freq: usize,

    #[serde(default = "_false")]
    space_avg: bool,
    #[serde(default = "_one")]
    spacing: usize,

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

enum CircuitType {
    Default,
    RandomClifford,
}

struct EntropyConfig {
    circuit_type: CircuitType,
    gate_width: usize,
    simulator_type: String,

    system_size: usize,
    partition_size: usize,
    mzr_prob: f32,
    timesteps: usize,

    num_runs: usize,

    equilibration_steps: usize,

    temporal_avg: bool,
    measurement_freq: usize,
    
    space_avg: bool,
    spacing: usize,
}

enum Gate {
    CZ,
    CX,
}

fn polarize<Q: QuantumState>(quantum_state: &mut Q) {
    for i in 0..quantum_state.system_size() {
        quantum_state.h_gate(i);
    }
}


fn apply_default_layer<Q: QuantumState>(quantum_state: &mut Q, rng: &mut ThreadRng, offset: bool, gate_type: &Gate) {
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

fn timesteps_default<Q: QuantumState>(quantum_state: &mut Q, timesteps: usize, mzr_prob: f32) {
    let mut rng: ThreadRng = rand::thread_rng();
    for i in 0..timesteps {
        apply_default_layer(quantum_state, &mut rng, false, &Gate::CX);
        apply_default_layer(quantum_state, &mut rng, false, &Gate::CZ);

        apply_default_layer(quantum_state, &mut rng, true, &Gate::CX);
        apply_default_layer(quantum_state, &mut rng, true, &Gate::CZ);

        for i in 0..quantum_state.system_size() {
            if rng.gen::<f32>() < mzr_prob {
                quantum_state.mzr_qubit(i);
                quantum_state.h_gate(i);
            }
        }
    }
}

fn timesteps_random_clifford<Q: QuantumState>(quantum_state: &mut Q, timesteps: usize, mzr_prob: f32, init_offset: bool) {
    let system_size = quantum_state.system_size();
    //assert!(system_size % gate_width == 0);
    //assert!(gate_width % 2 == 0);
    //let offset: usize = gate_width / 2;
    //let num_gates: usize = system_size / gate_width;

    let mut rng: ThreadRng = rand::thread_rng();

    let mut offset_layer: bool = init_offset;

    for t in 0..timesteps {

        if offset_layer {
            for i in 0..system_size/2 {
                quantum_state.random_clifford([2*i, (2*i+1) % system_size]);
            }
        } else {
            for i in 0..system_size/2 {
                quantum_state.random_clifford([(2*i+1) % system_size, (2*i+2) % system_size]);
            }
        }

        offset_layer = !offset_layer;

        for i in 0..system_size {
            if rng.gen::<f32>() < mzr_prob {
                quantum_state.mzr_qubit(i);
            }
        }
    }
}


impl EntropyConfig {
    pub fn from(json_config: &EntropyJSONConfig, system_size_idx: usize, timesteps_idx: usize, 
                                                 partition_size_idx: usize, mzr_idx: usize) -> Self {
        return EntropyConfig{
            circuit_type: match json_config.circuit_type.as_str() {
                "default" => CircuitType::Default,
                "random_clifford" => CircuitType::RandomClifford,
                _ => {
                    println!("circuit type {} not supported.", json_config.circuit_type);
                    panic!();
                }
            },
            gate_width: json_config.gate_width,
            simulator_type: json_config.simulator_type.clone(),

            system_size: json_config.system_sizes[system_size_idx],
            partition_size: json_config.partition_sizes[partition_size_idx],
            mzr_prob: json_config.mzr_probs[mzr_idx],
            timesteps: json_config.timesteps[timesteps_idx],

            num_runs: json_config.num_runs,

            equilibration_steps: json_config.equilibration_steps,

            temporal_avg: json_config.temporal_avg,
            measurement_freq: json_config.measurement_freq,

            space_avg: json_config.space_avg,
            spacing: json_config.spacing,
        }
    }

    fn compute_entropy<Q: QuantumState + Entropy>(&self, quantum_state: &mut Q) -> Vec<Sample> {
        let system_size = quantum_state.system_size();
        let qubits: Vec<usize> = (0..self.partition_size).collect();
        let mut entropy: Vec<Sample> = Vec::new();

        // Intially polarize in x-direction
        match self.circuit_type {
            CircuitType::Default => {
                polarize(quantum_state);
                timesteps_default(quantum_state, self.equilibration_steps, self.mzr_prob);
            },
            CircuitType::RandomClifford => {
                timesteps_random_clifford(quantum_state, self.equilibration_steps, self.mzr_prob, false);
            },
        }


        // Do timesteps
        let (num_timesteps, num_intervals): (usize, usize) = if self.timesteps == 0 { 
            (0, 1)
        } else { 
            (self.measurement_freq, self.timesteps/self.measurement_freq)
        };
        
        for t in 0..num_intervals {
            match self.circuit_type {
                CircuitType::Default => timesteps_default(quantum_state, num_timesteps, self.mzr_prob),
                CircuitType::RandomClifford => timesteps_random_clifford(quantum_state, num_timesteps, self.mzr_prob, t*num_timesteps % 2 == 0),
            }

            let sample: Sample = 
            if self.space_avg {
                let num_partitions = (self.system_size - self.partition_size)/self.spacing;

                let mut s: f32 = 0.;
                let mut s2: f32 = 0.;

                for i in 0..num_partitions {
                    let offset_qubits: Vec<usize> = qubits.iter().map(|x| x + i*self.spacing).collect();
                    let tmp: f32 = quantum_state.renyi_entropy(&offset_qubits);
                    s += tmp;
                    s2 += tmp.powi(2);
                }

                s /= num_partitions as f32;
                s2 /= num_partitions as f32;
                let std: f32 = (s2 - s.powi(2)).powf(0.5);
                Sample { mean: s, std: std, num_samples: num_partitions }
            } else {
                Sample::new(quantum_state.renyi_entropy(&qubits))
            };

            entropy.push(sample);
        }

        return entropy;
    }

    fn save_to_dataslide<Q: QuantumState + Entropy + Clone>(&self, dataslide: &mut DataSlide, quantum_state: &mut Q) {
        let init_quantum_state = quantum_state.clone();

        let mut entropy: Vec<Sample> = self.compute_entropy(quantum_state);

        for run in 0..(self.num_runs - 1) {
            *quantum_state = init_quantum_state.clone(); // Reinitialize state
            entropy = entropy.iter()
                             .zip(self.compute_entropy(quantum_state).iter())
                             .map(|(a, b)| a.combine(b))
                             .collect();
        }

        if self.temporal_avg {
            let sample: Sample = entropy.iter().fold(Sample { mean: 0., std: 0., num_samples: 0 }, |sum, val| sum.combine(val));
            entropy = vec![sample];
        }

        for s in entropy {
            dataslide.push_data("entropy", s);
        }
    }

}

impl RunConfig for EntropyConfig {
    fn init_state(&self) -> Simulator {
        return match self.simulator_type.as_str() {
            "chp" => Simulator::CHP(QuantumCHPState::new(self.system_size)),
            "graph" => Simulator::Graph(QuantumGraphState::new(self.system_size)),
            "vector" => Simulator::Vector(QuantumVectorState::new(self.system_size)),
            _ => {
                println!("Error: simulator type provided not supported.");
                panic!()
            }
        }       
    }

    fn gen_dataslide(&self, mut simulator: Simulator) -> DataSlide {
        assert!(self.num_runs > 0);
        let mut dataslide: DataSlide = DataSlide::new();

        // Parameters
        dataslide.add_int_param("system_size", self.system_size as i32);
        dataslide.add_int_param("timesteps", self.timesteps as i32);
        dataslide.add_int_param("partition_size", self.partition_size as i32);
        dataslide.add_float_param("mzr_prob", self.mzr_prob);

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

    println!("Producing {} configs", configs.len());
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
    let cfg_path: String = cfg_filename.to_string();
    let json_config: EntropyJSONConfig = EntropyJSONConfig::load_json(&cfg_path);
    json_config.print();


    let configs: Vec<EntropyConfig> = load_json_config(&json_config);

    let mut pc = ParallelCompute::new(num_threads, configs);
    pc.add_int_param("equilibration_steps", json_config.equilibration_steps as i32);
    pc.add_int_param("measurement_freq", json_config.measurement_freq as i32);
    let dataframe = pc.compute();
    
    if json_config.save_data {
        let data_filename: String = String::from("data/") + &json_config.filename;
        dataframe.save_json(data_filename);
    }
}

