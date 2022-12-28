use bit_vec::BitVec;
use num::complex::Complex;
use rand_pcg::Lcg64Xsh32;
use rand::{RngCore, SeedableRng};
use serde::{Serialize, Deserialize};

use crate::quantum_state::{Entropy, QuantumState, MzrForce};
use crate::dataframe::DataField;
use crate::quantum_vector_state::QuantumVectorState;

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
		assert!(num_qubits < 32); // TODO allow for larger random PauliStrings
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
		if (self.x(i) == other.x(i)) && (self.z(i) == other.z(i)) { // ops are equal
			true
		} else if !self.x(i) && !self.z(i) { // self is identity
			true
		} else if !other.x(i) && !other.z(i) { // other is identity
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
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
struct Tableau {
	rows: Vec<PauliString>,

	track_destabilizers: bool,

	pub print_ops: bool,
}

impl Tableau {
	pub fn new(num_qubits: usize) -> Self {
		let mut rows: Vec<PauliString> = vec![PauliString::new(num_qubits); 2*num_qubits + 1]; 

		for i in 0..num_qubits {
			rows[i].set_x(i, true);
			rows[i + num_qubits].set_z(i, true);
		}
		
		Tableau { rows: rows, track_destabilizers: true, print_ops: true}
	}

	fn num_rows(&self) -> usize {
		// If tracking destabilizers, need an extra 'scratch row' which should not affect unitary operations
		if self.track_destabilizers { self.rows.len() - 1 } else { self.rows.len() }
	}

	pub fn print(&self) -> String {
		let mut s: String = String::new();
		for i in 0..self.num_rows() {
			s.push_str(if i == 0 { "[" } else { " " });
			s.push_str(&self.rows[i].to_string(self.print_ops));
			s.push_str(if i == 2*self.rows.len() - 1 { "]" } else { "\n" });
		}
		
		s
	}

	fn x(&self, i: usize, j: usize) -> bool {
		self.rows[i].x(j)
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
			//return (z2 as i32) - (x2 as i32);
			if z2 { return if x2 { 0 } else { 1 } }
			else { return if x2 { -1 } else { 0 } }
		} else if x1 && !z1 { // z2 * (2*x2 - 1)
			//return (z2 as i32) * (2*(x2 as i32) - 1);
			if z2 { return if x2 { 1 } else { -1 } }
			else { return 0 }
		} else { // x2 * (1 - 2*z2) 
			//return (x2 as i32) * (1 - 2*(z2 as i32));
			if x2 { return if z2 { -1 } else { 1 } }
			else { return 0 }
		}
	}

	pub fn rowsum(&mut self, h: usize, i: usize) {
		assert!(self.track_destabilizers);
		let mut s: i32 = 0;
		if self.r(i) { s += 2 }
		if self.r(h) { s += 2 }

		let num_qubits: usize = self.num_rows()/2;
		for j in 0..num_qubits {
			s += Self::g(self.x(i,j), self.z(i,j), self.x(h,j), self.z(h,j));
		}
		
		if s % 4 == 0 {
			self.set_r(h, false);
		} else if (s % 4).abs() == 2 {
			self.set_r(h, true);
		}

		for j in 0..num_qubits {
			self.set_x(h, j, self.x(i, j) != self.x(h, j));
			self.set_z(h, j, self.z(i, j) != self.z(h, j));
		}
	}

	pub fn h_gate(&mut self, qubit: usize) {
		for i in 0..self.num_rows() {
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
		for i in 0..self.num_rows() {
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
		for i in 0..self.num_rows() {
			let xa = self.x(i, qubit1);
			let za = self.z(i, qubit1);
			let xb = self.x(i, qubit2);
			let zb = self.z(i, qubit2);

			let r = self.r(i);

			// Set r_i
			self.set_r(i, r != ((xa && zb) && ((xb != za) != true)));

			// Set x2
			self.set_x(i, qubit2, xa != xb);

			// Set z1
			self.set_z(i, qubit1, za != zb);
		}
	}

	pub fn mzr_deterministic(&self, qubit: usize) -> (bool, usize) {
		assert!(self.track_destabilizers);
		let num_qubits: usize = self.rows.len()/2;

		for i in num_qubits..2*num_qubits {
			if self.x(i, qubit) {
				return (true, i);
			}
		}

		return (false, 0);
	}

	pub fn mzr_qubit(&mut self, qubit: usize, mzr_outcome: bool) -> i32 {
		// Must be tracking destabilizers to perform measurements
		assert!(self.track_destabilizers);

		let num_qubits: usize = self.rows.len()/2;

		let (found_p, p): (bool, usize) = self.mzr_deterministic(qubit);

		if found_p {
			for i in 0..2*num_qubits {
				if i != p && self.x(i, qubit) {
					self.rowsum(i, p);
				}
			}

			self.rows[p - num_qubits] = self.rows[p].clone();
			self.rows[p] = PauliString::new(num_qubits);

			if mzr_outcome {
				self.set_r(p, true);
			}
			self.set_z(p, qubit, true);

			return mzr_outcome as i32;

		} else {
			self.rows[2*num_qubits] = PauliString::new(num_qubits);
			for i in 0..num_qubits {
				self.rowsum(2*num_qubits, i + num_qubits);
			}

			return self.r(2*num_qubits) as i32;
		}

	}
}

const ZERO : Complex<f32> = Complex::new(0., 0.);
const ONE : Complex<f32> = Complex::new(1., 0.);
const N_ONE : Complex<f32> = Complex::new(-1., 0.);
const I : Complex<f32> = Complex::new(0., 1.);
const N_I : Complex<f32> = Complex::new(0., -1.);
const HALF : Complex<f32> = Complex::new(0.5, 0.);

#[derive(Debug)]
struct Matrix {
	n: usize,
	m: usize,
	data: Vec<Vec<Complex<f32>>>,
}

impl std::ops::Index<usize> for Matrix {
	type Output = Vec<Complex<f32>>;

	fn index<'a>(&'a self, i: usize) -> &'a Vec<Complex<f32>> {
		&self.data[i]
	}
}

impl std::ops::IndexMut<usize> for Matrix {
	fn index_mut<'a>(&'a mut self, i: usize) -> &'a mut Vec<Complex<f32>> {
		&mut self.data[i]
	}
}

impl Matrix {
	pub fn new(n: usize, m: usize) -> Self {
		Matrix { n: n, m: m, data: vec![vec![ZERO; m]; n] }
	}

	pub fn identity(n: usize) -> Self {
		let mut identity = Matrix::new(n, n);
		for i in 0..n {
			identity[i][i] = ONE;
		}
		identity
	}

	pub fn scale(&mut self, f: Complex<f32>) {
		for i in 0..self.n {
			for j in 0..self.m {
				self[i][j] *= f;
			}
		}
	}

	pub fn add(&self, other: &Matrix) -> Self {
		assert!(self.n == other.n && self.m == other.m);
		let mut result: Matrix = Matrix::new(self.n, self.m);
		for i in 0..self.n {
			for j in 0..self.m {
				result[i][j] = self[i][j] + other[i][j];
			}
		}
		return result

	}

	pub fn mul(&self, other: &Matrix) -> Self {
		assert!(self.m == other.n);
		let mut result: Matrix = Matrix::new(self.n, other.m);
		for i in 0..self.n {
			for j in 0..other.m {
				for k in 0..self.m {
					result[i][j] += self[i][k]*other[k][j];
				}
			}
		}

		result
	}

	pub fn kron(&self, other: &Matrix) -> Matrix {
		let mut result: Matrix = Matrix::new(self.n*other.n, self.m*other.m);
		for r in 0..self.n {
			for s in 0..self.m {
				for v in 0..other.n {
					for w in 0..other.m {
						result[other.n*r + v][other.m*s + w] = self[r][s]*other[v][w];
					}
				}
			}
		}
		result
	}
}


#[derive(Serialize, Deserialize, Clone)]
pub struct QuantumCHPState {
	num_qubits: usize,
	tableau: Tableau,

	rng: Lcg64Xsh32,
}

impl QuantumCHPState {
	fn get_pauli(&self, i: usize, j: usize) -> Matrix {
		let mut p: Matrix = Matrix::new(2, 2);
		match self.tableau.rows[i].to_op(j) {
			"I" => { // I
				p[0][0] = ONE;
				p[1][1] = ONE;
			}
			"X" => { // X
				p[0][1] = ONE;
				p[1][0] = ONE;
			}
			"Z" => { // Z
				p[0][0] = ONE;
				p[1][1] = N_ONE;
			}
			"Y" => { // Y
				p[0][1] = N_I;
				p[1][0] = I;
			}
			_ => panic!()
		}

		p
	}

	fn generator(&self, idx: usize) -> Matrix {
		let mut g: Matrix = self.get_pauli(idx + self.num_qubits, 0);
		for i in 1..self.num_qubits {
			g = g.kron(&self.get_pauli(idx + self.num_qubits, i))
		}

		if self.tableau.r(idx + self.num_qubits) {
			g.scale(N_ONE);
		}

		g
	}

	pub fn to_vector_state(&self) -> QuantumVectorState {
		// Very slow; to be used for debugging small (n < 5) circuits
		let identity: Matrix = Matrix::identity(1 << self.num_qubits);
		let mut P: Matrix = Matrix::identity(1 << self.num_qubits);
		for i in 0..self.num_qubits {
			let g: Matrix = self.generator(i);
			P = P.mul(&g.add(&identity));
			P.scale(HALF);
		}

		// P is the projector = |p><p|
		let mut nonzero_basis: Vec<(usize, f32)> = Vec::new();
		for i in 0..(1 << self.num_qubits) {
			if P[i][i].re > 0.00001 || P[i][i].im > 0.00001 {
				nonzero_basis.push((i, (P[i][i].re * P[i][i].re + P[i][i].im*P[i][i].im).sqrt()));
			}
		}

		let mut vector_state: QuantumVectorState = QuantumVectorState::new(self.num_qubits);
		vector_state.state.clear();

		let r0: f32 = nonzero_basis[0].1;
		let norm: f32 = (1./(nonzero_basis.len() as f32)).sqrt();
		for j in 0..nonzero_basis.len() {
			let i: usize = nonzero_basis[j].0;
			let ri: f32 = nonzero_basis[j].1;
			let phase: Complex<f32> = P[i][nonzero_basis[0].0]/(r0*ri);

			let mut reversed_bits: usize = i;
			for k in 0..self.num_qubits/2 {
				// Swap bits
				let p1 = k;
				let p2 = self.num_qubits - 1 - k;
				if (((reversed_bits & (1 << p1)) >> p1) ^ ((reversed_bits & (1 << p2)) >> p2)) != 0 {
					reversed_bits ^= 1 << p1;
					reversed_bits ^= 1 << p2;
				}
			}

			vector_state.add_basis(reversed_bits as u64, ri*phase*norm);
		}

		vector_state
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

	// Generate a random clifford gate on N qubits following https://arxiv.org/pdf/2008.06011.pdf
	fn random_clifford(&mut self, qubits: Vec<usize>) {
		let num_qubits: usize = qubits.len();
		
		// First PauliString is totally random (non-identity)
		let mut pauli1: PauliString = PauliString::rand(num_qubits, &mut self.rng);

		// Second is randomly generated until it anticommutes with the first PauliString
		let mut pauli2: PauliString = {
			let mut anticommutes: bool = false;
			let mut pauli: PauliString = PauliString::rand(num_qubits, &mut self.rng);
			while !anticommutes {
				if !pauli1.commutes(&pauli) {
					anticommutes = true;
					break
				}
				pauli = PauliString::rand(num_qubits, &mut self.rng);
			}
			pauli
		};

		//println!("{}, {}, anticommutes: {}", pauli1.to_string(true), pauli2.to_string(true), !pauli1.commutes(&pauli2));
		let mut tableau: Tableau = Tableau { rows: vec![pauli1, pauli2], track_destabilizers: false, print_ops: true };

		// Step one: clear Z-block of first row
		for i in 0..num_qubits {
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
		let mut nonzero_idxs: Vec<usize> = (0..num_qubits).filter(|i| tableau.x(0, *i))
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
			for i in 0..num_qubits {
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
		let mut positive_Z1: PauliString = PauliString::new(num_qubits);
		positive_Z1.set_z(0, true);

		let mut negative_Z1: PauliString = PauliString::new(num_qubits);
		negative_Z1.set_z(0, true);
		negative_Z1.set_r(true);

		if tableau.rows[1] != positive_Z1 && tableau.rows[1] != negative_Z1 {
			tableau.h_gate(0);
			self.h_gate(qubits[0]);

			// Repeat steps one and two
			for i in 0..num_qubits {
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

			let mut nonzero_idxs: Vec<usize> = (0..num_qubits).filter(|i| tableau.x(1, *i))
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


impl MzrForce for QuantumCHPState {
	fn mzr_qubit_forced(&mut self, qubit: usize, outcome: bool) -> bool {
		let (outcome_random, i): (bool, usize) = self.tableau.mzr_deterministic(qubit);
		if outcome_random {
			self.tableau.mzr_qubit(qubit, outcome);
			return true
		} else {
			return self.mzr_qubit(qubit) == (outcome as i32);
		}
	}
}