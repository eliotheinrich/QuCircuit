use crate::quantum_chp_state::QuantumCHPState;
use crate::quantum_state::{QuantumState, Entropy};
use crate::dataframe::DataFrame;

use rayon::prelude::*;
use rand::Rng;

pub fn compute_entropy<Q: QuantumState + Entropy>(system_size: usize, subsystem_size: usize, mzr_prob: f32, time: usize) -> f32 {
    assert!(0. < mzr_prob && mzr_prob < 1.);
    let mut quantum_state: Q = Q::new(system_size);
    let mut rng = rand::thread_rng();
    for t in 0..time {
        for i in 0..system_size/2 { // first layer of unitaries
            if rng.gen::<u8>() % 2 == 0 {
                quantum_state.cz_gate(2*i, (2*i + 1) % system_size);
            } else {
                quantum_state.cx_gate(2*i, (2*i + 1) % system_size);
            }
        }
        for i in 0..system_size/2 { // second layer of unitaries
            if rng.gen::<u8>() % 2 == 0 {
                quantum_state.cz_gate((2*i + 1) % system_size, (2*i + 2) % system_size);
            } else {
                quantum_state.cx_gate((2*i + 1) % system_size, (2*i + 2) % system_size);
            }
        }

        for i in 0..system_size {
            if rng.gen::<f32>() < mzr_prob {
                quantum_state.mzr_qubit(i);
                quantum_state.h_gate(i);
            }
        }
    }

    let qubits: Vec<usize> = (0..subsystem_size).collect();
    return quantum_state.renyi_entropy(&qubits);
}


fn gen_dataframe<Q: QuantumState + Entropy>(system_size: usize, partition_size: usize, prob: f32, timesteps: usize, num_runs: usize) -> DataFrame {
	let mut df: DataFrame = DataFrame::new();
	df.add_int_param("L", system_size as i32);
	df.add_int_param("LA", partition_size as i32);
	df.add_float_param("p", prob);
	df.add_data("entropy");

	for n in 0..num_runs {
		df.push_data("entropy", compute_entropy::<Q>(system_size, partition_size, prob, timesteps));
	}

	return df;
}



pub fn take_data<Q: QuantumState + Entropy>(system_size: usize, num_system_sizes: usize, 
                 timesteps: usize, num_runs: usize, filename: String) {
    
    let probs: Vec<f32> = vec![0.06, 0.08, 0.1, 0.138, 0.16];
    let partition_sizes: Vec<usize> = (0..system_size/2).step_by(system_size/num_system_sizes).collect();

    let num_sizes: usize = partition_sizes.len();
    let num_probs: usize = probs.len();

    let mut params: Vec<(f32, usize)> = Vec::new();
    for i in 0..num_probs {
        for j in 0..num_sizes {
            params.push((probs[i], partition_sizes[j]));
        }
    }

    let data: Vec<DataFrame> = params.into_par_iter().map(|x| {
                                        gen_dataframe::<Q>(system_size, x.1, x.0, timesteps, num_runs)
                                   }).collect();

    println!("done!");
    DataFrame::write_dataframes(&filename, data);
}

