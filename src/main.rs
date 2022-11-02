use quantum_circuit::compute_entropy::take_data;
use quantum_circuit::quantum_chp_state::QuantumCHPState;
use std::time::Instant;

fn main() {
    let num_threads = 4;
    rayon::ThreadPoolBuilder::new().num_threads(num_threads).build_global().unwrap();

    
    let now = Instant::now();
    
    take_data::<QuantumCHPState>(40, 2, 
                                10, 100, 
                                String::from("data.txt") );

    let elapsed = now.elapsed();
    println!("Time: {:.2?}", elapsed);

}
