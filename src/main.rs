use quantum_circuit::brickwall_run::take_data;
use std::time::Instant;


fn compute_entropy_run(num_threads: usize, config_filename: &String) {

    let now = Instant::now();

    take_data(num_threads, config_filename);

    let elapsed = now.elapsed();
    println!("Time: {:.2?}", elapsed);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    //let num_threads = args[1].parse::<usize>().unwrap();
    //let cfg_filename = &args[2];
    
    //compute_entropy_run(num_threads, cfg_filename);

    use quantum_circuit::quantum_chp_state::QuantumCHPState;
    use quantum_circuit::quantum_state::QuantumState;
    let mut state: QuantumCHPState = QuantumCHPState::new(3);
    for i in 0..20 {
        state.random_clifford([1]);
    }

}
