<script lang="ts">
  import { activeTab, activeTabId, patchActiveTab, clearLog } from "$lib/stores/tabs";
  import { ipc, type LogEntryDto } from "$lib/ipc";
  import { save } from "@tauri-apps/plugin-dialog";

  async function exportLog() {
    const path = await save({
      title: "Save log as",
      defaultPath: `rustcom_${new Date().toISOString().replace(/[:.]/g, "-")}.log`,
      filters: [{ name: "Log", extensions: ["log", "txt"] }],
    });
    if (!path) return;
    const entries: LogEntryDto[] = $activeTab.logEntries.map((e) => ({
      ts_ms: e.ts_ms,
      direction: e.direction,
      bytes: e.bytes,
    }));
    try {
      const msg = await ipc.saveLog(path, entries);
      patchActiveTab({ errorMessage: msg });
    } catch (e) {
      patchActiveTab({ errorMessage: String(e) });
    }
  }
</script>

<div class="panel">
  <h4>Logs</h4>

  <label class="toggle">
    <input
      type="checkbox"
      checked={$activeTab.loggingEnabled}
      onchange={(e) => patchActiveTab({ loggingEnabled: (e.currentTarget as HTMLInputElement).checked })}
    />
    Capture RX/TX to log
  </label>

  <p class="meta">{$activeTab.logEntries.length} entr{$activeTab.logEntries.length === 1 ? "y" : "ies"} buffered</p>

  <div class="row">
    <button class="primary" onclick={exportLog} disabled={$activeTab.logEntries.length === 0}>Save log…</button>
    <button onclick={() => clearLog($activeTabId)} disabled={$activeTab.logEntries.length === 0}>Clear</button>
  </div>

  <p class="hint">Capture starts from the moment you toggle this on. Entries are kept in memory only (not persisted across restarts).</p>
</div>

<style>
  .panel { padding: 12px; height: 100%; box-sizing: border-box; display: flex; flex-direction: column; gap: 8px; }
  h4 { font-size: 10px; letter-spacing: 0.1em; text-transform: uppercase; color: var(--fg-subtle); margin: 0 0 6px; font-weight: 600; }
  .toggle { display: flex; align-items: center; gap: 6px; font-size: 11px; color: var(--fg-muted); }
  .meta { color: var(--fg-subtle); font-size: 11px; margin: 4px 0; }
  .row { display: flex; gap: 6px; }
  .row button { padding: 6px 10px; font-size: 10px; }
  .hint { color: var(--fg-subtle); font-size: 10px; margin-top: 8px; }
</style>
