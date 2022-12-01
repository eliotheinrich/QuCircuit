#[cfg(test)]
pub mod tests {
	use rand::Rng;
	use rayon::prelude::*;

	use crate::util;

	use crate::quantum_chp_state::QuantumCHPState;
	use crate::quantum_graph_state::QuantumGraphState;
	use crate::quantum_vector_state::QuantumVectorState;
	use crate::quantum_state::{QuantumProgram, QuantumState, Entropy};
	use crate::dataframe::Sample;

	const EPS: f32 = 0.0001;

	fn isclose(f1: f32, f2: f32) -> bool {
		return (f1 - f2).abs() < EPS
	}

	#[test]
	fn test_simulators() {
		for i in 0..100 {
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
		let num_qubits = 30;
		let num_gates = 100;
		let entropy: Vec<(f32, f32)> = 
		(0..100).into_par_iter().map(|_| {
			let mut rng = rand::thread_rng();
			let circuit = util::generate_brick_wall_circuit(0.1, num_qubits, 100);
			//util::generate_random_circuit(&util::GATES, num_gates, num_qubits, 0);
			let qubits: Vec<usize> = (0..rng.gen::<usize>()%num_qubits).collect();
			let mut program1 = QuantumProgram::<QuantumGraphState>::from_qasm(&circuit);
			let mut program2 = QuantumProgram::<QuantumCHPState>::from_qasm(&circuit);
			//let mut program3 = QuantumProgram::<QuantumVectorState>::from_qasm(&circuit);
			program1.execute();
			program2.execute();
			//program3.execute();
			
			let graph_entropy = program1.quantum_state.renyi_entropy(&qubits);
			let chp_entropy = program2.quantum_state.renyi_entropy(&qubits);
			//let vector_entropy = program3.quantum_state.renyi_entropy(&qubits);


			(graph_entropy, chp_entropy)//, vector_entropy)
		}).collect();
		
		for s in entropy {
			assert!(isclose(s.0, s.1)); 
		}

	}

	#[test]
	pub fn test_sample() {

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
}

