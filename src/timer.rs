use std::time::Instant;

pub struct MprisTimer {
    base_position: u128,
    base_instant: Instant,
    last_effective_time: u128,
}

impl MprisTimer {
    pub fn new() -> Self {
        Self {
            base_position: 0,
            base_instant: Instant::now(),
            last_effective_time: 0,
        }
    }

    /// 更新内部基准并返回 (当前有效进度, 是否 Stopped)
    pub fn update(
        &mut self,
        status: mpris::PlaybackStatus,
        playback_rate: f64,
        position_micros: Option<u128>,
    ) -> u128 {
        // Playing 时根据基准点和当前时间计算出当前进度
        if status == mpris::PlaybackStatus::Playing {
            if let Some(pos_micros) = position_micros {
                // 播放过程中，如果 mpris 上报了新的进度，则刷新基准点
                if pos_micros != self.base_position {
                    self.base_position = pos_micros;
                    self.base_instant = Instant::now();
                }
            }
            let delta = Instant::now().saturating_duration_since(self.base_instant);
            let delta_micros = delta.as_micros() as f64;
            let rate = if playback_rate <= 0.0 {
                1.0
            } else {
                playback_rate
            };
            let advanced = (delta_micros * rate) as u128;
            self.last_effective_time = self.base_position.saturating_add(advanced);
        };

        // Stopped 和 Paused 时直接返回上次的有效进度
        self.last_effective_time
    }

    /// 在 Stopped 时重置本地状态
    pub fn reset(&mut self) {
        self.base_position = 0;
        self.base_instant = Instant::now();
        self.last_effective_time = 0;
    }

    pub fn last_effective_time(&self) -> u128 {
        self.last_effective_time
    }
}
