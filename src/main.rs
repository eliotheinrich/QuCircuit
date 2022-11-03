use quantum_circuit::compute_entropy::take_data;
use quantum_circuit::quantum_chp_state::QuantumCHPState;
use std::time::Instant;

fn main() {
    let num_threads = 1;
    rayon::ThreadPoolBuilder::new().num_threads(num_threads).build_global().unwrap();

    
    let system_size = 100;
    let num_system_sizes = 25;
    let partition_sizes: Vec<usize> = (0..system_size/2).step_by(system_size/num_system_sizes).collect();
    let timesteps = 50000;

    let probs: Vec<f32> = vec![0.06, 0.08, 0.1, 0.138, 0.16];
    //let probs: Vec<f32> = vec![0.01, 0.02, 0.03, 0.04, 0.05, 0.06, 0.08, 0.138];

    let now = Instant::now();

    take_data::<QuantumCHPState>(system_size, &partition_sizes, 
                                 &probs, timesteps, 5,
                                 String::from("data_small.txt") );

    let elapsed = now.elapsed();
    println!("Time: {:.2?}", elapsed);

}
