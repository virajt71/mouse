use super::Engine;
use std::time::{Duration, Instant};

impl Engine {
    pub fn handle_hscroll_event(&self, delta: i32, action_id: &str) {
        let threshold = {
            let cfg = self.inner.config.lock().unwrap();
            cfg.settings.hscroll_threshold as f32
        };
        let now = Instant::now();

        let is_volume = action_id == "volume_up" || action_id == "volume_down";
        let cooldown = if is_volume {
            Duration::from_millis(60)
        } else {
            Duration::from_millis(350)
        };

        let (accum_ref, last_fire_ref) = if delta < 0 {
            (
                &self.inner.hscroll_accum_right,
                &self.inner.hscroll_last_fire_right,
            )
        } else {
            (
                &self.inner.hscroll_accum_left,
                &self.inner.hscroll_last_fire_left,
            )
        };

        {
            let last_fire = last_fire_ref.lock().unwrap();
            if now.duration_since(*last_fire) < cooldown {
                let mut accum = accum_ref.lock().unwrap();
                *accum = 0.0;
                return;
            }
        }

        let step = (delta.abs() as f32).min(1.0);
        let mut accum = accum_ref.lock().unwrap();
        *accum += step;

        if *accum < threshold {
            return;
        }

        *accum = 0.0;
        *last_fire_ref.lock().unwrap() = now;

        log::info!("[Engine] HScroll action triggered: {}", action_id);
        self.execute_engine_action(action_id);
    }
}
