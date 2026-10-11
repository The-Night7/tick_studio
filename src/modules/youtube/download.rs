use std::path::PathBuf;

use crate::common::{CancelToken, Task};

use super::ytdlp::{self, DownloadProgress};

pub struct ActiveDownload {
    task: Task<DownloadProgress>,
}

impl ActiveDownload {
    pub fn spawn(url: String, format_id: String, folder: PathBuf) -> Self {
        let task = Task::spawn(DownloadProgress::Started, |state, cancel: CancelToken| {
            async move {
                ytdlp::download_video(url, format_id, folder, state, cancel).await;
            }
        });
        Self { task }
    }

    pub fn progress(&self) -> DownloadProgress {
        self.task.state().lock().clone()
    }

    pub fn is_running(&self) -> bool {
        self.task.is_running()
    }

    pub fn stop(&self) {
        self.task.stop();
    }
}
