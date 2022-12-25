use std::collections::HashMap;
use std::fs;

use crate::quantum_chp_state::QuantumCHPState;
use crate::quantum_graph_state::QuantumGraphState;
use crate::quantum_vector_state::QuantumVectorState;
use crate::quantum_state::{QuantumState, Entropy};
use crate::dataframe::{Sample, DataFrame, DataSlide, Simulator, RunConfig, ParallelCompute};

use serde::{Serialize, Deserialize};
use rand::rngs::ThreadRng;
use rayon::prelude::*;
use rand::Rng;

pub enum CircuitType {
    QuantumAutomaton,
    RandomClifford,
}

enum Gate {
    CZ,
    CX,
}

// Initial polarization applied to QA circuits
pub fn polarize<Q: QuantumState>(quantum_state: &mut Q) {
    for i in 0..quantum_state.system_size() {
        quantum_state.h_gate(i);
    }
}

// Apply a quantum automaton layer
fn apply_qa_layer<Q: QuantumState>(quantum_state: &mut Q, rng: &mut ThreadRng, offset: bool, gate_type: &Gate) {
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

// Apply some timesteps to the quantum automaton circuit
pub fn timesteps_qa<Q: QuantumState>(quantum_state: &mut Q, timesteps: usize, mzr_prob: f32) {
    let mut rng: ThreadRng = rand::thread_rng();
    for i in 0..timesteps {
        apply_qa_layer(quantum_state, &mut rng, false, &Gate::CX);
        apply_qa_layer(quantum_state, &mut rng, false, &Gate::CZ);

        apply_qa_layer(quantum_state, &mut rng, true, &Gate::CX);
        apply_qa_layer(quantum_state, &mut rng, true, &Gate::CZ);

        for i in 0..quantum_state.system_size() {
            if rng.gen::<f32>() < mzr_prob {
                quantum_state.mzr_qubit(i);
                quantum_state.h_gate(i);
            }
        }
    }
}

// Apply some timesteps to the random clifford circuit
pub fn timesteps_rc<Q: QuantumState>(quantum_state: &mut Q, timesteps: usize, mzr_prob: f32, gate_width: usize, init_offset: bool) {
    let system_size = quantum_state.system_size();

    // System size must be divisible by gate width
    assert!(system_size % gate_width == 0);

    // For now, gate width must be divisible by 2 for offset to function correctly
    assert!(gate_width % 2 == 0);

    let offset: usize = gate_width / 2;
    let num_gates: usize = system_size / gate_width;

    let mut rng: ThreadRng = rand::thread_rng();

    let mut offset_layer: bool = init_offset;

    for t in 0..timesteps {

        let qubits: Vec<usize> = (0..gate_width).map(|i| i % system_size).collect();

        for i in 0..num_gates {
            let offset_qubits: Vec<usize> = 
            if offset_layer {
                qubits.iter().map(|j| (j + gate_width*i) % system_size).collect()
            } else {
                qubits.iter().map(|j| (j + gate_width*i + gate_width/2) % system_size).collect()
            };
            
            quantum_state.random_clifford(offset_qubits);
        }

        offset_layer = !offset_layer;

        for i in 0..system_size {
            if rng.gen::<f32>() < mzr_prob {
                quantum_state.mzr_qubit(i);
            }
        }
    }
}

use crate::entropy_config::{EntropyJSONConfig, EntropyConfig};

pub fn take_data(num_threads: usize, cfg_filename: &String) {
    let cfg_path: String = cfg_filename.to_string();
    let json_config: EntropyJSONConfig = EntropyJSONConfig::load_json(&cfg_path);
    json_config.print();


    let configs: Vec<EntropyConfig> = json_config.to_configs();

    let mut pc: ParallelCompute<EntropyConfig> = ParallelCompute::new(num_threads, configs);
    pc.add_int_param("equilibration_steps", json_config.equilibration_steps as i32);
    pc.add_int_param("measurement_freq", json_config.measurement_freq as i32);
    let dataframe: DataFrame = pc.compute();
    
    if json_config.save_data {
        let data_filename: String = String::from("data/") + &json_config.filename;
        fs::remove_file(&data_filename);
        dataframe.save_json(data_filename);
    }
}

use crate::state_config::{StateJSONConfig, StateConfig};

pub fn generate_states(num_threads: usize, cfg_filename: &String) {
    let cfg_path: String = cfg_filename.to_string();
    let json_config: StateJSONConfig = StateJSONConfig::load_json(&cfg_path);
    json_config.print();


    let configs: Vec<StateConfig> = json_config.to_configs();

    let mut pc: ParallelCompute<StateConfig> = ParallelCompute::new(num_threads, configs);
    pc.add_int_param("equilibration_steps", json_config.equilibration_steps as i32);
    pc.add_int_param("system_size", json_config.system_size as i32);
    pc.add_int_param("num_states", json_config.num_runs);
    let dataframe: DataFrame = pc.compute();
    
    let data_filename: String = String::from("data/") + &json_config.filename;
    fs::remove_file(&data_filename);
    dataframe.save_json(data_filename);
}