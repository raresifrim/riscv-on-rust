use ahash::AHashMap;

use crate::risc_soc::wire::Wire;

/// Logic for Common Data Bus shared by Pipeline stages to forward data directly between them
/// It should simulate the behaviour of a wire assignment in Verilog

/*

/// A later pipeline stage must be able to forward data to any ealrier stage
/// Thus we define the `CommonDataBus`, which holds a collection of "buses" for each pipeline stage 
/// It represents a mapping between that pipeline stage and an array of `Wire` where each `Wire` has as destination any earlier stage before the current one
/// Each `Wire` on its own rewpresent a collection of bits so that multiple data can be passed from a stage to another
/// In rust we represent the `Wire` as a vector of bytes, and each `DataLane` is a vector of `Wire`
/// The final `CommonDataBus` is a collection of `DataLane`

  IF       ID        EX        MEM
       _         _         _     
      | |       | |       | |
      | |       | |       | |   
COMB  | | COMB  | | COMB  | | COMB
LOGIC | | LOGIC | | LOGIC | | LOGIC
  /\  | |   /\  | |   /\  | |   |
  |   |_|   |   |_|   |   |_|   | 
  |Wire     |         ----------| <-
  |(bits)   --------------------|   | -> DataLane
  ------------------------------| <-

                                */ 



type DataLanes = Vec<Wire>;
type StageIndex = usize;
pub struct CommonDataBus {
   pub bus: AHashMap<StageIndex, DataLanes>
}

impl CommonDataBus {
    pub fn new(num_stages: usize, critical_path: Option<u128>, debug: bool) -> Self {
        let mut bus = AHashMap::new();
        for i in 0..num_stages {
            let mut data_lane = DataLanes::with_capacity(num_stages);
            for l in 0..num_stages {
                data_lane.push(Wire::new(critical_path, debug));
            }
            bus.insert(i, data_lane);
        }
        Self { bus }
    }

    pub fn assign(&self, from: StageIndex, to: StageIndex, data: super::pipeline_stage::PipelineData) {
        let data_lane = self.bus.get(&from).unwrap();
        assert!(to < data_lane.len());
        let wire = &data_lane[to];
        wire.assign(data);
    }

    pub fn pull(&self, from: StageIndex, to: StageIndex) -> super::pipeline_stage::PipelineData {
        let data_lane = self.bus.get(&from).unwrap();
        assert!(to < data_lane.len());
        let wire = &data_lane[to];
        wire.read() 
    }

    pub fn clear(&self, stage: StageIndex) {
        let data_lanes = self.bus.get(&stage).unwrap();
        for wire in data_lanes {
            wire.clear();
        }
    }
}
