use crate::{
    common::Task,
    scanner::{ScannerState, scan_video},
};

use super::ScanningModule;

impl ScanningModule {
    pub fn start_scanning(&mut self) {
        self.stop_scanning();

        self.texture = None;
        self.last_shown_tick = None;
        self.export_error = None;

        let settings = self.settings.clone();
        self.scan = Some(Task::spawn(ScannerState::default(), |state, cancel| {
            async move {
                scan_video(&settings, state, cancel).await;
            }
        }));
    }

    pub fn stop_scanning(&mut self) {
        if let Some(task) = self.scan.as_ref() {
            task.stop();
        }
    }

    pub fn is_scanning(&self) -> bool {
        self.scan.as_ref().is_some_and(|task| task.is_running())
    }
}
