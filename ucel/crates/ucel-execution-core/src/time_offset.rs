use std::time::{Duration, SystemTime};

#[derive(Clone, Debug)]
pub struct ServerTimeOffset {
    offset: Duration,
    last_local: Option<SystemTime>,
}

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum TimeOffsetError {
    #[error("local clock moved backwards")]
    LocalClockBackwards,
}

impl Default for ServerTimeOffset {
    fn default() -> Self {
        Self {
            offset: Duration::from_secs(0),
            last_local: None,
        }
    }
}

impl ServerTimeOffset {
    pub fn apply_offset(
        &mut self,
        local_now: SystemTime,
        server_now: SystemTime,
    ) -> Result<(), TimeOffsetError> {
        if let Some(last) = self.last_local {
            if local_now < last {
                return Err(TimeOffsetError::LocalClockBackwards);
            }
        }
        self.last_local = Some(local_now);

        self.offset = if server_now >= local_now {
            server_now.duration_since(local_now).unwrap_or_default()
        } else {
            Duration::from_secs(0)
        };

        Ok(())
    }

    pub fn now_server(&self, local_now: SystemTime) -> SystemTime {
        local_now + self.offset
    }

    pub fn offset(&self) -> Duration {
        self.offset
    }
}
