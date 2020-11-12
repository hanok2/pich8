use std::sync::mpsc::Receiver;
use getset::{CopyGetters, Getters};

pub enum FileDialogType {
    OpenRom,
    SaveState,
    
    #[cfg(feature = "rom-download")]
    InputUrl,
}

pub enum FileDialogResult {
    None,
    OpenRom(String),
    SaveState(String),

    #[cfg(feature = "rom-download")]
    InputUrl(String),
}

/// This module handles dialogs in a separate thread.
/// Unforutnately, it's necessary due to a bug in the winit event loop.
/// See https://github.com/rust-windowing/winit/issues/1698
#[derive(CopyGetters, Getters)]
pub struct DialogHandler {
    #[getset(get_copy = "pub")]
    is_open: bool,
    chan_rx: Option<Receiver<FileDialogResult>>,
}

impl DialogHandler {
    const STATE_FILTER_PATT: &'static [&'static str] = &["*.p8s"];
    const STATE_FILTER_DESC: &'static str = "pich8 State (*.p8s)";

    pub fn new() -> Self {
        Self {
            is_open: false,
            chan_rx: None,
        }
    }

    pub fn open_file_dialog(&mut self, dialog_type: FileDialogType) {
        self.is_open = true;

        let (tx, rx) = std::sync::mpsc::channel();
        self.chan_rx = Some(rx);

        std::thread::spawn(move || {
            let mut result = FileDialogResult::None;
            match dialog_type {
                FileDialogType::OpenRom => {
                    if let Some(file_path) = tinyfiledialogs::open_file_dialog("Open ROM", "", None) {
                        result = FileDialogResult::OpenRom(file_path);
                    }
                },
                FileDialogType::SaveState => {
                    if let Some(file_path) = tinyfiledialogs::save_file_dialog_with_filter("Save State", "", DialogHandler::STATE_FILTER_PATT, DialogHandler::STATE_FILTER_DESC) {
                        result = FileDialogResult::SaveState(if file_path.contains(".") { file_path } else { format!("{}.p8s", file_path) });
                    }
                },
                
                #[cfg(feature = "rom-download")]
                FileDialogType::InputUrl => {
                    if let Some(url) = tinyfiledialogs::input_box("Input ROM URL", "Please input the URL pointing to the ROM file.\nFor Github, please make sure to use the raw file link!", "") {
                        if url.len() > 0 {
                            result = FileDialogResult::InputUrl(url);
                        }
                    }
                },
            }

            tx.send(result).expect("Communication failed");
        });
    }

    pub fn check_result(&mut self) -> FileDialogResult {
        let mut result = FileDialogResult::None;
        if self.chan_rx.is_some() {
            if let Some(chan) = self.chan_rx.as_ref() {
                if let Ok(dialog_result) = chan.try_recv() {
                    self.is_open = false;
                    result = dialog_result;
                }
            }
        }

        result
    }
}