use quantum_circuit::compute_entropy::take_data;
use quantum_circuit::quantum_chp_state::QuantumCHPState;
use quantum_circuit::quantum_graph_state::QuantumGraphState;
use std::time::Instant;

use quantum_circuit::quantum_vector_state::QuantumVectorState;
use quantum_circuit::quantum_state::{QuantumProgram, QuantumState, Entropy};
use rand::Rng;
use rayon::prelude::*;

fn compute_entropy_run(num_threads: usize, config_filename: &String) {
    rayon::ThreadPoolBuilder::new().num_threads(num_threads).build_global().unwrap();

    let args: Vec<String> = std::env::args().collect();
    let now = Instant::now();

    take_data(config_filename);

    let elapsed = now.elapsed();
    println!("Time: {:.2?}", elapsed);

}


fn main() {
    let args: Vec<String> = std::env::args().collect();
    let num_threads = args[1].parse::<usize>().unwrap();
    let cfg_filename = &args[2];
    compute_entropy_run(num_threads, cfg_filename)
}
