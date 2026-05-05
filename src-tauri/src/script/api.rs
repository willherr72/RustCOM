use std::sync::mpsc::Sender;

use mlua::{Lua, Result as LuaResult, Table, Value};

use crate::port::worker::TabId;

/// Commands the script engine sends out to the rest of the app.
#[derive(Debug)]
pub enum ScriptOut {
    Send { tab_id: TabId, bytes: Vec<u8> },
    Toast { level: String, msg: String },
    Log { msg: String },
}

/// Install the `serial`, `ui`, and free functions onto the Lua globals.
/// Closures capture an `mpsc::Sender<ScriptOut>` so script API calls become messages.
pub fn install_api(lua: &Lua, tab_id: TabId, out: Sender<ScriptOut>) -> LuaResult<()> {
    let globals = lua.globals();

    // serial.send(bytes) — bytes is a Lua table of integers 0..255.
    let send_out = out.clone();
    let serial = lua.create_table()?;
    serial.set(
        "send",
        lua.create_function(move |_, bytes: Table| {
            let mut buf = Vec::new();
            for pair in bytes.sequence_values::<u8>() {
                buf.push(pair?);
            }
            let _ = send_out.send(ScriptOut::Send { tab_id, bytes: buf });
            Ok(())
        })?,
    )?;

    let send_text_out = out.clone();
    serial.set(
        "send_text",
        lua.create_function(move |_, text: String| {
            let _ = send_text_out.send(ScriptOut::Send {
                tab_id,
                bytes: text.into_bytes(),
            });
            Ok(())
        })?,
    )?;

    let send_hex_out = out.clone();
    serial.set(
        "send_hex",
        lua.create_function(move |_, input: String| {
            match crate::hex::parse_hex_input(&input) {
                Ok(bytes) => {
                    let _ = send_hex_out.send(ScriptOut::Send { tab_id, bytes });
                    Ok(())
                }
                Err(e) => Err(mlua::Error::external(e)),
            }
        })?,
    )?;

    globals.set("serial", serial)?;

    // ui.toast(msg, level?)
    let toast_out = out.clone();
    let ui = lua.create_table()?;
    ui.set(
        "toast",
        lua.create_function(move |_, args: (String, Option<String>)| {
            let (msg, level) = args;
            let _ = toast_out.send(ScriptOut::Toast {
                level: level.unwrap_or_else(|| "info".to_string()),
                msg,
            });
            Ok(())
        })?,
    )?;
    globals.set("ui", ui)?;

    // log(msg)
    let log_out = out;
    globals.set(
        "log",
        lua.create_function(move |_, msg: Value| {
            let s = match msg {
                Value::String(s) => s.to_str()?.to_string(),
                other => format!("{other:?}"),
            };
            let _ = log_out.send(ScriptOut::Log { msg: s });
            Ok(())
        })?,
    )?;

    // delay(ms) — sleeps the script thread.
    globals.set(
        "delay",
        lua.create_function(|_, ms: u64| {
            std::thread::sleep(std::time::Duration::from_millis(ms));
            Ok(())
        })?,
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    fn fresh() -> (Lua, mpsc::Receiver<ScriptOut>) {
        let lua = Lua::new();
        let (tx, rx) = mpsc::channel();
        install_api(&lua, 7, tx).unwrap();
        (lua, rx)
    }

    #[test]
    fn serial_send_forwards_bytes() {
        let (lua, rx) = fresh();
        lua.load("serial.send({0x41, 0x42})").exec().unwrap();
        match rx.recv().unwrap() {
            ScriptOut::Send { tab_id, bytes } => {
                assert_eq!(tab_id, 7);
                assert_eq!(bytes, vec![0x41, 0x42]);
            }
            other => panic!("expected Send, got {other:?}"),
        }
    }

    #[test]
    fn serial_send_text_forwards_string_bytes() {
        let (lua, rx) = fresh();
        lua.load(r#"serial.send_text("AT")"#).exec().unwrap();
        match rx.recv().unwrap() {
            ScriptOut::Send { bytes, .. } => assert_eq!(bytes, b"AT".to_vec()),
            other => panic!("expected Send, got {other:?}"),
        }
    }

    #[test]
    fn serial_send_hex_forwards_parsed_bytes() {
        let (lua, rx) = fresh();
        lua.load(r#"serial.send_hex("AA BB 0D 0A")"#).exec().unwrap();
        match rx.recv().unwrap() {
            ScriptOut::Send { bytes, .. } => assert_eq!(bytes, vec![0xAA, 0xBB, 0x0D, 0x0A]),
            other => panic!("expected Send, got {other:?}"),
        }
    }

    #[test]
    fn log_forwards_string() {
        let (lua, rx) = fresh();
        lua.load(r#"log("hello")"#).exec().unwrap();
        match rx.recv().unwrap() {
            ScriptOut::Log { msg } => assert_eq!(msg, "hello"),
            other => panic!("expected Log, got {other:?}"),
        }
    }

    #[test]
    fn ui_toast_with_level() {
        let (lua, rx) = fresh();
        lua.load(r#"ui.toast("oops", "warn")"#).exec().unwrap();
        match rx.recv().unwrap() {
            ScriptOut::Toast { level, msg } => {
                assert_eq!(level, "warn");
                assert_eq!(msg, "oops");
            }
            other => panic!("expected Toast, got {other:?}"),
        }
    }
}
