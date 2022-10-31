use bit_vec::BitVec;
use rand::Rng;
use rand::rngs::ThreadRng;

use crate::quantum_state::{Entropy, QuantumState};

struct Tableau {
	num_qubits: usize,
	rows: Vec<BitVec>,
	phase: BitVec,
}

impl Tableau {
	pub fn new(num_qubits: usize) -> Self {
		let mut rows: Vec<BitVec> = vec![BitVec::from_elem(2*num_qubits, false); 2*num_qubits + 1]; 
		let phase: BitVec = BitVec::from_elem(2*num_qubits + 1, false);
		for i in 0..2*num_qubits {
			rows[i].set(i, true);
		}
		return Tableau { num_qubits: num_qubits, rows: rows, phase: phase };
	}

	pub fn print(&self) -> String {
		let mut s: String = String::new();
		for i in 0..2*self.num_qubits {
			s.push_str(&format!("["));
			for j in 0..2*self.num_qubits {
				s.push_str(&format!("{}", if self.rows[i][j] { 1 } else { 0 }));
				if j != 2*self.num_qubits - 1 { s.push_str(&format!(" ")); }
			}
			s.push_str(&format!(" | {}]\n", if self.phase[i] { 1 } else { 0 }));
		}
		return s;
	}

	fn x(&self, i: usize, j: usize) -> bool {
		return self.rows[i][j];
	}

	fn z(&self, i: usize, j: usize) -> bool {
		return self.rows[i][j + self.num_qubits];
	}

	fn r(&self, i: usize) -> bool {
		return self.phase[i];
	}

	fn set_x(&mut self, i: usize, j: usize, v: bool) {
		self.rows[i].set(j, v);
	}

	fn set_z(&mut self, i: usize, j: usize, v: bool) {
		self.rows[i].set(j + self.num_qubits, v);
	}

	fn set_r(&mut self, i: usize, v: bool) {
		self.phase.set(i, v);
	}

	fn g(x1: bool, z1: bool, x2: bool, z2: bool) -> i32 {
		if !x1 && !z1 { 
			return 0 
		} else if x1 && z1 { // z2 - x2
			if z2 { return if x2 { 0 } else { 1 } }
			else { return if x2 { -1 } else { 0 } }
		} else if x1 && !z1 { // z2 * (2*x2 - 1)
			if z2 { return if x2 { 1 } else { -1 } }
			else { return 0 }
		} else { // x2 * (1 - 2*z2) 
			if x2 { return if z2 { -1 } else { 1 } }
			else { return 0 }
		}
	}

	pub fn rowsum(&mut self, h: usize, i: usize) {
		let mut s: i32 = 0;
		if self.r(i) { s += 2 }
		if self.r(h) { s += 2 }

		for j in 0..self.num_qubits {
			s += Self::g(self.x(i, j), self.z(i, j), self.x(h, j), self.z(h, j));
		}
		if s % 4 == 0 {
			self.set_r(h, false);
		} else if s % 4 == 2 {
			self.set_r(h, true);
		}

		for j in 0..self.num_qubits {
			self.set_x(h, j, self.x(i, j) != self.x(h, j));
			self.set_z(h, j, self.z(i, j) != self.z(h, j));
		}
	}
}


pub struct QuantumCHPState {
	num_qubits: usize,
	tableau: Tableau,
	rng: ThreadRng,
}

impl QuantumState for QuantumCHPState {
	fn new(num_qubits: usize) -> Self {
		return QuantumCHPState { num_qubits: num_qubits, tableau: Tableau::new(num_qubits), rng: rand::thread_rng() };
	}

	fn print(&self) -> String {
		let mut s: String = String::new();
		s.push_str(&format!("Tableau: \n"));
		s.push_str(&self.tableau.print());
		return s;
	}

	fn h_gate(&mut self, qubit: usize) {
		for i in 0..2*self.num_qubits {
			let x = self.tableau.x(i, qubit);
			let z = self.tableau.z(i, qubit);
			let r = self.tableau.r(i);

			// Set r_i
			self.tableau.set_r(i, r != (x && z));

			// Set x_ia
			self.tableau.set_x(i, qubit, z);
			// Set z_ia
			self.tableau.set_z(i, qubit, x);
		}
	}

	fn s_gate(&mut self, qubit: usize) {
		for i in 0..2*self.num_qubits {
			let x = self.tableau.x(i, qubit);
			let z = self.tableau.z(i, qubit);
			let r = self.tableau.r(i);

			// Set r_i
			self.tableau.set_r(i, r != (x && z));

			// Set z_ia
			self.tableau.set_z(i, qubit, x != z);
		}
	}

	fn cx_gate(&mut self, qubit1: usize, qubit2: usize) {
		for i in 0..2*self.num_qubits {
			let x1 = self.tableau.x(i, qubit1);
			let z1 = self.tableau.z(i, qubit1);
			let x2 = self.tableau.x(i, qubit2);
			let z2 = self.tableau.z(i, qubit2);

			let r = self.tableau.phase[i];

			// Set r_i
			self.tableau.set_r(i, r != ((x1 && z2) && ((x2 != z1) != true)));

			// Set x2
			self.tableau.set_x(i, qubit2, x1 != x2);

			// Set z1
			self.tableau.set_z(i, qubit1, z1 != z2);
		}
	}

	fn cz_gate(&mut self, qubit1: usize, qubit2: usize) {
		self.h_gate(qubit2);
		self.cx_gate(qubit1, qubit2);
		self.h_gate(qubit2);
	}

	fn mzr_qubit(&mut self, qubit: usize) -> i32 {
		// TODO
		let mut found_p: bool = false;
		let mut p: usize = 0;
		for i in (self.num_qubits+1)..2*self.num_qubits {
			if self.tableau.x(i, qubit) {
				found_p = true;
				p = i;
				break;
			}
		}

		if found_p {
			for i in 0..2*self.num_qubits {
				if i != p && self.tableau.x(i, qubit) {
					self.tableau.rowsum(i, p);
				}
			}

			self.tableau.rows[p - self.num_qubits] = self.tableau.rows[p].clone();
			self.tableau.rows[p] = BitVec::from_elem(2*self.num_qubits, false);
			self.tableau.set_r(p, false);
			let mut measured: i32 = 0;
			if self.rng.gen::<u8>() % 2 == 0 {
				measured = 1;
				self.tableau.set_r(p, true);
			}
			self.tableau.set_z(p, qubit, true);
			return measured;

		} else {
			self.tableau.rows[2*self.num_qubits] = BitVec::from_elem(2*self.num_qubits, false);
			self.tableau.set_r(2*self.num_qubits, false);
			for i in 0..self.num_qubits {
				self.tableau.rowsum(2*self.num_qubits, i + self.num_qubits);
			}

			return self.tableau.r(2*self.num_qubits) as i32;
		}

	}

}

fn print_tableau(tableau: &Vec<BitVec>) {
	let rows = tableau.len();
	let cols = tableau[0].len();
	let mut s: String = String::new();
	for i in 0..rows {
		s.push_str(&format!("["));
		for j in 0..cols {
			s.push_str(&format!("{}", if tableau[i][j] { 1 } else { 0 }));
			if j != cols - 1 { s.push_str(&format!(" ")); }
		}
		s.push_str(&format!(" | {}]\n", if true { 1 } else { 0 }));
	}
	println!("{s}");
}

impl Entropy for QuantumCHPState {
	fn renyi_entropy(&self, qubits: &Vec<usize>) -> f32 {
		// First, truncate tableau to subsystem A
		//println!("qubits: {:?}", qubits);
		//println!("calling renyi entropy on full tableau: \n{}", self.tableau.print());

		let mut truncated_tableau: Vec<BitVec> = vec![BitVec::from_elem(2*qubits.len(), false); self.num_qubits];
		for i in 0..self.num_qubits {
			for j in 0..qubits.len() {
				truncated_tableau[i].set(j, 			   self.tableau.x(i + self.num_qubits, qubits[j]));
				truncated_tableau[i].set(j + qubits.len(), self.tableau.z(i + self.num_qubits, qubits[j]));				
			}
		}

		// Do Gaussian elimination to determine rank of truncated tableau
		let mut found_pivot: bool = false;
		let mut pivot_row: usize = 0;
		let mut row: usize = 0;
		let mut leading: usize = 0;

		for c in 0..2*qubits.len() {
			//println!("on col {c}");
			//println!("before: ");
			//print_tableau(&truncated_tableau);
			found_pivot = false;
			for i in row..self.num_qubits {
				if truncated_tableau[i][c] {
					pivot_row = i;
					found_pivot = true;
					break;
				}
			}

			if found_pivot {
				//println!("pivoting on {pivot_row} with {:?}\n", truncated_tableau[pivot_row]);
				truncated_tableau.swap(row, pivot_row);

				for i in (row+1)..self.num_qubits {
					if truncated_tableau[i][c] {
						//println!("cancelling row {i}");
						for j in 0..2*qubits.len() {
							let v1 = truncated_tableau[row][j];
							let v2 = truncated_tableau[i][j];
							truncated_tableau[i].set(j, v1 ^ v2);
						}
					}
				}
				leading += 1;
				row += 1;
			} else { 
				//println!("no pivot; moving on");
				leading += 2;
				continue;
			}
			//println!("after: ");
			//print_tableau(&truncated_tableau);
		}

		// Now to compute the rank
		let mut rank: usize = 0;
		for i in 0..self.num_qubits {
			if truncated_tableau[i].any() {
				rank += 1;
			}
		}

		//println!("final: ");
		//print_tableau(&truncated_tableau);
		return rank as f32 - qubits.len() as f32;
	}
}