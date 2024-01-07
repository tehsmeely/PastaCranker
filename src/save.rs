use crate::core_elements::{CoreParameters, CoreState};
use crate::State;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use anyhow::{anyhow, Error};
use crankstart::file::FileSystem;
use crankstart::log_to_console;
use crankstart_sys::FileOptions;
use serde::{Deserialize, Serialize};
use serde_json_core::heapless;

const SAVE_FILE: &str = "PastaCranker-savefile.json";

/// This is picked randomly, but should be large enough to hold the save state. The BigUint values
/// are not a static size so we can't know for certain!
const FILE_BUFFER_SIZE: usize = 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SaveState {
    parameters: CoreParameters,
    state: CoreState,
    menu_counts: Vec<usize>,
}

pub fn save_state(state: &State) {
    let mut fs = FileSystem::get();
    let save_state = SaveState {
        parameters: state.parameters.clone(),
        state: state.state.clone(),
        menu_counts: state.menu.to_counts(),
    };
    let result: heapless::String<FILE_BUFFER_SIZE> =
        serde_json_core::ser::to_string(&save_state).unwrap();
    log_to_console!("result: {}", result);
    let buf = result.as_bytes();
    let mut file = fs.open(SAVE_FILE, FileOptions::kFileWrite).unwrap();
    let num_bytes_written = file.write(&buf).unwrap();
    log_to_console!("num_bytes_written: {}", num_bytes_written);
    file.flush().unwrap();
}

pub fn load_state() -> Result<(CoreState, CoreParameters, Vec<usize>), Error> {
    let mut fs = FileSystem::get();
    let mut file = fs.open(SAVE_FILE, FileOptions::kFileReadData)?;
    let mut buf = [0u8; FILE_BUFFER_SIZE];
    let bytes_read = file.read(&mut buf)?;
    log_to_console!("bytes_read: {}", bytes_read);
    let (save_state, bytes_parsed): (SaveState, usize) =
        serde_json_core::de::from_slice(&buf[..bytes_read])
            .map_err(|e| anyhow!("Serde-error deserialising: {}", e))?;
    log_to_console!("bytes_parsed: {}", bytes_parsed);
    Ok((
        save_state.state,
        save_state.parameters,
        save_state.menu_counts,
    ))
}
