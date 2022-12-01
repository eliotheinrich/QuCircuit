use std::collections::HashMap;
use num::complex::Complex;
use rand_pcg::Lcg64Xsh32;
use rand::RngCore;
use serde::{Serialize, Deserialize};

use crate::quantum_state::{Entropy, QuantumState};
use crate::dataframe::DataField;

use std::f32::consts::SQRT_2;
const ZERO : Complex<f32> = Complex::new(0., 0.);
const ONE : Complex<f32> = Complex::new(1., 0.);
const I : Complex<f32> = Complex::new(0., 1.);
const _SQRT2 : Complex<f32> = Complex::new(1./SQRT_2, 0.);
const N_SQRT2 : Complex<f32> = Complex::new(-1./SQRT_2, 0.);
const EPS : f32 = 1e-6;

const HGATE: [Complex<f32>; 4] = [_SQRT2, _SQRT2, _SQRT2, N_SQRT2]; 


#[derive(Serialize, Deserialize, Clone)]
pub struct BasisState {
    bits : u64,
    amp : Complex<f32>
}

impl BasisState {
    fn qubit_val(&self, qubit : usize) -> u64 {
        return self.bits >> qubit & 1;
    }
}

fn approx_equal(f1: Complex<f32>, f2: Complex<f32>) -> bool {
    return ((f1.re - f2.re).abs()/f1.norm()) < EPS && ((f1.im - f2.im).abs()/f1.norm()) < EPS;
}

impl std::cmp::PartialEq for BasisState {
    fn eq(&self, other: &BasisState) -> bool {
        return self.bits == other.bits && approx_equal(self.amp, other.amp);
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct QuantumVectorState {
    pub num_qubits : usize,
    pub state : Vec<BasisState>,

    rng : Lcg64Xsh32,
}

impl QuantumVectorState {
    fn arbitrary_gate(&mut self, qubit: usize, gate: [Complex<f32>; 4]) {
        let bitshift: u64 = 1 << qubit;
        let mut new_state_map: HashMap<u64, Complex<f32>> = HashMap::new();
        for b in &self.state {
            let qubit_val = b.qubit_val(qubit);

            new_state_map.entry(b.bits).or_insert(ZERO);
            new_state_map.insert(b.bits, new_state_map[&b.bits] + b.amp*(if qubit_val == 0 { gate[0] } else { gate[3] }));

            new_state_map.entry(b.bits ^ bitshift).or_insert(ZERO);
            new_state_map.insert(b.bits ^ bitshift, new_state_map[&(b.bits ^ bitshift)] + b.amp*(if qubit_val == 0 { gate[2] } else { gate[1] }));
        }

        let mut new_state = Vec::new();
        for (key, val) in &new_state_map {
            if val.norm() > EPS {
                new_state.push(BasisState { bits: *key, amp: *val} );
            }
        }

        self.state = new_state;

    }

    fn fix_phase(&mut self) {
        let mut min_basis: u64 = u64::MAX;
        let mut phase: Complex<f32> = ZERO;

        // We fix global phase with respect to smallest basis vector which appears
        for b in self.state.iter() {
            if b.bits < min_basis {
                min_basis = b.bits;
                phase = b.amp / b.amp.norm();
            }
        }

        for i in 0..self.state.len() {
            self.state[i].amp /= phase;
        }
    }

    fn sort_basis(&mut self) {
        self.state.sort_by(|b1, b2| b1.bits.cmp(&b2.bits));
    }

    // Returns the partial density matrix for the specified qubits
    pub fn reduced_state(&self, qubits: &Vec<usize>) -> Vec<Vec<Complex<f32>>> {
        let mut rho: Vec<Vec<Complex<f32>>> = vec![vec![ZERO; 1 << qubits.len()]; 1 << qubits.len()];
        let mut idx: usize; 
        let mut jdx: usize;

        let mut mask: u64 = !0;
        for i in 0..qubits.len() {
            mask ^= 1 << qubits[i];
        }

        for b1 in 0..self.state.len() {
            for b2 in 0..self.state.len() {
                idx = 0;
                jdx = 0;
                if (self.state[b1].bits & mask) == (self.state[b2].bits & mask) {
                    for j in 0..qubits.len() {
                        idx += (self.state[b1].qubit_val(qubits[j]) as usize) * (1 << j);
                        jdx += (self.state[b2].qubit_val(qubits[j]) as usize) * (1 << j);
                    }
                    //println!("adding {:b} and {:b} to rho at {} {}", self.state[b1].bits, self.state[b2].bits, idx, jdx);
					rho[idx][jdx] += self.state[b1].amp*self.state[b2].amp.conj()
                }
            }
        }

        //println!("{:?}", rho);
        return rho;
    }
}

impl Entropy for QuantumVectorState {
    fn renyi_entropy(&self, qubits: &Vec<usize>) -> f32 {
        let rhoA: Vec<Vec<Complex<f32>>> = self.reduced_state(qubits);
        let mut rhoA2: Vec<Vec<Complex<f32>>> = vec![vec![ZERO; rhoA.len()]; rhoA.len()];
        for i in 0..rhoA.len() {
            for j in 0..rhoA.len() {
                for k in 0..rhoA.len() {
                    rhoA2[i][j] += rhoA[i][k]*rhoA[k][j];
                }
            }
        }

		let mut s: f32 = 0.;
        for i in 0..rhoA.len() {
            s += rhoA2[i][i].re;
        }

        return -s.log2();
    }
}

impl std::cmp::PartialEq for QuantumVectorState {
    fn eq(&self, other: &QuantumVectorState) -> bool {
        if self.num_qubits != other.num_qubits {
            return false;
        }

        if self.state.len() != other.state.len() {
            return false;
        }

        for i in 0..self.state.len() {
            if self.state[i] != other.state[i] { return false }
        }

        return true;
    }
}


impl QuantumState for QuantumVectorState {
    fn new(num_qubits: usize) -> QuantumVectorState {
        let mut s : Vec<BasisState> = Vec::new();
        s.push(BasisState { bits: 0, amp: ONE });
        let rng = Lcg64Xsh32::new(10, 10);
        return QuantumVectorState { num_qubits: num_qubits, state: s, rng: rng };
    }

    fn print(&self) -> String {
        let mut s : String = String::from("\n");

        for b in self.state.iter() {
            s += &format!("{:0width$b}: {:.2}\n", b.bits as usize, b.amp, width = self.num_qubits);
        }
        s = s[0..s.len()-1].to_string();
        return s
    }

    fn system_size(&self) -> usize {
        return self.num_qubits;
    }

    fn x_gate(&mut self, qubit: usize) {
        assert!(qubit < self.num_qubits);
        let bitshift : u64 = 1 << qubit;
        for b in &mut self.state {
            b.bits ^= bitshift;
        }
    }

    fn y_gate(&mut self, qubit: usize) {
        assert!(qubit < self.num_qubits);
        let bitshift = 1 << qubit;
        for b in &mut self.state {
            b.amp *= if b.qubit_val(qubit) == 0 { I } else { -I };
            b.bits ^= bitshift;
        } 
    }

    fn z_gate(&mut self, qubit: usize) {
        assert!(qubit < self.num_qubits);
        for b in &mut self.state {
            b.amp *= if b.qubit_val(qubit) == 1 { -ONE } else { ONE }; 
        }
    }

    fn h_gate(&mut self, qubit: usize) {
        assert!(qubit < self.num_qubits);
        self.arbitrary_gate(qubit, HGATE);
    }

    fn s_gate(&mut self, qubit: usize) {
        assert!(qubit < self.num_qubits);
        for b in &mut self.state {
            b.amp *= if b.qubit_val(qubit) == 0 { ONE } else { I }; 
        }
    }
    
    fn cx_gate(&mut self, qubit1: usize, qubit2: usize) { 
        let bitshift: u64 = 1 << qubit2;
        for b in &mut self.state {
            if b.qubit_val(qubit1) == 1 {
                b.bits ^= bitshift;
            }
        }
    }

    fn cy_gate(&mut self, qubit1: usize, qubit2: usize) { 
        let bitshift: u64 = 1 << qubit2;
        for b in &mut self.state {
            if b.qubit_val(qubit1) == 1 {
                b.bits ^= bitshift;
                b.amp *= if b.qubit_val(qubit2) == 0 { I } else { -I };
            }
        }
    }

    fn cz_gate(&mut self, qubit1: usize, qubit2: usize) { 
        for b in &mut self.state {
            if b.qubit_val(qubit1) == 1 {
                b.amp *= if b.qubit_val(qubit2) == 0 { ONE } else { -ONE };
            }
        } 
    }

    fn mzr_qubit(&mut self, qubit: usize) -> i32 {
        let mut prob_zero: f32 = 0.;
        let mut prob_one: f32 = 0.;
        for b in &self.state {
            match b.qubit_val(qubit) {
                0 => prob_zero += b.amp.norm().powi(2),
                1 => prob_one += b.amp.norm().powi(2),
                _ => panic!()
            };
        }

        let p: f32 = ((self.rng.next_u32() as f64) / (u32::MAX as f64)) as f32;
        let measured: i32 = if p < prob_zero { 0 } else { 1 };
        let norm = if p < prob_zero { prob_zero.sqrt() } else { prob_one.sqrt() };

        for b in &mut self.state {
            if b.qubit_val(qubit) == measured as u64 { b.amp /= norm; } else { b.amp = ZERO; }
        }

        self.state.retain(|b| b.amp.norm() > EPS);

        return measured;
    }

    fn mxr_qubit(&mut self, qubit: usize) -> i32 {
        self.h_gate(qubit);
        let bit = self.mzr_qubit(qubit);
        self.h_gate(qubit);
        return bit;
    }

    fn finish_execution(&mut self) {
        self.fix_phase();
        self.sort_basis();
    }
}