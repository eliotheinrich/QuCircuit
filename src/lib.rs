#![allow(warnings)]

pub mod quantum_state;
pub mod quantum_vector_state;
pub mod quantum_graph_state;
pub mod quantum_chp_state;
pub mod compute_entropy;
pub mod dataframe;
pub mod tests;

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
    		take_data::<QuantumCHPState>(&String::from("test_cfg.json"));
		}));
	}


	pub fn take_data_graph() {
		println!("{}", time_code("take_data_graph", &|| {
    		take_data::<QuantumGraphState>(&String::from("test_cfg.json"));
		}));
	}

}