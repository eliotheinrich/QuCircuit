use bit_vec::BitVec;
use rand_pcg::Lcg64Xsh32;
use rand::{RngCore, SeedableRng};
use serde::{Serialize, Deserialize};

use crate::quantum_state::{Entropy, QuantumState};
use crate::dataframe::DataField;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct PauliString {
	num_qubits: usize,
	bit_string: BitVec,
	phase: bool,
}

impl PauliString {
	pub fn new(num_qubits: usize) -> Self {
		PauliString { num_qubits: num_qubits, bit_string: BitVec::from_elem(2*num_qubits, false), phase: false }
	}

	// Generates a random non-identity PauliString
	pub fn rand(num_qubits: usize, rng: &mut Lcg64Xsh32) -> Self {
		let i: u32 = rng.next_u32() % (4_u32.pow(num_qubits as u32) - 1) + 1;
		let mut bits: BitVec = BitVec::from_elem(2*num_qubits, false);
		for j in 0..(2*num_qubits) {
			bits.set(j, (i >> j & 1) != 0);
		}

		let p = PauliString { num_qubits: num_qubits, bit_string: bits, phase: rng.next_u32() % 2 == 0 };
		println!("rand: {} -> {}", i, p.to_string(true));
		p
	}

	fn to_op(&self, i: usize) -> &str {
		match (self.x(i), self.z(i)) {
			(false, false) => "I",
			(true,  false) => "X",
			(false, true) =>  "Z",
			(true,  true) =>  "Y",
		}
	}

	pub fn to_string(&self, to_ops: bool) -> String {
		if to_ops {
			let mut s: String = String::from("");
			s.push_str("[");
			s.push_str(if self.phase { "-" } else { "+" });
			for i in 0..self.num_qubits {
				s.push_str(self.to_op(i));
			}
			s.push_str("]");
			s
		} else {
			format!("[{:?} | {}]", self.bit_string, if self.phase { 1 } else { 0 } )
		}
	}

	pub fn x(&self, i: usize) -> bool {
		self.bit_string[i]
	}

	pub fn z(&self, i: usize) -> bool {
		self.bit_string[i + self.num_qubits]
	}

	pub fn r(&self) -> bool {
		self.phase
	}

	pub fn set_x(&mut self, i: usize, val: bool) {
		self.bit_string.set(i, val);
	}

	pub fn set_z(&mut self, i: usize, val: bool) {
		self.bit_string.set(i + self.num_qubits, val);
	}

	pub fn set_r(&mut self, val: bool) {
		self.phase = val;
	}

	pub fn commutes_at(&self, other: &PauliString, i: usize) -> bool {
		if (self.x(i) == other.x(i)) && (self.z(i) == other.z(i)) {
			true
		} else if !self.x(i) && !self.z(i) {
			true
		} else if !other.x(i) && !other.z(i) {
			true
		} else {
			false
		}
	}

	pub fn commutes(&self, other: &PauliString) -> bool {
		let commuting_indices: usize = (0..self.num_qubits).map(|i| {
			self.commutes_at(other, i)
		}).filter(|i| *i).count();
		commuting_indices % 2 == 0
	}

	pub fn anticommutes(&self, other: &PauliString) -> bool {
		!self.commutes(other)
	}
}

#[derive(Serialize, Deserialize, Clone)]
struct Tableau {
	num_qubits: usize,
	rows: Vec<PauliString>,
	//rows: Vec<BitVec>,
	//phase: BitVec,
}

impl Tableau {
	pub fn new(num_qubits: usize) -> Self {
		let mut rows: Vec<PauliString> = vec![PauliString::new(num_qubits); 2*num_qubits + 1]; 
		for i in 0..num_qubits {
			rows[i].set_x(i, true);
			rows[i + num_qubits].set_z(i, true);
		}
		return Tableau { num_qubits: num_qubits, rows: rows };
	}

	pub fn print(&self) -> String {
		let mut s: String = String::new();
		for i in 0..2*self.num_qubits {
			s.push_str(if i == 0 { "[" } else { " " });
			s.push_str(&self.rows[i].to_string(true));
			s.push_str(if i == 2*self.num_qubits - 1 { "]" } else { "\n" });
		}
		return s;
	}

	fn x(&self, i: usize, j: usize) -> bool {
		return self.rows[i].x(j);
	}

	fn z(&self, i: usize, j: usize) -> bool {
		return self.rows[i].z(j);
	}

	fn r(&self, i: usize) -> bool {
		return self.rows[i].r();
	}

	fn set_x(&mut self, i: usize, j: usize, v: bool) {
		self.rows[i].set_x(j, v);
	}

	fn set_z(&mut self, i: usize, j: usize, v: bool) {
		self.rows[i].set_z(j, v);
	}

	fn set_r(&mut self, i: usize, v: bool) {
		self.rows[i].set_r(v);
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


#[derive(Serialize, Deserialize, Clone)]
pub struct QuantumCHPState {
	num_qubits: usize,
	tableau: Tableau,

	rng: Lcg64Xsh32,
}

impl QuantumCHPState {

	pub fn random_clifford<const N: usize>(&mut self, qubits: [usize; N]) {
		let mut row1: PauliString = PauliString::rand(N, &mut self.rng);
		let mut row2: PauliString = {
			let mut anticommutes: bool = false;
			let mut p: PauliString = PauliString::rand(N, &mut self.rng);
			while !anticommutes {
				if row1.anticommutes(&p) {
					anticommutes = true;
					break
				}
				anticommutes=true;
			}

			p
		};

		//println!("{}, {}", row1.to_string(true), row2.to_string(true));
	}
}

impl QuantumState for QuantumCHPState {
	fn new(num_qubits: usize) -> Self {
		return QuantumCHPState { num_qubits: num_qubits, tableau: Tableau::new(num_qubits), rng: Lcg64Xsh32::from_entropy() };
	}

	fn print(&self) -> String {
		let mut s: String = String::new();
		s.push_str(&format!("Tableau: \n"));
		s.push_str(&self.tableau.print());
		return s;
	}

	fn system_size(&self) -> usize {
		return self.num_qubits;
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

			let r = self.tableau.r(i);

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
		let mut found_p: bool = false;
		let mut p: usize = 0;
		for i in (self.num_qubits)..2*self.num_qubits {
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
			self.tableau.rows[p] = PauliString::new(self.num_qubits);
			//BitVec::from_elem(2*self.num_qubits, false);
			self.tableau.set_r(p, false);
			let mut measured: i32 = 0;
			if self.rng.next_u32() % 2 == 0 {
				measured = 1;
				self.tableau.set_r(p, true);
			}
			self.tableau.set_z(p, qubit, true);
			return measured;

		} else {
			self.tableau.rows[2*self.num_qubits] = PauliString::new(self.num_qubits);
			//BitVec::from_elem(2*self.num_qubits, false);
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
		s.push_str(&format!("]\n"));
	}
	println!("{s}");
}

impl Entropy for QuantumCHPState {
	fn renyi_entropy(&self, qubits: &Vec<usize>) -> f32 {
		// First, truncate tableau to subsystem A

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
			found_pivot = false;
			for i in row..self.num_qubits {
				if truncated_tableau[i][c] {
					pivot_row = i;
					found_pivot = true;
					break;
				}
			}

			if found_pivot {
				truncated_tableau.swap(row, pivot_row);

				for i in (row+1)..self.num_qubits {
					if truncated_tableau[i][c] {
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
				leading += 2;
				continue;
			}
		}

		// Now to compute the rank
		let mut rank: usize = 0;
		for i in 0..self.num_qubits {
			if truncated_tableau[i].any() {
				rank += 1;
			}
		}

		return rank as f32 - qubits.len() as f32;
	}
}