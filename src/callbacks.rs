use crate::{GpsState, MainState, SlowState};

#[derive(Clone, Copy, Debug)]
pub struct BlackboxCallbacks {
    pub load_main_state: fn(&mut MainState, u32),
    pub load_slow_state: fn(&mut SlowState),
    pub load_gps_state: fn(&mut GpsState),
}

impl Default for BlackboxCallbacks {
    fn default() -> Self {
        Self { load_main_state: |_, _| {}, load_slow_state: |_| {}, load_gps_state: |_| {} }
    }
}
