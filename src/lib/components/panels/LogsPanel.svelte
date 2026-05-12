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

  const decoder = new TextDecoder("utf-8", { fatal: false });
  function decodeForDisplay(bytes: number[]): string {
    return decoder.decode(new Uint8Array(bytes)).replace(/\r?\n$/, "");
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

  <div class="preview">
    {#if $activeTab.logEntries.length === 0}
      <p class="hint">No entries yet. Capture is {$activeTab.loggingEnabled ? "on" : "off"}.</p>
    {:else}
      {#each $activeTab.logEntries.slice(-50) as e, i (i)}
        <div class="line">
          <span class="ts">{new Date(e.ts_ms).toLocaleTimeString()}</span>
          <span class="dir {e.direction}">{e.direction.toUpperCase()}</span>
          <span class="text">{decodeForDisplay(e.bytes)}</span>
        </div>
      {/each}
    {/if}
  </div>

  <div class="row">
    <button class="primary" onclick={exportLog} disabled={$activeTab.logEntries.length === 0}>Save log…</button>
    <button onclick={() => clearLog($activeTabId)} disabled={$activeTab.logEntries.length === 0}>Clear</button>
  </div>

  <p class="hint">Capture starts from the moment you toggle this on. Entries are kept in memory only (not persisted across restarts).</p>
</div>

<style>
  .panel { padding: 12px; height: 100%; box-sizing: border-box; display: flex; flex-direction: column; gap: 8px; min-height: 0; }
  h4 { font-size: 10px; letter-spacing: 0.1em; text-transform: uppercase; color: var(--fg-subtle); margin: 0 0 6px; font-weight: 600; }
  .toggle { display: flex; align-items: center; gap: 6px; font-size: 11px; color: var(--fg-muted); }
  .meta { color: var(--fg-subtle); font-size: 11px; margin: 4px 0; }
  .preview {
    flex: 1;
    min-height: 80px;
    max-height: 320px;
    overflow-y: auto;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    padding: 6px 8px;
    font-family: var(--font-mono);
    font-size: 10px;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }
  .line { display: grid; grid-template-columns: auto auto 1fr; gap: 6px; align-items: baseline; }
  .ts { color: var(--fg-subtle); font-size: 9px; }
  .dir.rx { color: var(--rx); font-weight: 600; }
  .dir.tx { color: var(--tx); font-weight: 600; }
  .text { color: var(--fg); white-space: pre-wrap; word-break: break-word; }
  .row { display: flex; gap: 6px; }
  .row button { padding: 6px 10px; font-size: 10px; }
  .hint { color: var(--fg-subtle); font-size: 10px; margin-top: 8px; }
</style>
