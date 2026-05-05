use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread::JoinHandle;
use std::time::Duration;

use serde::Serialize;
use serialport::SerialPort;
use tauri::{AppHandle, Emitter};

use crate::error::{AppError, AppResult};
use crate::port::config::PortConfig;

pub type TabId = u32;

#[derive(Debug)]
pub enum WorkerCommand {
    Write(Vec<u8>),
    SetDtr(bool),
    SetRts(bool),
    Shutdown,
}

#[derive(Serialize, Clone)]
struct RxPayload {
    bytes: Vec<u8>,
    ts: i64,
}

#[derive(Serialize, Clone)]
struct TxPayload {
    bytes: Vec<u8>,
    ts: i64,
}

#[derive(Serialize, Clone)]
struct DisconnectPayload {
    reason: String,
    ts: i64,
}

const READ_BUFFER_SIZE: usize = 1024;

pub struct PortHandle {
    pub sender: Sender<WorkerCommand>,
    pub join: JoinHandle<()>,
}

impl PortHandle {
    pub fn send_command(&self, cmd: WorkerCommand) -> AppResult<()> {
        self.sender
            .send(cmd)
            .map_err(|_| AppError::Serial("worker is not running".to_string()))
    }
}

pub fn spawn(
    tab_id: TabId,
    config: PortConfig,
    app: AppHandle,
) -> AppResult<PortHandle> {
    let port = config.to_serial().open()?;
    let (tx, rx) = mpsc::channel::<WorkerCommand>();
    let join = std::thread::Builder::new()
        .name(format!("rustcom-port-{tab_id}"))
        .spawn(move || run_loop(tab_id, port, rx, app))
        .map_err(|e| AppError::Io(e.to_string()))?;

    Ok(PortHandle { sender: tx, join })
}

fn run_loop(
    tab_id: TabId,
    mut port: Box<dyn SerialPort>,
    rx: Receiver<WorkerCommand>,
    app: AppHandle,
) {
    let mut read_buf = vec![0u8; READ_BUFFER_SIZE];
    loop {
        // Drain pending commands.
        loop {
            match rx.try_recv() {
                Ok(WorkerCommand::Write(bytes)) => {
                    let result = port.write_all(&bytes);
                    if result.is_ok() {
                        emit_tx(&app, tab_id, bytes);
                    } else if let Err(e) = result {
                        emit_disconnect(&app, tab_id, format!("write failed: {e}"));
                        return;
                    }
                }
                Ok(WorkerCommand::SetDtr(state)) => {
                    let _ = port.write_data_terminal_ready(state);
                }
                Ok(WorkerCommand::SetRts(state)) => {
                    let _ = port.write_request_to_send(state);
                }
                Ok(WorkerCommand::Shutdown) => {
                    return;
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => return,
            }
        }

        match port.read(&mut read_buf) {
            Ok(n) if n > 0 => emit_rx(&app, tab_id, read_buf[..n].to_vec()),
            Ok(_) => std::thread::sleep(Duration::from_millis(1)),
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {
                // Normal — no data this tick.
            }
            Err(e) => {
                emit_disconnect(&app, tab_id, format!("read failed: {e}"));
                return;
            }
        }
    }
}

fn emit_rx(app: &AppHandle, tab_id: TabId, bytes: Vec<u8>) {
    let payload = RxPayload { bytes, ts: chrono::Local::now().timestamp_millis() };
    let _ = app.emit(&format!("rx://{tab_id}"), payload);
}

fn emit_tx(app: &AppHandle, tab_id: TabId, bytes: Vec<u8>) {
    let payload = TxPayload { bytes, ts: chrono::Local::now().timestamp_millis() };
    let _ = app.emit(&format!("tx://{tab_id}"), payload);
}

fn emit_disconnect(app: &AppHandle, tab_id: TabId, reason: String) {
    let payload = DisconnectPayload { reason, ts: chrono::Local::now().timestamp_millis() };
    let _ = app.emit(&format!("disconnect://{tab_id}"), payload);
}
