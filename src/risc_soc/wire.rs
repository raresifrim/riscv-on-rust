use std::{sync::{Arc, Condvar, Mutex}, time::Duration};
use crate::risc_soc::pipeline_stage::PipelineData;

/// WireData should represent combinational logic data that is passed through "wire" structures such as in the case of the wire net type in Verilog
/// In order to react to it we are using the CondVar sync mechanism in Rust
/// If there is any kind of data that arrived until the specified `critical_path` delay, then we can read it
/// The `critical_path` delay should usually be within the clock cycle of the cpu, thus modeling the behaviour of metastability if the setup and hold up times are violated

/// we are reusing Pipeline data here for olding the actual bits and bytes that we want to "wire"
pub struct WireData {
    data: Arc<(Mutex<PipelineData>, Condvar)>,
    critical_path: u64,
}

impl WireData {

    pub fn new(critical_path: u64) -> Self {
        Self {
            critical_path,
            data: Arc::new((Mutex::new(PipelineData(vec![])), Condvar::new()))
        }
    }

    pub fn clear(&self) {
        let pair = self.data.clone();
        let (lock, cvar) = &*pair;
        let mut wire = lock.lock().unwrap();
        *wire = PipelineData(vec![]); 
    }

    pub fn put(&self, data: PipelineData){
        let pair = self.data.clone();
        let (lock, cvar) = &*pair;
        let mut wire = lock.lock().unwrap();
        *wire = data.clone();
        cvar.notify_all();
    }

    pub fn get(&self) -> PipelineData{
        let pair = self.data.clone();
        let (lock, cvar) = &*pair; 
        let wire = lock.lock().unwrap(); 
        let result = cvar.wait_timeout(wire, Duration::from_nanos(self.critical_path)).unwrap();
        if result.1.timed_out() && result.0.is_empty() {
            tracing::warn!("Setup + Holdup times might have been violated!");
            PipelineData(vec![])
        } else {
            tracing::info!("Combinational logic path was within the defined critical path");
            result.0.clone()
        }
    }
}