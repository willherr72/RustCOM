<script lang="ts">
  import { type LineEnding, type Macro } from "$lib/ipc";

  let {
    draft = $bindable<Macro>(),
    saving = false,
    panelError = null,
    onSave,
    onCancel,
    onDelete = null,
  }: {
    draft: Macro;
    saving?: boolean;
    panelError?: string | null;
    onSave: () => void;
    onCancel: () => void;
    onDelete?: (() => void) | null;
  } = $props();

  const HOTKEYS = ["", "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "F10", "F11", "F12"];
</script>

<div class="edit">
  <input placeholder="Name" bind:value={draft.name} />
  <div class="row">
    <select bind:value={draft.mode}>
      <option value="ascii">ASCII</option>
      <option value="hex">Hex</option>
    </select>
    {#if draft.mode === "ascii"}
      <select
        value={draft.line_ending ?? "none"}
        onchange={(e) => (draft.line_ending = (e.currentTarget as HTMLSelectElement).value as LineEnding)}
      >
        <option value="none">None</option>
        <option value="cr">\r</option>
        <option value="lf">\n</option>
        <option value="crlf">\r\n</option>
      </select>
    {/if}
    <select
      value={draft.hotkey ?? ""}
      onchange={(e) => (draft.hotkey = (e.currentTarget as HTMLSelectElement).value || null)}
    >
      {#each HOTKEYS as k}
        <option value={k}>{k || "no hotkey"}</option>
      {/each}
    </select>
  </div>
  <textarea
    rows="2"
    bind:value={draft.payload}
    placeholder={draft.mode === "ascii" ? "Message…" : "AA BB 0D 0A"}
  ></textarea>
  {#if panelError}<p class="err">{panelError}</p>{/if}
  <div class="row gap">
    <button class="primary" onclick={onSave} disabled={saving}>{saving ? "Saving…" : "Save"}</button>
    <button onclick={onCancel}>Cancel</button>
    {#if onDelete}<button class="danger" onclick={onDelete}>Delete</button>{/if}
  </div>
</div>

<style>
  .edit { display: flex; flex-direction: column; gap: 6px; padding: 8px; background: var(--bg-input); border-radius: var(--radius-md); border: 1px solid var(--accent); }
  .edit input, .edit select, .edit textarea { font-family: var(--font-mono); font-size: 11px; }
  .edit textarea { resize: vertical; min-height: 36px; padding: 6px 8px; border-radius: var(--radius-md); background: var(--bg); color: var(--fg); border: 1px solid transparent; }
  .edit textarea:focus { border-color: var(--accent); outline: none; }
  .row { display: flex; gap: 4px; min-width: 0; }
  .row.gap { gap: 6px; flex-wrap: wrap; }
  .row select { flex: 1 1 0; min-width: 0; }
  .edit input, .edit textarea { box-sizing: border-box; width: 100%; }
  .err { color: var(--err); font-size: 11px; margin: 0; }
</style>
