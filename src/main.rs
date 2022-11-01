use quantum_circuit::compute_entropy::take_data;
use quantum_circuit::util::*;
use std::time::Instant;
use rayon::prelude::*;

fn main() {
    let now = Instant::now();
    
    let num_threads = 48;

    take_data(400, 25, 1000, 100, num_threads, String::from("data.txt") );


    let elapsed = now.elapsed();
    println!("Time: {:.2?}", elapsed);
}
