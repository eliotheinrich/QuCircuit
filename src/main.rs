use quantum_circuit::compute_entropy::take_data;
use quantum_circuit::quantum_chp_state::QuantumCHPState;
use std::time::Instant;

fn main() {
    let num_threads = 1;
    rayon::ThreadPoolBuilder::new().num_threads(num_threads).build_global().unwrap();
    //let probs: Vec<f32> = vec![0.01, 0.02, 0.03, 0.04, 0.05, 0.06, 0.08, 0.138];

    let args: Vec<String> = std::env::args().collect();
    println!("{:?}", args);
    let now = Instant::now();

    take_data::<QuantumCHPState>(&args[1]);

    let elapsed = now.elapsed();
    println!("Time: {:.2?}", elapsed);

}
