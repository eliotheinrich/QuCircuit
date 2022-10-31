use quantum_circuit::compute_entropy::take_data;
use quantum_circuit::util::*;
use rayon::{ThreadPoolBuilder};


use std::fs;

fn main() {
    //take_data(5, 2, 100, 100, String::from("data.txt"), );
    let pool = ThreadPoolBuilder::new().num_threads(4).build().unwrap();
    let mut vals: Vec<i32> = Vec::new();
    for i in 0..5 {
        vals.push(pool.install(|| i*i));
    }
    println!("{:?}", vals);
}
