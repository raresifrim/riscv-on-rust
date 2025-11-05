use crate::risc_soc::pipeline_stage::PipelineData;
use std::{
    sync::{Arc, Condvar, Mutex},
    time::Duration,
};
use std::hash::{BuildHasher, Hasher, RandomState};

/// WireData should represent combinational logic data that is passed through "wire" structures such as in the case of the wire net type in Verilog
/// In order to react to it we are using the CondVar sync mechanism in Rust
/// If there is any kind of data that arrived until the specified `critical_path` delay, then we can read it
/// The `critical_path` delay should usually be within the clock cycle of the cpu, thus modeling the behaviour of metastability if the setup and hold up times are violated

/// we are reusing Pipeline data here for olding the actual bits and bytes that we want to "wire"
pub struct Wire {
    /// We make use of Option as a Valid assertion for our wire data
    data: Arc<(Mutex<Option<PipelineData>>, Condvar)>,
    critical_path: Option<u128>,
    debug: bool,
}

impl Wire {
    pub fn new(critical_path: Option<u128>, debug: bool) -> Self {
        Self {
            critical_path,
            data: Arc::new((Mutex::new(None), Condvar::new())),
            debug,
        }
    }

    pub fn enable_debug(&mut self, debug: bool) {
        self.debug = debug;
    }

    pub fn clear(&self) {
        let pair = self.data.clone();
        let (lock, cvar) = &*pair;
        let mut wire = lock.lock().unwrap();
        *wire = None;
    }

    pub fn assign(&self, data: PipelineData) {
        let pair = self.data.clone();
        let (lock, cvar) = &*pair;
        let mut wire = lock.lock().unwrap();
        *wire = Some(data.clone());
        cvar.notify_all();
    }

    pub fn read(&self) -> PipelineData {
        let pair = self.data.clone();
        let (lock, cvar) = &*pair;
        let wire = lock.lock().unwrap();

        if self.critical_path.is_some() {
            let result = cvar
                .wait_timeout_while(wire, Duration::from_nanos(self.critical_path.unwrap() as u64), |data|{
                    data.is_none()
                })
                .unwrap();
            if result.1.timed_out() {
                if self.debug {
                    println!("Setup + Holdup times might have been violated by some critical path!");
                } else {
                    tracing::warn!("Setup + Holdup times might have been violated by some critical path!");
                }
                //return some undefined large enough data to mimic the behaviour of reading while updating in setup/holdup times
                let bytes: Vec<u8> = (0..256)
                .map(|_| (RandomState::new().build_hasher().finish() % 255) as u8)
                .collect();
                return PipelineData(bytes)
            } else {
                if self.debug {
                    println!("Combinational logic delay was within the defined critical path");
                } else {
                    tracing::info!("Combinational logic delay was within the defined critical path");
                }
                return result.0.as_ref().unwrap().clone()
            }
        } else {
            let result = cvar.wait_while(wire, |data|{
                data.is_none()
            }).unwrap();
            let data = result.as_ref().unwrap();
            //should never get empty data as this models ideal behaviour
            assert!(!data.is_empty());
            return data.clone();
        }
    }

}

