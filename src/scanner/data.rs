use std::path::PathBuf;

#[derive(Debug, Default, Clone)]
pub struct VideoTime {
    pub hours: u32,
    pub minutes: u32,
    pub seconds: u32,
    pub enabled: bool,
}

impl VideoTime {
    pub fn get(&self) -> Option<String> {
        self.enabled
            .then(|| format!("{:02}:{:02}:{:02}", self.hours, self.minutes, self.seconds))
    }
}

#[derive(Debug, Clone, Default)]
pub struct CrackingSettings {
    pub video_path: Option<PathBuf>,
    pub start_time: VideoTime,
    pub end_time: VideoTime,
    pub cuda: bool,
    pub rect: (u32, u32, u32, u32),
}
