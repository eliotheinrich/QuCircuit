use serde::{Serialize, Deserialize};
use crate::brickwall_run::{CircuitType, polarize, timesteps_qa, timesteps_rc};
use crate::quantum_state::{QuantumState, Entropy};
use crate::quantum_chp_state::QuantumCHPState;
use crate::quantum_graph_state::QuantumGraphState;
use crate::quantum_vector_state::QuantumVectorState;
use crate::dataframe::{Sample, DataSlide, Simulator, RunConfig};

const fn _true() -> bool { true }
const fn _false() -> bool { false }
const fn _zero() -> usize { 0 }
const fn _one() -> usize { 1 }
const fn _two() -> usize { 2 }


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EntropyJSONConfig {
    pub run_name: String,

    pub circuit_type: String,

    #[serde(default = "_two")]
    pub gate_width: usize,

    pub simulator_type: String,

    pub system_sizes: Vec<usize>,
    pub partition_sizes: Vec<usize>,
    pub mzr_probs: Vec<f32>,
    pub timesteps: Vec<usize>,

    #[serde(default = "_one")]
    pub num_runs: usize,

    #[serde(default = "_zero")]
    pub equilibration_steps: usize,

    #[serde(default = "_false")]
    pub temporal_avg: bool,
    #[serde(default = "_zero")]
    pub measurement_freq: usize,

    #[serde(default = "_false")]
    pub space_avg: bool,
    #[serde(default = "_one")]
    pub spacing: usize,

    #[serde(default = "_true")]
    pub save_data: bool, 

    pub filename: String
}

impl EntropyJSONConfig {
    pub fn load_json(cfg_filename: &String) -> Self {
        let data = std::fs::read_to_string(cfg_filename).unwrap();
        let cfg: EntropyJSONConfig = serde_json::from_str(&data).unwrap();
        
		cfg
    }

	pub fn to_configs(&self) -> Vec<EntropyConfig> {
		let mut configs: Vec<EntropyConfig> = Vec::new();
		for system_size_idx in 0..self.system_sizes.len() {
			for timesteps_idx in 0..self.timesteps.len() {
				for partition_size_idx in 0..self.partition_sizes.len() {
					for mzr_prob_idx in 0..self.mzr_probs.len() {
						configs.push(EntropyConfig::from(&self, system_size_idx, 
																	timesteps_idx, 
																	partition_size_idx, 
																	mzr_prob_idx));
					}
				}
			}
		}

		println!("Producing {} configs", configs.len());
		
		configs
	}

    pub fn print(&self) {
        println!("{:?}", self);
    }
}

pub struct EntropyConfig {
    circuit_type: CircuitType,
    gate_width: usize,
    simulator_type: String,

    system_size: usize,
    partition_size: usize,
    mzr_prob: f32,
    timesteps: usize,

    num_runs: usize,

    equilibration_steps: usize,

    temporal_avg: bool,
    measurement_freq: usize,
    
    space_avg: bool,
    spacing: usize,
}

impl EntropyConfig {
    pub fn from(json_config: &EntropyJSONConfig, system_size_idx: usize, timesteps_idx: usize, 
                                                 partition_size_idx: usize, mzr_idx: usize) -> Self {
        assert!(json_config.system_sizes[system_size_idx] >= json_config.partition_sizes[partition_size_idx]);
        assert!(json_config.mzr_probs[mzr_idx] >= 0. && json_config.mzr_probs[mzr_idx] <= 1.);
        EntropyConfig{
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

            system_size: json_config.system_sizes[system_size_idx],
            partition_size: json_config.partition_sizes[partition_size_idx],
            mzr_prob: json_config.mzr_probs[mzr_idx],
            timesteps: json_config.timesteps[timesteps_idx],

            num_runs: json_config.num_runs,

            equilibration_steps: json_config.equilibration_steps,

            temporal_avg: json_config.temporal_avg,
            measurement_freq: json_config.measurement_freq,

            space_avg: json_config.space_avg,
            spacing: json_config.spacing,
        }
    }

    fn compute_entropy<Q: QuantumState + Entropy>(&self, quantum_state: &mut Q) -> Vec<Sample> {
        let system_size = quantum_state.system_size();
        let qubits: Vec<usize> = (0..self.partition_size).collect();
        let mut entropy: Vec<Sample> = Vec::new();

        // Intially polarize in x-direction
        match self.circuit_type {
            CircuitType::QuantumAutomaton => {
                polarize(quantum_state);
                timesteps_qa(quantum_state, self.equilibration_steps, self.mzr_prob);
            },
            CircuitType::RandomClifford => {
                timesteps_rc(quantum_state, self.equilibration_steps, self.mzr_prob, self.gate_width, false);
            },
        }


        // Do timesteps
        let (num_timesteps, num_intervals): (usize, usize) = if self.timesteps == 0 { 
            (0, 1)
        } else { 
            (self.measurement_freq, self.timesteps/self.measurement_freq)
        };
        
        for t in 0..num_intervals {
            match self.circuit_type {
                CircuitType::QuantumAutomaton => timesteps_qa(quantum_state, num_timesteps, self.mzr_prob),
                CircuitType::RandomClifford => timesteps_rc(quantum_state, num_timesteps, self.mzr_prob, self.gate_width, t*num_timesteps % 2 == 0),
            }

            let sample: Sample = 
            if self.space_avg {
                let num_partitions = std::cmp::max((self.system_size - self.partition_size)/self.spacing, 1);

                let mut s: f32 = 0.;
                let mut s2: f32 = 0.;

                for i in 0..num_partitions {
                    let offset_qubits: Vec<usize> = qubits.iter().map(|x| x + i*self.spacing).collect();
                    let tmp: f32 = quantum_state.renyi_entropy(&offset_qubits);
                    s += tmp;
                    s2 += tmp.powi(2);
                }

                s /= num_partitions as f32;
                s2 /= num_partitions as f32;
                let std: f32 = (s2 - s.powi(2)).powf(0.5);
                
                Sample { mean: s, std: std, num_samples: num_partitions }
            } else {
                Sample::new(quantum_state.renyi_entropy(&qubits))
            };

            entropy.push(sample);
        }

        return entropy;
    }

    fn save_to_dataslide<Q: QuantumState + Entropy + Clone>(&self, dataslide: &mut DataSlide, quantum_state: &mut Q) {
        let init_quantum_state = quantum_state.clone();

        let mut entropy: Vec<Sample> = self.compute_entropy(quantum_state);

        for run in 0..(self.num_runs - 1) {
            *quantum_state = init_quantum_state.clone(); // Reinitialize state
            entropy = entropy.iter()
                             .zip(self.compute_entropy(quantum_state).iter())
                             .map(|(a, b)| a.combine(b))
                             .collect();
        }

        if self.temporal_avg {
            let sample: Sample = entropy.iter().fold(Sample { mean: 0., std: 0., num_samples: 0 }, |sum, val| sum.combine(val));
            entropy = vec![sample];
        }

        for s in entropy {
            dataslide.push_data("entropy", s);
        }
    }

}

impl RunConfig for EntropyConfig {
    fn init_state(&self) -> Simulator {
        return match self.simulator_type.as_str() {
            "chp" => Simulator::CHP(QuantumCHPState::new(self.system_size)),
            "graph" => Simulator::Graph(QuantumGraphState::new(self.system_size)),
            "vector" => Simulator::Vector(QuantumVectorState::new(self.system_size)),
            _ => {
                println!("Error: simulator type provided not supported.");
                panic!()
            }
        }       
    }

    fn gen_dataslide(&self, mut simulator: Simulator) -> DataSlide {
        assert!(self.num_runs > 0);
        let mut dataslide: DataSlide = DataSlide::new();

        // Parameters
        dataslide.add_int_param("system_size", self.system_size as i32);
        dataslide.add_int_param("timesteps", self.timesteps as i32);
        dataslide.add_int_param("partition_size", self.partition_size as i32);
        dataslide.add_float_param("mzr_prob", self.mzr_prob);

        dataslide.add_data("entropy");
        
        match simulator {
            Simulator::CHP(mut state) => self.save_to_dataslide(&mut dataslide, &mut state),
            Simulator::Graph(mut state) => self.save_to_dataslide(&mut dataslide, &mut state),
            Simulator::Vector(mut state) => self.save_to_dataslide(&mut dataslide, &mut state),
            Simulator::None => {
                println!("State not initialized!");
                panic!();
            }
        };

        return dataslide;
            
    }
}