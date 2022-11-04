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

    //let args: Vec<String> = std::env::args().collect();
    //let now = Instant::now();

    //take_data::<QuantumCHPState>(&args[1]);

    //let elapsed = now.elapsed();
    //println!("Time: {:.2?}", elapsed);

    let num_qubits = 6;
    let num_gates = 100;
    let entropy: Vec<(f32, f32, f32)> = 
    (0..10).into_par_iter().map(|_| {
        let mut rng = rand::thread_rng();
        let circuit = util::generate_brick_wall_circuit(0.1, num_qubits, 100);
        //util::generate_random_circuit(&util::GATES, num_gates, num_qubits, 0);
        let qubits: Vec<usize> = (0..rng.gen::<usize>()%num_qubits).collect();
        let mut program1 = QuantumProgram::<QuantumGraphState>::from_qasm(&circuit);
        let mut program2 = QuantumProgram::<QuantumCHPState>::from_qasm(&circuit);
        let mut program3 = QuantumProgram::<QuantumVectorState>::from_qasm(&circuit);
        program1.execute();
        program2.execute();
        program3.execute();
        
        let graph_entropy = program1.quantum_state.renyi_entropy(&qubits);
        let chp_entropy = program2.quantum_state.renyi_entropy(&qubits);
        let vector_entropy = program3.quantum_state.renyi_entropy(&qubits);


        (graph_entropy, chp_entropy, vector_entropy)
    }).collect();
    
    for s in entropy {
        if !(((s.0 - s.1).abs() < 0.001 && (s.0 - s.2) < 0.001 && (s.1 - s.2).abs() < 0.001)) {
            println!("graph: {}, chp: {}, vector: {}", s.0, s.1, s.2);
        } 
    }
}
