#![cfg(target_arch = "wasm32")]

use wasm_bindgen::UnwrapThrowExt;

use web_sys::Performance;

#[cfg(debug_assertions)]
use gloo_console::info;

/// P value dictate the weigth given to newer value.
///
/// 0.0 <= P <= 1.0
const MOVING_AVERAGE_P: f64 = 0.15;

#[derive(Clone)]
pub struct ExponentialMovingAverage {
    performance: Performance,

    download_time: f64,

    moving_average: f64,
}

impl ExponentialMovingAverage {
    pub fn new() -> Self {
        let window = web_sys::window().unwrap_throw();
        let performance = window.performance().unwrap_throw();

        Self {
            performance,

            download_time: 0.0,
            moving_average: 0.0,
        }
    }

    pub fn start_timer(&mut self) {
        self.download_time = self.performance.now();
    }

    /// Returns the newly calculated average if start_timer() was previously called
    pub fn recalculate_average_speed(&mut self, bandwidth: f64) -> Option<f64> {
        if self.download_time <= 0.0 {
            return None;
        }

        let time = self.performance.now() - self.download_time;

        self.download_time = 0.0;

        #[cfg(debug_assertions)]
        info!(&format!("Last Download {:.0}ms", time));

        let new_bitrate = bandwidth / time * 1000.0;

        if self.moving_average >= 0.0 {
            self.moving_average += (new_bitrate - self.moving_average) * MOVING_AVERAGE_P;
        } else {
            self.moving_average = new_bitrate; // the first entry
        }

        #[cfg(debug_assertions)]
        info!(&format!(
            "Average Download Speed {:.0} kbps",
            self.moving_average / 1000.0
        ));

        Some(self.moving_average)
    }
}
