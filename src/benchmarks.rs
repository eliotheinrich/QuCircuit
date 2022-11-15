
use crate::compute_entropy::take_data;
use crate::quantum_chp_state::QuantumCHPState;
use crate::quantum_graph_state::QuantumGraphState;

use std::time::Instant;

fn time_code(func_name: &str, func: &dyn Fn() -> ()) -> String {
	let now = Instant::now();
	func();
	let elapsed = now.elapsed();
	return format!("{} took {:2?}.", func_name, elapsed);
}

pub fn take_data_chp() {
	println!("{}", time_code("take_data_chp", &|| {
		take_data(&String::from("test_cfg.json"));
	}));
}


pub fn take_data_graph() {
	println!("{}", time_code("take_data_graph", &|| {
		take_data(&String::from("test_cfg.json"));
	}));
}
