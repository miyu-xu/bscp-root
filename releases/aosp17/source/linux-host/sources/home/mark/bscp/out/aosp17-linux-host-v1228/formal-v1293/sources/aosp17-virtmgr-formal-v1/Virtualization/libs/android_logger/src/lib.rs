use log::LevelFilter;

#[derive(Clone, Copy)]
pub enum LogId {
    System,
}

#[derive(Default, Clone)]
pub struct Config {
    _tag: Option<&'static str>,
    _level: Option<LevelFilter>,
    _log_id: Option<LogId>,
}

impl Config {
    pub fn with_tag(mut self, tag: &'static str) -> Self {
        self._tag = Some(tag);
        self
    }
    pub fn with_max_level(mut self, level: LevelFilter) -> Self {
        self._level = Some(level);
        self
    }
    pub fn with_log_buffer(mut self, log_id: LogId) -> Self {
        self._log_id = Some(log_id);
        self
    }
}

pub fn init_once(_config: Config) {}
