use quantum_circuit::compute_entropy::take_data;
use quantum_circuit::quantum_chp_state::QuantumCHPState;
use std::time::Instant;

fn main() {
    let num_threads = 48;
    rayon::ThreadPoolBuilder::new().num_threads(num_threads).build_global().unwrap();

    
    let now = Instant::now();
    
    take_data::<QuantumCHPState>(400, 25, 
                                100000, 5,
                                String::from("data2.txt") );

    let elapsed = now.elapsed();
    println!("Time: {:.2?}", elapsed);

}
