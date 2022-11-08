use quantum_circuit::compute_entropy::take_data;
use quantum_circuit::quantum_chp_state::QuantumCHPState;
use quantum_circuit::quantum_graph_state::QuantumGraphState;
use std::time::Instant;

use quantum_circuit::quantum_vector_state::QuantumVectorState;
use quantum_circuit::quantum_state::{QuantumProgram, QuantumState, Entropy};
use rand::Rng;
use rayon::prelude::*;

use quantum_circuit::util;

fn test_circuit() {
    let circuit: String = String::from("
    @pragma total_num_qbits 5
    @pragma total_num_cbits 1
    h q0
    h q1
    h q2
    h q3
    h q4
    cx q0 q1
    cz q2 q3
    cz q0 q1
    cx q2 q3
    cx q1 q2
    cx q3 q4
    cz q1 q2
    cz q3 q4
    cx q0 q1
    cx q2 q3
    cx q0 q1
    cx q2 q3
    cz q1 q2
    cx q3 q4
    cz q1 q2
    cx q3 q4
    h q0
    h q2
    h q4
    z q2
    mzr q1 r0
    ");
    let qubits: Vec<usize> = vec![0];
    let mut program1 = QuantumProgram::<QuantumGraphState>::from_qasm(&circuit);
    let mut program2 = QuantumProgram::<QuantumCHPState>::from_qasm(&circuit);
    let mut program3 = QuantumProgram::<QuantumVectorState>::from_qasm(&circuit);
    program1.execute();
    program2.execute();
    program3.execute();
    
    let graph_entropy = program1.quantum_state.renyi_entropy(&qubits);
    let chp_entropy = program2.quantum_state.renyi_entropy(&qubits);
    let vector_entropy = program3.quantum_state.renyi_entropy(&qubits);

    let EPS: f32 = 0.0001;
    println!("qubits: {:?}", qubits);
    println!("graph: {}, chp: {}, vector: {}", graph_entropy, chp_entropy, vector_entropy);
    println!("graph state: {}", program1.print());
    println!("graph state from debug circuit: {}", program1.quantum_state.to_vector_state().print());
    println!("{}", program1.quantum_state.graph.partition(&qubits).print());
    println!("vector state: {}", program3.print());
}

fn test_many_circuits() {
    let num_qubits = 6;
    let num_gates = 1;
    for _ in 0..1000000 {
        let mut rng = rand::thread_rng();
        let circuit = util::generate_brick_wall_circuit(0.5, num_qubits, num_gates);
        //util::generate_random_circuit(&util::GATES, num_gates, num_qubits, 0);
        let qubits: Vec<usize> = (0..rng.gen::<usize>() % num_qubits).collect();
        let mut program1 = QuantumProgram::<QuantumGraphState>::from_qasm(&circuit);
        let mut program2 = QuantumProgram::<QuantumCHPState>::from_qasm(&circuit);
        let mut program3 = QuantumProgram::<QuantumVectorState>::from_qasm(&circuit);
        program1.execute();
        program2.execute();
        program3.execute();
        
        let graph_entropy = program1.quantum_state.renyi_entropy(&qubits);
        let chp_entropy = program2.quantum_state.renyi_entropy(&qubits);
        let vector_entropy = program3.quantum_state.renyi_entropy(&qubits);

        let EPS: f32 = 0.0001;
        if (graph_entropy - chp_entropy).abs() > EPS || 
            (graph_entropy - vector_entropy).abs() > EPS ||
            (vector_entropy - chp_entropy).abs() > EPS {
            println!("{circuit}");
            println!("qubits: {:?}", qubits);
            println!("graph: {}, chp: {}, vector: {}", graph_entropy, chp_entropy, vector_entropy);
            println!("{}", program1.print());
            println!("{}", program1.quantum_state.graph.partition(&qubits).print());
            println!("{}", program1.quantum_state.to_vector_state().print());
            println!("{}", program2.print());
            println!("{}", program3.print());
            panic!();
        }
    }
    

}


fn main() {
    let num_threads = 48;
    rayon::ThreadPoolBuilder::new().num_threads(num_threads).build_global().unwrap();

    let args: Vec<String> = std::env::args().collect();
    let now = Instant::now();

    take_data(&args[1]);

    let elapsed = now.elapsed();
    println!("Time: {:.2?}", elapsed);
}
