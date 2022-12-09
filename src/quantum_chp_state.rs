use bit_vec::BitVec;
use rand_pcg::Lcg64Xsh32;
use rand::{RngCore, SeedableRng};
use serde::{Serialize, Deserialize};

use crate::quantum_state::{Entropy, QuantumState};
use crate::dataframe::DataField;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct PauliString {
	pub num_qubits: usize,
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

		PauliString { num_qubits: num_qubits, bit_string: bits, phase: rng.next_u32() % 2 == 0 }
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
		commuting_indices % 2 == 1
	}

	pub fn anticommutes(&self, other: &PauliString) -> bool {
		!self.commutes(other)
	}
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
struct Tableau {
	rows: Vec<PauliString>,
	track_destabilizers: bool,
}

impl Tableau {
	pub fn new(num_qubits: usize) -> Self {
		let mut rows: Vec<PauliString> = vec![PauliString::new(num_qubits); 2*num_qubits + 1]; 
		for i in 0..num_qubits {
			rows[i].set_x(i, true);
			rows[i + num_qubits].set_z(i, true);
		}
		
		Tableau { rows: rows, track_destabilizers: true }
	}

	pub fn print(&self) -> String {
		let mut s: String = String::new();
		for i in 0..self.rows.len() {
			s.push_str(if i == 0 { "[" } else { " " });
			s.push_str(&self.rows[i].to_string(true));
			s.push_str(if i == 2*self.rows.len() - 1 { "]" } else { "\n" });
		}
		
		s
	}

	fn x(&self, i: usize, j: usize) -> bool {
		self.rows[i].x(j)
	}

	fn x_block(&self, i: usize) -> Vec<u8> {
		(0..self.rows[i].num_qubits).map(|j| self.x(i, j) as u8).collect()
	}

	fn z_block(&self, i: usize) -> Vec<u8> {
		(0..self.rows[i].num_qubits).map(|j| self.z(i, j) as u8).collect()
	}

	fn z(&self, i: usize, j: usize) -> bool {
		self.rows[i].z(j)
	}

	fn r(&self, i: usize) -> bool {
		self.rows[i].r()
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
		assert!(self.track_destabilizers);
		let mut s: i32 = 0;
		if self.r(i) { s += 2 }
		if self.r(h) { s += 2 }

		let num_qubits: usize = self.rows.len()/2;
		for j in 0..num_qubits {
			s += Self::g(self.x(i, j), self.z(i, j), self.x(h, j), self.z(h, j));
		}
		if s % 4 == 0 {
			self.set_r(h, false);
		} else if s % 4 == 2 {
			self.set_r(h, true);
		}

		for j in 0..num_qubits {
			self.set_x(h, j, self.x(i, j) != self.x(h, j));
			self.set_z(h, j, self.z(i, j) != self.z(h, j));
		}
	}

	pub fn h_gate(&mut self, qubit: usize) {
		for i in 0..self.rows.len() {
			let x = self.x(i, qubit);
			let z = self.z(i, qubit);
			let r = self.r(i);

			// Set r_i
			self.set_r(i, r != (x && z));

			// Set x_ia
			self.set_x(i, qubit, z);
			// Set z_ia
			self.set_z(i, qubit, x);
		}
	}

	pub fn s_gate(&mut self, qubit: usize) {
		for i in 0..self.rows.len() {
			let x = self.x(i, qubit);
			let z = self.z(i, qubit);
			let r = self.r(i);

			// Set r_i
			self.set_r(i, r != (x && z));

			// Set z_ia
			self.set_z(i, qubit, x != z);
		}
	}

	pub fn x_gate(&mut self, qubit: usize) {
        self.h_gate(qubit);
        self.z_gate(qubit);
        self.h_gate(qubit);

	}

	pub fn y_gate(&mut self, qubit: usize) {
        self.x_gate(qubit);
        self.z_gate(qubit);
	}

	pub fn z_gate(&mut self, qubit: usize) {
		self.s_gate(qubit);
		self.s_gate(qubit);
	}

	pub fn cx_gate(&mut self, qubit1: usize, qubit2: usize) {
		for i in 0..self.rows.len() {
			let x1 = self.x(i, qubit1);
			let z1 = self.z(i, qubit1);
			let x2 = self.x(i, qubit2);
			let z2 = self.z(i, qubit2);

			let r = self.r(i);

			// Set r_i
			self.set_r(i, r != ((x1 && z2) && ((x2 != z1) != true)));

			// Set x2
			self.set_x(i, qubit2, x1 != x2);

			// Set z1
			self.set_z(i, qubit1, z1 != z2);
		}
	}

	pub fn mzr_qubit(&mut self, qubit: usize, mzr_outcome: bool) -> i32 {
		// Must be tracking destabilizers to perform measurements
		assert!(self.track_destabilizers);

		let num_qubits: usize = self.rows.len()/2;

		let mut found_p: bool = false;
		let mut p: usize = 0;
		for i in num_qubits..2*num_qubits {
			if self.x(i, qubit) {
				found_p = true;
				p = i;
				break;
			}
		}

		if found_p {
			for i in 0..2*num_qubits {
				if i != p && self.x(i, qubit) {
					self.rowsum(i, p);
				}
			}

			self.rows[p - num_qubits] = self.rows[p].clone();
			self.rows[p] = PauliString::new(num_qubits);
			self.set_r(p, false);
			let mut measured: i32 = 0;
			if !mzr_outcome {
				measured = 1;
				self.set_r(p, true);
			}
			self.set_z(p, qubit, true);
			return measured;

		} else {
			self.rows[2*num_qubits] = PauliString::new(num_qubits);
			//BitVec::from_elem(2*self.num_qubits, false);
			self.set_r(2*num_qubits, false);
			for i in 0..num_qubits {
				self.rowsum(2*num_qubits, i + num_qubits);
			}

			return self.r(2*num_qubits) as i32;
		}

	}
}


#[derive(Serialize, Deserialize, Clone)]
pub struct QuantumCHPState {
	num_qubits: usize,
	tableau: Tableau,

	rng: Lcg64Xsh32,
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

	// Generate a random clifford gate on N qubits following https://arxiv.org/pdf/2008.06011.pdf
	fn random_clifford<const N: usize>(&mut self, qubits: [usize; N]) {
		// First PauliString is totally random (non-identity)
		let mut pauli1: PauliString = PauliString::rand(N, &mut self.rng);

		// Second is randomly generated until it anticommutes with the first PauliString
		let mut pauli2: PauliString = {
			let mut anticommutes: bool = false;
			let mut pauli: PauliString = PauliString::rand(N, &mut self.rng);
			while !anticommutes {
				if pauli1.anticommutes(&pauli) {
					anticommutes = true;
					break
				}
				pauli = PauliString::rand(N, &mut self.rng);
			}
			pauli
		};

		let mut tableau: Tableau = Tableau { rows: vec![pauli1, pauli2] , track_destabilizers: false };

		// Step one: clear Z-block of first row
		for i in 0..N {
			if tableau.z(0, i) {
				match tableau.x(0, i) {
					true => {
						tableau.s_gate(i);
						self.s_gate(qubits[i]);
					},
					false => {
						tableau.h_gate(i);
						self.s_gate(qubits[i]);
					},
				}
			}
		}

		// Step two: clear half of nonzero coefficients in X-block of first row
		let mut nonzero_idxs: Vec<usize> = (0..N).filter(|i| tableau.x(0, *i))
												 .map(|i| i)
												 .collect();
		while nonzero_idxs.len() > 1 {
			for j in 0..nonzero_idxs.len()/2 {
				tableau.cx_gate(nonzero_idxs[2*j], nonzero_idxs[2*j+1]);
				self.cx_gate(qubits[nonzero_idxs[2*j]], qubits[nonzero_idxs[2*j+1]]);
			}

			nonzero_idxs = nonzero_idxs
				.iter()
				.enumerate()
				.filter_map(|(i, x)| if i % 2 == 0 { Some(*x) } else { None } )
				.collect();
		}


		// Step three
		if nonzero_idxs[0] != 0 {
			for i in 0..N {
				if tableau.x(0, i) {
					tableau.cx_gate(0, nonzero_idxs[0]);
					tableau.cx_gate(nonzero_idxs[0], 0);
					tableau.cx_gate(0, nonzero_idxs[0]);

					self.cx_gate(qubits[0], qubits[nonzero_idxs[0]]);
					self.cx_gate(qubits[nonzero_idxs[0]], qubits[0]);
					self.cx_gate(qubits[0], qubits[nonzero_idxs[0]]);

					break
				}
			}
		}

		// Step four
		let mut Z1p: PauliString = PauliString::new(N);
		Z1p.set_z(0, true);

		let mut Z1m: PauliString = PauliString::new(N);
		Z1m.set_z(0, true);
		Z1m.set_r(true);

		if tableau.rows[1] != Z1p && tableau.rows[1] != Z1m {
			tableau.h_gate(0);
			self.h_gate(qubits[0]);

			// Repeat steps one and two
			for i in 0..N {
				if tableau.z(1, i) {
					match tableau.x(1, i) {
					true => {
						tableau.s_gate(i);
						self.s_gate(qubits[i]);
					},
					false => {
						tableau.h_gate(i);
						self.s_gate(qubits[i]);
					},
					}
				}
			}

			let mut nonzero_idxs: Vec<usize> = (0..N).filter(|i| tableau.x(1, *i))
													.map(|i| i)
													.collect();
			while nonzero_idxs.len() > 1 {
				for j in 0..nonzero_idxs.len()/2 {
					tableau.cx_gate(nonzero_idxs[2*j], nonzero_idxs[2*j+1]);
					self.cx_gate(qubits[nonzero_idxs[2*j]], qubits[nonzero_idxs[2*j+1]]);
				}

				nonzero_idxs = nonzero_idxs
					.iter()
					.enumerate()
					.filter_map(|(i, x)| if i % 2 == 0 { Some(*x) } else { None } )
					.collect();
			}

			tableau.h_gate(0);
			self.h_gate(qubits[0]);
		}

		// Step five
		match (tableau.r(0), tableau.r(1)) {
			(false, true) => {
				tableau.x_gate(0);
				self.x_gate(qubits[0]);
			},
			(true, true) => {
				tableau.y_gate(0);
				self.y_gate(qubits[0]);
			},
			_ => {
				tableau.z_gate(0);
				self.z_gate(qubits[0]);
			}
		}
	}

	fn h_gate(&mut self, qubit: usize) {
		self.tableau.h_gate(qubit);
	}

	fn s_gate(&mut self, qubit: usize) {
		self.tableau.s_gate(qubit);
	}

	fn cx_gate(&mut self, qubit1: usize, qubit2: usize) {
		self.tableau.cx_gate(qubit1, qubit2);
	}

	fn cz_gate(&mut self, qubit1: usize, qubit2: usize) {
		self.tableau.h_gate(qubit2);
		self.tableau.cx_gate(qubit1, qubit2);
		self.tableau.h_gate(qubit2);
	}

	fn mzr_qubit(&mut self, qubit: usize) -> i32 {
		self.tableau.mzr_qubit(qubit, self.rng.next_u32() % 2 == 0)
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