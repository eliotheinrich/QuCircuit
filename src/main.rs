use quantum_circuit::compute_entropy::take_data;
use quantum_circuit::quantum_chp_state::QuantumCHPState;
use quantum_circuit::quantum_graph_state::QuantumGraphState;
use std::time::Instant;

use quantum_circuit::quantum_vector_state::QuantumVectorState;
use quantum_circuit::quantum_state::{QuantumProgram, Entropy};
use rand::Rng;
use rayon::prelude::*;

use quantum_circuit::util;


fn main() {
    let num_threads = 1;
    rayon::ThreadPoolBuilder::new().num_threads(num_threads).build_global().unwrap();

    let args: Vec<String> = std::env::args().collect();
    let now = Instant::now();

    take_data::<QuantumCHPState>(&args[1]);

    let elapsed = now.elapsed();
    println!("Time: {:.2?}", elapsed);
}
