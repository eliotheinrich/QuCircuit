use std::collections::HashMap;

const XGATE: u32  =      0;
const YGATE: u32  =      1;
const ZGATE: u32  =      2;
const HGATE: u32  =      3;
const CXGATE: u32 =      4;
const CYGATE: u32 =      5;
const CZGATE: u32 =      6;
const MXRGATE: u32 =     7;
const MYRGATE: u32 =     8;
const MZRGATE: u32 =     9;
const SGATE: u32 =      10;
const SDGATE: u32 =     11;
const SQRTXGATE: u32 =  12;
const SQRTXDGATE: u32 = 13;
const SQRTYGATE: u32 =  14;
const SQRTYDGATE: u32 = 15;


const PRINT: u32 = 100;

pub trait QuantumState {
    // At minimum, a QuantumState must implement the S-gate, the H-gate, 
    // the CZ-gate, and measurements in the computational basis
    fn new(num_qubits: usize) -> Self;
    fn print(&self) -> String;

    fn x_gate(&mut self, qubit: usize) {
        self.h_gate(qubit);
        self.z_gate(qubit);
        self.h_gate(qubit);
    }
    fn y_gate(&mut self, qubit: usize) {
        self.x_gate(qubit);
        self.z_gate(qubit);
    }
    fn z_gate(&mut self, qubit: usize) {
        self.s_gate(qubit);
        self.s_gate(qubit);
    }

    fn h_gate(&mut self, qubit: usize);

    fn s_gate(&mut self, qubit: usize);
    fn sd_gate(&mut self, qubit: usize) {
        self.s_gate(qubit);
        self.s_gate(qubit);
        self.s_gate(qubit);
    }

    fn cx_gate(&mut self, qubit1: usize, qubit2: usize) {
        self.h_gate(qubit2);
        self.cz_gate(qubit1, qubit2);
        self.h_gate(qubit2);
    }
    fn cy_gate(&mut self, qubit1: usize, qubit2: usize) {
        self.s_gate(qubit2);
        self.h_gate(qubit2);
        self.cz_gate(qubit1, qubit2);
        self.h_gate(qubit2);
        self.sd_gate(qubit2);
    }
    fn cz_gate(&mut self, qubit1: usize, qubit2: usize);

    fn mxr_qubit(&mut self, qubit: usize) -> i32 {
        self.h_gate(qubit);
        let measured = self.mzr_qubit(qubit);
        self.h_gate(qubit);
        return measured;
    }
    fn myr_qubit(&mut self, qubit: usize) -> i32 {
        self.s_gate(qubit);
        self.h_gate(qubit);
        let measured = self.mzr_qubit(qubit);
        self.h_gate(qubit);
        self.sd_gate(qubit);
        return measured;
    }
    fn mzr_qubit(&mut self, qubit: usize) -> i32;

    fn finish_execution(&mut self) {}
}

pub trait Entropy {
    fn renyi_entropy(&self, qubits: &Vec<usize>) -> f32;
}

pub struct Instruction {
    id: u32,
    qubits: Vec<usize>,
    cbits: Vec<usize>
}

pub struct QuantumProgram<Q: QuantumState> {
    total_num_qubits: usize,
    total_num_cbits: usize,
    classical_data: Vec<i32>,
    pub quantum_state: Q, // TODO private
    circuit: Vec<Instruction>,
}

fn parse_register(s: &str) -> usize {
    return s[1..s.len()].parse::<usize>().unwrap();
}

impl<Q: QuantumState> QuantumProgram<Q> {
    pub fn print(&self) -> String {
        return self.quantum_state.print() + &format!("\nClassical data: {:?}", self.classical_data);
    }

    fn init_mapped_gates() -> HashMap<String, (u32, usize, usize)> {
        let mut gates: HashMap<String, (u32, usize, usize)> = HashMap::new();
        gates.insert(String::from("x"), (XGATE, 1, 0)); gates.insert(String::from("X"), (XGATE, 1, 0));
        gates.insert(String::from("y"), (YGATE, 1, 0)); gates.insert(String::from("Y"), (YGATE, 1, 0));
        gates.insert(String::from("z"), (ZGATE, 1, 0)); gates.insert(String::from("Z"), (ZGATE, 1, 0));

        gates.insert(String::from("h"), (HGATE, 1, 0)); gates.insert(String::from("H"), (HGATE, 1, 0));
        
        gates.insert(String::from("s"), (SGATE, 1, 0)); gates.insert(String::from("S"), (SGATE, 1, 0));
        gates.insert(String::from("sd"), (SDGATE, 1, 0)); gates.insert(String::from("SD"), (SDGATE, 1, 0));

        gates.insert(String::from("sqrtx"), (SQRTXGATE, 1, 0)); gates.insert(String::from("SQRTX"), (SQRTXGATE, 1, 0));
        gates.insert(String::from("sqrtxd"), (SQRTXDGATE, 1, 0)); gates.insert(String::from("SQRTXD"), (SQRTXDGATE, 1, 0));
        gates.insert(String::from("sqrty"), (SQRTYGATE, 1, 0)); gates.insert(String::from("SQRTY"), (SQRTYGATE, 1, 0));
        gates.insert(String::from("sqrtyd"), (SQRTYDGATE, 1, 0)); gates.insert(String::from("SQRTYD"), (SQRTYDGATE, 1, 0));
        gates.insert(String::from("sqrtz"), (SGATE, 1, 0)); gates.insert(String::from("SQRTZ"), (SGATE, 1, 0));
        gates.insert(String::from("sqrtzd"), (SDGATE, 1, 0)); gates.insert(String::from("SQRTZD"), (SDGATE, 1, 0));

        gates.insert(String::from("cx"), (CXGATE, 2, 0)); gates.insert(String::from("CX"), (CXGATE, 2, 0));
        gates.insert(String::from("cnot"), (CXGATE, 2, 0)); gates.insert(String::from("CNOT"), (CXGATE, 2, 0));
        gates.insert(String::from("cy"), (CYGATE, 2, 0)); gates.insert(String::from("CY"), (CYGATE, 2, 0));
        gates.insert(String::from("cz"), (CZGATE, 2, 0)); gates.insert(String::from("CZ"), (CZGATE, 2, 0));


        gates.insert(String::from("mxr"), (MXRGATE, 1, 1)); gates.insert(String::from("MXR"), (MXRGATE, 1, 1));
        gates.insert(String::from("myr"), (MYRGATE, 1, 1)); gates.insert(String::from("MYR"), (MYRGATE, 1, 1));
        gates.insert(String::from("mzr"), (MZRGATE, 1, 1)); gates.insert(String::from("MZR"), (MZRGATE, 1, 1));


        return gates;
    }

    pub fn from_qasm(circuit: &String) -> QuantumProgram<Q> {
        let lines = circuit.split("\n").collect::<Vec<&str>>();
        let mut line_data: Vec<&str> = Vec::new();

        let mut total_num_qubits: usize = 0;
        let mut total_num_cbits: usize = 0;
        let mut num_qubits: usize = 0;
        let mut num_cbits: usize = 0;
        let mut qubits: Vec<usize> = Vec::new();
        let mut cbits: Vec<usize> = Vec::new();
        let mut gate: &str = "";
        let mut id: u32 = 0;
        let mut instructions: Vec<Instruction> = Vec::new();

        let gates = QuantumProgram::<Q>::init_mapped_gates();
        for line in lines {
            qubits.clear();
            cbits.clear();

            line_data = line.split_whitespace().collect::<Vec<&str>>();
            if line_data.len() == 0 {
                continue
            } else if line_data[0] == "@pragma" {
                if line_data[1] == "total_num_qbits" || line_data[1] == "total_num_qubits" {
                    total_num_qubits = line_data[2].parse::<usize>().unwrap();
                    continue;
                } else if line_data[1] == "total_num_cbits" || line_data[1] == "total_num_bits" {
                    total_num_cbits = line_data[2].parse::<usize>().unwrap();
                    continue;
                } else if line_data[1] == "print" {
                    id = PRINT;
                }
            } else {
                gate = line_data[0];
                if gates.contains_key(gate) {
                    id = gates[gate].0;
                    num_qubits = gates[gate].1;
                    num_cbits = gates[gate].2;

                    assert!(line_data.len() == (num_qubits + num_cbits + 1) as usize);

                    for i in 1..(num_qubits+1) as usize {
                        assert!(String::from(line_data[i]).chars().nth(0).unwrap() == 'q');
                        qubits.push(parse_register(line_data[i]));
                    }
                    for i in (1 + num_qubits as usize)..((1 + num_qubits + num_cbits) as usize) {
                        assert!(String::from(line_data[i]).chars().nth(0).unwrap() == 'r' || String::from(line_data[i]).chars().nth(0).unwrap() == 'c');
                        cbits.push(parse_register(line_data[i]));
                    }
                    

                }

            }
            instructions.push( Instruction { id: id, qubits: qubits.clone(), cbits: cbits.clone() })
        }

        let classical_data:Vec<i32> = vec![-1; total_num_cbits as usize];
        let state = Q::new(total_num_qubits);
        return QuantumProgram { total_num_qubits: total_num_qubits, total_num_cbits: total_num_cbits, 
                                classical_data: classical_data, quantum_state: state,
                                circuit: instructions };
    }

    pub fn execute(&mut self) {
        for inst in &self.circuit {
            match inst.id {
                XGATE => self.quantum_state.x_gate(inst.qubits[0]), 
                YGATE => self.quantum_state.y_gate(inst.qubits[0]),
                ZGATE => self.quantum_state.z_gate(inst.qubits[0]),
                HGATE => self.quantum_state.h_gate(inst.qubits[0]),
                CXGATE => self.quantum_state.cx_gate(inst.qubits[0], inst.qubits[1]),
                CYGATE => self.quantum_state.cy_gate(inst.qubits[0], inst.qubits[1]),
                CZGATE => self.quantum_state.cz_gate(inst.qubits[0], inst.qubits[1]),
                MXRGATE => self.classical_data[inst.cbits[0]] = self.quantum_state.mxr_qubit(inst.qubits[0]),
                MYRGATE => self.classical_data[inst.cbits[0]] = self.quantum_state.myr_qubit(inst.qubits[0]),
                MZRGATE => self.classical_data[inst.cbits[0]] = self.quantum_state.mzr_qubit(inst.qubits[0]),
                SGATE => self.quantum_state.s_gate(inst.qubits[0]),
                SDGATE => self.quantum_state.sd_gate(inst.qubits[0]),
                PRINT => println!("{}", self.quantum_state.print()),
                _ => { println!("Unknown gate: {}", inst.id); 
                       panic!() },
            };
        }

        self.quantum_state.finish_execution();
    }
    
    pub fn get_classical_data(&self) -> Vec<i32> {
        return self.classical_data.clone();
    }

    pub fn get_classical_register(&self, register: usize) -> i32 {
        assert!(register < self.total_num_cbits);
        return self.classical_data[register];
    }
}
