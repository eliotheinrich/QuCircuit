use serde::{Serialize, Deserialize};
use crate::brickwall_run::{CircuitType, polarize, timesteps_qa, timesteps_rc};
use crate::quantum_state::{QuantumState, Entropy};
use crate::quantum_chp_state::QuantumCHPState;
use crate::quantum_graph_state::QuantumGraphState;
use crate::quantum_vector_state::QuantumVectorState;
use crate::dataframe::{Sample, DataSlide, Simulator, RunConfig};

const fn _two() -> usize { 2 }

#[derive(Serialize, Deserialize, Debug)]
pub struct StateJSONConfig {
	pub run_name: String,
    pub circuit_type: String,

	#[serde(default = "_two")]
    pub gate_width: usize,

    pub simulator_type: String,

    pub system_size: usize,
    pub mzr_probs: Vec<f32>,

    pub num_runs: usize,
    pub equilibration_steps: usize,

	pub filename: String,
}

impl StateJSONConfig {
    pub fn load_json(cfg_filename: &String) -> Self {
        let data = std::fs::read_to_string(cfg_filename).unwrap();
        let cfg: StateJSONConfig = serde_json::from_str(&data).unwrap();
        
		cfg
    }

	pub fn to_configs(&self) -> Vec<StateConfig> {
		let mut configs: Vec<StateConfig> = Vec::new();
		for mzr_prob_idx in 0..self.mzr_probs.len() {
			configs.push(StateConfig::from(&self, mzr_prob_idx));
		}

		println!("Producing {} configs", configs.len());
		
		configs
	}

    pub fn print(&self) {
        println!("{:?}", self);
    }

}

pub struct StateConfig {
    circuit_type: CircuitType,
    gate_width: usize,

    simulator_type: String,

    system_size: usize,
    mzr_prob: f32,

    num_runs: usize,
    equilibration_steps: usize,
}

impl StateConfig {
    pub fn from(json_config: &StateJSONConfig, mzr_idx: usize) -> Self {
        assert!(json_config.mzr_probs[mzr_idx] >= 0. && json_config.mzr_probs[mzr_idx] <= 1.);
        StateConfig{
            circuit_type: match json_config.circuit_type.as_str() {
                "default" => CircuitType::QuantumAutomaton,
                "quantum_automaton" => CircuitType::QuantumAutomaton,
                "random_clifford" => CircuitType::RandomClifford,
                _ => {
                    println!("circuit type {} not supported.", json_config.circuit_type);
                    panic!();
                }
            },
            gate_width: json_config.gate_width,
            simulator_type: json_config.simulator_type.clone(),

            system_size: json_config.system_size,
            mzr_prob: json_config.mzr_probs[mzr_idx],

            num_runs: json_config.num_runs,

            equilibration_steps: json_config.equilibration_steps,
        }
    }

    fn evolve_state<Q: QuantumState> (&self, quantum_state: &mut Q) {
        match self.circuit_type {
            CircuitType::QuantumAutomaton => {
				polarize(quantum_state);
				timesteps_qa(quantum_state, self.equilibration_steps, self.mzr_prob);
			},
            CircuitType::RandomClifford => timesteps_rc(quantum_state, self.equilibration_steps, self.mzr_prob, self.gate_width, false),
        }
    }

    fn save_to_dataslide<Q: QuantumState>(&self, key: &str, dataslide: &mut DataSlide, quantum_state: &mut Q) {
        self.evolve_state(quantum_state);

        // Save to dataslide
        let num_qubits: usize = quantum_state.system_size();
        dataslide.add_string_param(key, &quantum_state.print());
    }
}

impl RunConfig for StateConfig {
    fn init_state(&self) -> Simulator {
        match self.simulator_type.as_str() {
            "chp" => Simulator::CHP(QuantumCHPState::new(self.system_size)),
            "graph" => Simulator::Graph(QuantumGraphState::new(self.system_size)),
            _ => {
                println!("Error; simulator type not supported.");
                panic!();
            }
        }
    }

    fn gen_dataslide(&self, mut simulator: Simulator) -> DataSlide {
        assert!(self.num_runs > 0);
        let mut dataslide: DataSlide = DataSlide::new();

        // Parameters
        dataslide.add_int_param("system_size", self.system_size as i32);
        dataslide.add_float_param("mzr_prob", self.mzr_prob);

        for i in 0..self.num_runs {
            let key: &str = &("state".to_owned() + &i.to_string());
            match &simulator {
                Simulator::CHP(state) => self.save_to_dataslide(key, &mut dataslide, &mut state.clone()),
                Simulator::Graph(state) => self.save_to_dataslide(key, &mut dataslide, &mut state.clone()),
                _ => panic!(),
            }
        }

        dataslide
    }
}
