use quantum_circuit::compute_entropy::{take_data, time_series};
use std::time::Instant;


fn compute_entropy_run(num_threads: usize, config_filename: &String) {
    rayon::ThreadPoolBuilder::new().num_threads(num_threads).build_global().unwrap();

    let now = Instant::now();

    take_data(config_filename);

    let elapsed = now.elapsed();
    println!("Time: {:.2?}", elapsed);

}


fn main() {
    let args: Vec<String> = std::env::args().collect();
    let num_threads = args[1].parse::<usize>().unwrap();
    //rayon::ThreadPoolBuilder::new().num_threads(num_threads).build_global().unwrap();
    let cfg_filename = &args[2];
    compute_entropy_run(num_threads, cfg_filename)
    //time_series();
}
