use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

use mlua::{Function, Lua};
use parking_lot::Mutex;
use tauri::{AppHandle, Emitter, Manager};

use crate::error::{AppError, AppResult};
use crate::port::worker::TabId;
use crate::script::api::{install_api, ScriptOut};

#[derive(Debug)]
pub enum EngineCommand {
    Run { tab_id: TabId, source: String },
    Stop,
    OnRecv { tab_id: TabId, bytes: Vec<u8> },
    Shutdown,
}

pub struct ScriptEngine {
    sender: Mutex<Option<Sender<EngineCommand>>>,
    join: Mutex<Option<JoinHandle<()>>>,
}

impl ScriptEngine {
    pub fn new() -> Self {
        Self {
            sender: Mutex::new(None),
            join: Mutex::new(None),
        }
    }

    pub fn start(&self, app: AppHandle) -> AppResult<()> {
        let mut sender_slot = self.sender.lock();
        if sender_slot.is_some() {
            return Ok(()); // already running
        }
        let (tx, rx) = mpsc::channel::<EngineCommand>();
        *sender_slot = Some(tx);
        drop(sender_slot);

        let app_for_thread = app.clone();
        let handle = thread::Builder::new()
            .name("rustcom-script-engine".to_string())
            .spawn(move || run_engine(rx, app_for_thread))
            .map_err(|e| AppError::Io(e.to_string()))?;
        *self.join.lock() = Some(handle);
        Ok(())
    }

    pub fn send(&self, cmd: EngineCommand) -> AppResult<()> {
        let guard = self.sender.lock();
        let sender = guard
            .as_ref()
            .ok_or_else(|| AppError::Invalid("script engine is not running".to_string()))?;
        sender
            .send(cmd)
            .map_err(|_| AppError::Invalid("script engine thread is dead".to_string()))
    }

    pub fn run(&self, tab_id: TabId, source: String) -> AppResult<()> {
        self.send(EngineCommand::Run { tab_id, source })
    }

    pub fn stop(&self) -> AppResult<()> {
        self.send(EngineCommand::Stop)
    }

    pub fn dispatch_recv(&self, tab_id: TabId, bytes: Vec<u8>) {
        let _ = self.send(EngineCommand::OnRecv { tab_id, bytes });
    }
}

impl Default for ScriptEngine {
    fn default() -> Self { Self::new() }
}

fn run_engine(rx: Receiver<EngineCommand>, app: AppHandle) {
    let lua = Lua::new();
    let (out_tx, out_rx) = mpsc::channel::<ScriptOut>();

    // Active script context: the tab id it was bound to.
    let mut bound_tab: Option<TabId> = None;

    // Spawn the forwarder thread that turns ScriptOut into Tauri events / port writes.
    let app_forward = app.clone();
    thread::Builder::new()
        .name("rustcom-script-forwarder".to_string())
        .spawn(move || forward_loop(out_rx, app_forward))
        .ok();

    loop {
        match rx.recv() {
            Ok(EngineCommand::Run { tab_id, source }) => {
                // Reset globals: clear on_recv from any previous run.
                let _ = lua.globals().set("on_recv", mlua::Value::Nil);
                if let Err(e) = install_api(&lua, tab_id, out_tx.clone()) {
                    let _ = app.emit(
                        "script_error",
                        format!("install_api failed: {e}"),
                    );
                    continue;
                }
                if let Err(e) = lua.load(&source).exec() {
                    let _ = app.emit("script_error", format!("{e}"));
                    continue;
                }
                bound_tab = Some(tab_id);
            }
            Ok(EngineCommand::Stop) => {
                let _ = lua.globals().set("on_recv", mlua::Value::Nil);
                bound_tab = None;
            }
            Ok(EngineCommand::OnRecv { tab_id, bytes }) => {
                if bound_tab != Some(tab_id) {
                    continue;
                }
                let cb_value: mlua::Value = match lua.globals().get("on_recv") {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                let cb: Function = match cb_value {
                    mlua::Value::Function(f) => f,
                    _ => continue,
                };
                let table = match lua.create_sequence_from(bytes) {
                    Ok(t) => t,
                    Err(_) => continue,
                };
                if let Err(e) = cb.call::<()>(table) {
                    let _ = app.emit("script_error", format!("on_recv: {e}"));
                }
            }
            Ok(EngineCommand::Shutdown) => return,
            Err(_) => return,
        }
    }
}

fn forward_loop(rx: Receiver<ScriptOut>, app: AppHandle) {
    use crate::port::manager::PortManager;
    loop {
        let item = match rx.recv() {
            Ok(v) => v,
            Err(_) => return,
        };
        match item {
            ScriptOut::Send { tab_id, bytes } => {
                let mgr = app.state::<PortManager>();
                let _ = mgr.send(tab_id, bytes);
            }
            ScriptOut::Toast { level, msg } => {
                #[derive(serde::Serialize, Clone)]
                struct ToastPayload { level: String, msg: String, ts: i64 }
                let _ = app.emit(
                    "toast",
                    ToastPayload { level, msg, ts: chrono::Local::now().timestamp_millis() },
                );
            }
            ScriptOut::Log { msg } => {
                #[derive(serde::Serialize, Clone)]
                struct LogPayload { msg: String, ts: i64 }
                let _ = app.emit(
                    "script_log",
                    LogPayload { msg, ts: chrono::Local::now().timestamp_millis() },
                );
            }
        }
    }
}
