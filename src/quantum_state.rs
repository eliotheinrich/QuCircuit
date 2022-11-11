use std::collections::HashMap;

#[derive(Clone, Copy)]
enum Gate {
    I,
    X,
    Y,
    Z,
    H,
    CX,
    CY,
    CZ,
    MXR,
    MYR,
    MZR,
    S,
    Sd,
    SQRTX,
    SQRTXd,
    SQRTY,
    SQRTYd,
    SQRTZ,
    SQRTZd,
    
    PRINT,
}


pub trait QuantumState {
    // At minimum, a QuantumState must implement the S-gate, the H-gate, 
    // the CZ-gate, and measurements in the computational basis
    fn new(num_qubits: usize) -> Self;
    fn print(&self) -> String;

    fn system_size(&self) -> usize;

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

    fn sqrtx_gate(&mut self, qubit: usize) {
        self.sd_gate(qubit);
        self.h_gate(qubit);
        self.sd_gate(qubit);
    }
    fn sqrtxd_gate(&mut self, qubit: usize) {
        self.s_gate(qubit);
        self.h_gate(qubit);
        self.s_gate(qubit);
    }
    fn sqrty_gate(&mut self, qubit: usize) {
        self.z_gate(qubit);
        self.h_gate(qubit);
    }
    fn sqrtyd_gate(&mut self, qubit: usize) {
        self.h_gate(qubit);
        self.z_gate(qubit);
    }
    fn sqrtz_gate(&mut self, qubit: usize) {
        self.s_gate(qubit);
    }
    fn sqrtzd_gate(&mut self, qubit: usize) {
        self.sd_gate(qubit);
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
    id: Gate,
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

    fn init_mapped_gates() -> HashMap<String, (Gate, usize, usize)> {
        let mut gates: HashMap<String, (Gate, usize, usize)> = HashMap::new();
        gates.insert(String::from("x"), (Gate::X, 1, 0)); gates.insert(String::from("X"), (Gate::X, 1, 0));
        gates.insert(String::from("y"), (Gate::Y, 1, 0)); gates.insert(String::from("Y"), (Gate::Y, 1, 0));
        gates.insert(String::from("z"), (Gate::Z, 1, 0)); gates.insert(String::from("Z"), (Gate::Z, 1, 0));

        gates.insert(String::from("h"), (Gate::H, 1, 0)); gates.insert(String::from("H"), (Gate::H, 1, 0));
        
        gates.insert(String::from("s"), (Gate::S, 1, 0)); gates.insert(String::from("S"), (Gate::S, 1, 0));
        gates.insert(String::from("sd"), (Gate::Sd, 1, 0)); gates.insert(String::from("SD"), (Gate::Sd, 1, 0));

        gates.insert(String::from("sqrtx"),  (Gate::SQRTX, 1, 0)); gates.insert(String::from("SQRTX"), (Gate::SQRTX, 1, 0));
        gates.insert(String::from("sqrtxd"), (Gate::SQRTXd, 1, 0)); gates.insert(String::from("SQRTXD"), (Gate::SQRTXd, 1, 0));
        gates.insert(String::from("sqrty"),  (Gate::SQRTY, 1, 0)); gates.insert(String::from("SQRTY"), (Gate::SQRTY, 1, 0));
        gates.insert(String::from("sqrtyd"), (Gate::SQRTYd, 1, 0)); gates.insert(String::from("SQRTYD"), (Gate::SQRTYd, 1, 0));
        gates.insert(String::from("sqrtz"),  (Gate::S, 1, 0)); gates.insert(String::from("SQRTZ"), (Gate::S, 1, 0));
        gates.insert(String::from("sqrtzd"), (Gate::Sd, 1, 0)); gates.insert(String::from("SQRTZD"), (Gate::Sd, 1, 0));

        gates.insert(String::from("cx"), (Gate::CX, 2, 0)); gates.insert(String::from("CX"), (Gate::CX, 2, 0));
        gates.insert(String::from("cnot"), (Gate::CX, 2, 0)); gates.insert(String::from("CNOT"), (Gate::CX, 2, 0));
        gates.insert(String::from("cy"), (Gate::CY, 2, 0)); gates.insert(String::from("CY"), (Gate::CY, 2, 0));
        gates.insert(String::from("cz"), (Gate::CZ, 2, 0)); gates.insert(String::from("CZ"), (Gate::CZ, 2, 0));


        gates.insert(String::from("mxr"), (Gate::MXR, 1, 1)); gates.insert(String::from("MXR"), (Gate::MXR, 1, 1));
        gates.insert(String::from("myr"), (Gate::MYR, 1, 1)); gates.insert(String::from("MYR"), (Gate::MYR, 1, 1));
        gates.insert(String::from("mzr"), (Gate::MZR, 1, 1)); gates.insert(String::from("MZR"), (Gate::MZR, 1, 1));


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
        let mut id: Gate = Gate::I;
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
                    id = Gate::PRINT;
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
                Gate::I => (),
                Gate::X => self.quantum_state.x_gate(inst.qubits[0]), 
                Gate::Y => self.quantum_state.y_gate(inst.qubits[0]),
                Gate::Z => self.quantum_state.z_gate(inst.qubits[0]),
                Gate::H => self.quantum_state.h_gate(inst.qubits[0]),
                Gate::SQRTX => self.quantum_state.sqrtx_gate(inst.qubits[0]),
                Gate::SQRTXd => self.quantum_state.sqrtxd_gate(inst.qubits[0]),
                Gate::SQRTY => self.quantum_state.sqrty_gate(inst.qubits[0]),
                Gate::SQRTYd => self.quantum_state.sqrtyd_gate(inst.qubits[0]),
                Gate::SQRTZ => self.quantum_state.sqrtz_gate(inst.qubits[0]),
                Gate::SQRTZd => self.quantum_state.sqrtzd_gate(inst.qubits[0]),
                Gate::CX => self.quantum_state.cx_gate(inst.qubits[0], inst.qubits[1]),
                Gate::CY => self.quantum_state.cy_gate(inst.qubits[0], inst.qubits[1]),
                Gate::CZ => self.quantum_state.cz_gate(inst.qubits[0], inst.qubits[1]),
                Gate::MXR => self.classical_data[inst.cbits[0]] = self.quantum_state.mxr_qubit(inst.qubits[0]),
                Gate::MYR => self.classical_data[inst.cbits[0]] = self.quantum_state.myr_qubit(inst.qubits[0]),
                Gate::MZR => self.classical_data[inst.cbits[0]] = self.quantum_state.mzr_qubit(inst.qubits[0]),
                Gate::S => self.quantum_state.s_gate(inst.qubits[0]),
                Gate::Sd => self.quantum_state.sd_gate(inst.qubits[0]),
                Gate::PRINT => println!("{}", self.quantum_state.print()),
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
