use quantum_circuit::brickwall_run::take_data;
use std::time::Instant;


fn compute_entropy_run(args: Vec<String>) {
    let num_threads: usize = args[0].parse::<usize>().unwrap();
    let config_filename: &String = &args[1];

    let now = Instant::now();

    take_data(num_threads, config_filename);

    let elapsed = now.elapsed();
    println!("Time: {:.2?}", elapsed);
}


fn default() {
    println!("Success!");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        default();
        return;
    }

    let flags: Vec<String> = args[1].chars().into_iter().map(|x| String::from(x)).collect();
    assert!(flags[0] == "-");

    match flags[1].as_str() {
        "e" => compute_entropy_run(args[2..].to_vec()),
        _ => {
            println!("Not a valid run option.");
            panic!();
        }
    }
}
