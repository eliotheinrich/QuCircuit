#![allow(warnings)]

pub mod quantum_state;
pub mod quantum_vector_state;
pub mod quantum_graph_state;
pub mod quantum_chp_state;
pub mod compute_entropy;
pub mod dataframe;

pub mod util {
	use rand::Rng;

	use crate::quantum_vector_state::QuantumVectorState;
	use crate::quantum_graph_state::QuantumGraphState;
	use crate::quantum_state::{QuantumProgram, QuantumState, Entropy};

	pub const GATES: [(&str, usize, usize); 8] = [("x", 1, 0), ("y", 1, 0), ("z", 1, 0), 
											  	  ("s", 1, 0), ("h", 1, 0),
												  ("cz", 2, 0), ("cy", 2, 0), ("cz", 2, 0)];

	pub fn generate_random_circuit(gateset: &[(&str, usize, usize)], circuit_depth: usize, 
								num_qubits: usize, num_cbits:usize) -> String {
		let mut circuit: Vec<String> = Vec::new();
		circuit.push(String::from("@pragma total_num_qubits ") + &num_qubits.to_string());
		circuit.push(String::from("@pragma total_num_cbits ") + &num_cbits.to_string());

		let mut rng = rand::thread_rng();
		let num_gates: usize = gateset.len();

		let mut line = String::from("");
		let mut gate_ind: usize = 0;
		let mut bits: Vec<usize> = Vec::new();
		for _i in 0..circuit_depth {
			line = "".to_owned();
			gate_ind = rng.gen_range(0..num_gates);
			line += &gateset[gate_ind].0;

			bits = rand::seq::index::sample(&mut rng, num_qubits, gateset[gate_ind].1).into_vec();
			for j in 0..gateset[gate_ind].1 {
				line += &(" q".to_owned() + &bits[j].to_string());
			}
			bits = rand::seq::index::sample(&mut rng, num_cbits, gateset[gate_ind].2).into_vec();
			for j in 0..gateset[gate_ind].2 {
				line += &(" r".to_owned() + &bits[j].to_string());
			}
			circuit.push(line);
		}

		return circuit.join("\n");
	}

	pub fn generate_fully_entangled_circuit(num_qubits: usize) -> String {
		let mut circuit: Vec<String> = Vec::new();
		circuit.push(format!("@pragma total_num_qbits {}", 2*num_qubits));
		circuit.push(format!("@pragma total_num_cbits 1"));

		for i in 0..num_qubits {
			circuit.push(format!("h q{i}"));
		}
		for i in 0..num_qubits {
			circuit.push(format!("cx q{i} q{}", i + num_qubits));
		}
		return circuit.join("\n");
	}

	pub fn generate_brick_wall_circuit(mzr_prob: f32, num_qubits: usize, steps: usize) -> String {
		assert!(0. < mzr_prob && mzr_prob < 1. && num_qubits % 2 == 0);
		let mut rng = rand::thread_rng();

		let mut circuit: Vec<String> = Vec::new();

		circuit.push(format!("@pragma total_num_qbits {}", num_qubits));
		circuit.push(format!("@pragma total_num_cbits 1"));

		// Polarize in x-direction
		for i in 0..num_qubits {
			circuit.push(format!("h q{i}"));
		}

		for _t in 0..steps {
			// Build a block
			for _j in 0..2 {
				for q in 0..num_qubits/2 {
					if rng.gen::<u8>() % 2 == 0 {
						circuit.push(format!("cx q{} q{}", 2*q, 2*q + 1));
					} else {
						circuit.push(format!("cz q{} q{}", 2*q, 2*q + 1));
					}
				}
			}

			for _j in 0..2 {
				for q in 0..(num_qubits/2-1) {
					if rng.gen::<u8>() % 2 == 0 {
						circuit.push(format!("cx q{} q{}", 2*q+1, 2*q + 2));
					} else {
						circuit.push(format!("cz q{} q{}", 2*q+1, 2*q + 2));
					}
				}
			}

			for q in 0..num_qubits {
				if rng.gen::<f32>() < mzr_prob {
					circuit.push(format!("mzr q{} r0", q));
					circuit.push(format!("h q{}", q));
				}
			}
		}

		return circuit.join("\n");
	}

	pub fn compute_entropy<Q: QuantumState + Entropy>(circuit: &String, qubits: &Vec<usize>) -> f32 {
		let mut program = QuantumProgram::<Q>::from_qasm(circuit);
		program.execute();
		return program.quantum_state.renyi_entropy(qubits);
	}
}

#[cfg(test)]
pub mod tests {
	use rand::Rng;
	use rayon::prelude::*;

	use crate::util;

	use crate::quantum_chp_state::QuantumCHPState;
	use crate::quantum_graph_state::QuantumGraphState;
	use crate::quantum_vector_state::QuantumVectorState;
	use crate::quantum_state::{QuantumProgram, QuantumState, Entropy};


	#[test]
	fn test_simulators() {
		for i in 0..10 {
			let circuit = util::generate_random_circuit(&util::GATES, 100, 10, 0);
			let mut qc1 = QuantumProgram::<QuantumGraphState>::from_qasm(&circuit);
			let mut qc2 = QuantumProgram::<QuantumVectorState>::from_qasm(&circuit);
			qc1.execute();
			qc2.execute();

			let mut qcp = QuantumProgram::<QuantumVectorState>::from_qasm(&qc1.quantum_state.debug_circuit());
			qcp.execute();
			let mut output1 = qcp.print();
			let output1v: Vec<String> = output1.split("\n").map(|s| s.to_string()).collect();
			output1 = output1v[0..output1v.len()-1].join("\n");
			let mut output2 = qc2.print();
			let output2v: Vec<String> = output2.split("\n").map(|s| s.to_string()).collect();
			output2 = output2v[0..output2v.len()-1].join("\n");
			println!("{}", qcp.quantum_state == qc2.quantum_state);
			println!("graph: {}\nvector: {}", qcp.print(), qc2.print());
			assert!(qcp.quantum_state == qc2.quantum_state);
		}
	}

	#[test]
	pub fn test_entropy() {
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
			assert!((s.0 - s.1).abs() < 0.001 && (s.0 - s.2) < 0.001 && (s.1 - s.2).abs() < 0.001); 
		}

	}

	#[test]
	fn time_take_data_graph() {
		
	}
}


pub mod benchmarks {
	use crate::compute_entropy::take_data;
	use crate::quantum_chp_state::QuantumCHPState;
	use crate::quantum_graph_state::QuantumGraphState;

	use std::time::Instant;

	fn time_code(func_name: &str, func: &dyn Fn() -> ()) -> String {
		let now = Instant::now();
		func();
		let elapsed = now.elapsed();
		return format!("{} took {:2?}.", func_name, elapsed);
	}

	pub fn take_data_chp() {
		println!("{}", time_code("take_data_chp", &|| {
			let system_size = 10;
			let partition_sizes: Vec<usize> = vec![1, 3, 5];
			let probs: Vec<f32> = vec![0.06, 0.08, 0.1, 0.138, 0.16];
    		take_data::<QuantumCHPState>(10, &partition_sizes, &probs, 1000, 100, String::from("_data.txt") )
		}));
	}


	pub fn take_data_graph() {
		println!("{}", time_code("take_data_graph", &|| {
			let system_size = 10;
			let partition_sizes: Vec<usize> = vec![1, 3, 5];
			let probs: Vec<f32> = vec![0.06, 0.08, 0.1, 0.138, 0.16];
    		take_data::<QuantumGraphState>(10, &partition_sizes, &probs, 1000, 100, String::from("_data.txt") )
		}));
	}

}