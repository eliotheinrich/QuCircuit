// TODO: Redo all unit tests; it's a mess in here.

#[cfg(test)]
pub mod tests {
	use rand::Rng;
	use rand::rngs::ThreadRng;
	use rayon::prelude::*;

	use crate::util;

	use crate::quantum_chp_state::QuantumCHPState;
	use crate::quantum_graph_state::QuantumGraphState;
	use crate::quantum_vector_state::QuantumVectorState;
	use crate::quantum_state::{QuantumProgram, QuantumState, Entropy, MzrForce};
	use crate::dataframe::{Sample, DataSlide};

	const EPS: f32 = 0.0001;

	fn isclose(f1: f32, f2: f32) -> bool {
		return (f1 - f2).abs() < EPS
	}


	#[derive(Debug)]
	enum Instruction {
		S(usize),
		H(usize),
		MZR(usize, bool),
		CZ(usize, usize),

	}

	impl Instruction {
		pub fn random(rng: &mut ThreadRng, num_qubits: usize) -> Self {
			let num_cmds: u8 = if num_qubits > 1 { 4 } else { 3 };
			match rng.gen::<u8>() % num_cmds {
				0 => Instruction::S(rng.gen::<usize>() % num_qubits),
				1 => Instruction::H(rng.gen::<usize>() % num_qubits),
				2 => Instruction::MZR(rng.gen::<usize>() % num_qubits, rng.gen::<usize>() % 2 == 0),
				3 => {
					let r1 = rng.gen::<usize>() % num_qubits;
					let mut r2 = rng.gen::<usize>() % num_qubits;
					while r1 == r2 {
						r2 = rng.gen::<usize>() % num_qubits;
					}
					Instruction::CZ(r1, r2)
				}
				_ => panic!()
			}
		}
	}


	#[test]
	fn test_simulators() {
		for i in 0..100 {
			println!("{i}");
			let circuit = util::generate_random_circuit(&util::GATES, 100, 10, 0);
			let mut qc1 = QuantumProgram::<QuantumVectorState>::from_qasm(&circuit);
			let mut qc2 = QuantumProgram::<QuantumGraphState>::from_qasm(&circuit);
			let mut qc3 = QuantumProgram::<QuantumCHPState>::from_qasm(&circuit);
			qc1.execute();
			qc2.execute();
			qc3.execute();

			let state1 = qc1.quantum_state;
			let state2 = qc2.quantum_state.to_vector_state();
			let state3 = qc3.quantum_state.to_vector_state();


			assert!(state1 == state2);
			assert!(state1 == state3);
			assert!(state2 == state3);
		}
	}

	use crate::brickwall_run::timesteps_qa;

	#[test]
	fn test_entropy() {
		let num_qubits = 50;
		let num_gates = 100;
		for i in 0..100 {
			let mut rng = rand::thread_rng();
			let qubits: Vec<usize> = (0..rng.gen::<usize>()%num_qubits).collect();
			//println!("qubits: {:?}", qubits);

			let mut state1 = QuantumCHPState::new(num_qubits);
			let mut state2 = QuantumGraphState::new(num_qubits);
			//let mut state3 = QuantumVectorState::new(num_qubits);

			timesteps_qa(&mut state1, num_gates, 0.1);
			timesteps_qa(&mut state2, num_gates, 0.1);

				//if !isclose(state1.renyi_entropy(&qubits), state3.renyi_entropy(&qubits)) {
					//println!("After: ");
					//println!("chp: \n{}, {}", state1.to_vector_state().print(), state1.renyi_entropy(&qubits));
					//println!("vector: \n{}, {}", state3.print(), state3.renyi_entropy(&qubits));
				//}
			//}
			
			let chp_entropy = state1.renyi_entropy(&qubits);
			//let vector_entropy = state3.renyi_entropy(&qubits);
			let graph_entropy = state2.renyi_entropy(&qubits);

			println!("{chp_entropy}, {graph_entropy}");

			assert!(isclose(graph_entropy, chp_entropy));

		}
	}

	#[test]
	fn test_sample() {
		let mut v1: Vec<f32> = vec![1.5, 3.2, 5.3, 2.2];
		let m1: f32 = v1.iter().sum::<f32>()/(v1.len() as f32);
		let s1: f32 = (v1.iter().map(|x| (x - m1).powi(2)).sum::<f32>()/(v1.len() as f32)).powf(0.5);
		let sample1 = Sample::from(&v1);
		assert!(isclose(m1, sample1.mean));
		assert!(isclose(s1, sample1.std));
		assert!(v1.len() == sample1.num_samples);

		let mut v2: Vec<f32> = vec![4.3, 7.8, 9.2];
		let m2: f32 = v2.iter().sum::<f32>()/(v2.len() as f32);
		let s2: f32 = (v2.iter().map(|x| (x - m2).powi(2)).sum::<f32>()/(v2.len() as f32)).powf(0.5);
		let sample2 = Sample::from(&v2);
		assert!(isclose(m2, sample2.mean));
		assert!(isclose(s2, sample2.std));
		assert!(v2.len() == sample2.num_samples);



		let mut v3: Vec<f32> = Vec::new();
		v3.append(&mut v1);
		v3.append(&mut v2);
		let m3: f32 = v3.iter().sum::<f32>()/(v3.len() as f32);
		let s3: f32 = (v3.iter().map(|x| (x - m3).powi(2)).sum::<f32>()/(v3.len() as f32)).powf(0.5);
		let sample3 = Sample::from(&v3);
		assert!(isclose(m3, sample3.mean));
		assert!(isclose(s3, sample3.std));
		assert!(v3.len() == sample3.num_samples);




	}

	#[test]
	fn test_dataslide() {
		let mut ds1: DataSlide = DataSlide::new();
		let mut ds2: DataSlide = DataSlide::new();
		ds1.add_int_param("int1", 1);
		ds2.add_int_param("int1", 1);

		ds1.add_float_param("float1", 0.1);
		ds2.add_float_param("float1", 0.1);

		ds1.add_data("data_key");
		ds1.push_data("data_key", Sample { mean: 0.5, std: 0.0, num_samples: 3 });
		ds1.push_data("data_key", Sample { mean: 1.0, std: 0.0, num_samples: 8 });

		ds2.add_data("data_key");
		ds2.push_data("data_key", Sample { mean: 1.0, std: 0.0, num_samples: 3 });
		ds2.push_data("data_key", Sample { mean: 1.5, std: 0.0, num_samples: 8 });


		//let mut ds3 = ds1.combine(&ds2);
		println!("{}", ds1.congruent(&ds2));
		//println!("{:?}", ds3);

	}
	

	#[test]
	fn test_chp_vs_vector() {
		let num_qubits: usize = 5;
		let circuit_depth: usize = 100;
		let mut rng = rand::thread_rng();
		for i in 0..100 {
			println!("run #{i}");

			let mut state1 = QuantumCHPState::new(num_qubits);
			let mut state2 = QuantumGraphState::new(num_qubits);
			let mut state3 = QuantumVectorState::new(num_qubits);

			let circuit: Vec<Instruction> = (0..circuit_depth).map(|_| Instruction::random(&mut rng, num_qubits)).collect();
			for j in 0..circuit_depth {
				match circuit[j] {
					Instruction::S(x) => {
						state1.s_gate(x);
						state2.s_gate(x);
						state3.s_gate(x);
					}
					Instruction::H(x) => {
						state1.h_gate(x);
						state2.h_gate(x);
						state3.h_gate(x);
					}
					Instruction::MZR(x, b) => {
						if !state1.mzr_qubit_forced(x, b) {
							state1.mzr_qubit_forced(x, !b);
						}
						if !state2.mzr_qubit_forced(x, b) {
							state2.mzr_qubit_forced(x, !b);
						}
						if !state3.mzr_qubit_forced(x, b) {
							state3.mzr_qubit_forced(x, !b);
						}
					}
					Instruction::CZ(x, y) => {
						state1.cz_gate(x, y);
						state2.cz_gate(x, y);
						state3.cz_gate(x, y);
					}

				}
			}
			if state1.to_vector_state() != state2.to_vector_state() {
				panic!();
			}
		}
	}
}

