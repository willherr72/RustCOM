<script lang="ts">
  import { onMount } from "svelte";
  import { macros, loadMacros, upsertMacro, removeMacro, newMacroId } from "$lib/stores/macros";
  import { activeTab, activeTabId, patchActiveTab } from "$lib/stores/tabs";
  import { ipc, type LineEnding, type Macro } from "$lib/ipc";

  const HOTKEYS = ["", "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "F10", "F11", "F12"];

  let editingId = $state<string | null>(null);
  let draft = $state<Macro>(blankMacro());

  function blankMacro(): Macro {
    return { id: newMacroId(), name: "", mode: "ascii", payload: "", line_ending: "crlf", hotkey: null };
  }

  onMount(() => { void loadMacros(); });

  function startNew() {
    draft = blankMacro();
    editingId = draft.id;
  }

  function startEdit(m: Macro) {
    draft = { ...m };
    editingId = m.id;
  }

  async function save() {
    if (!draft.name.trim()) {
      patchActiveTab({ errorMessage: "Macro needs a name" });
      return;
    }
    await upsertMacro(draft);
    editingId = null;
  }

  async function del(id: string) {
    await removeMacro(id);
    if (editingId === id) editingId = null;
  }

  async function run(m: Macro) {
    if ($activeTab.state !== "connected") {
      patchActiveTab({ errorMessage: "Connect a port before running a macro" });
      return;
    }
    try {
      if (m.mode === "ascii") {
        await ipc.sendText($activeTabId, m.payload, (m.line_ending ?? "none") as LineEnding);
      } else {
        await ipc.sendHex($activeTabId, m.payload);
      }
    } catch (e) {
      patchActiveTab({ errorMessage: String(e) });
    }
  }
</script>

<div class="panel">
  <div class="header">
    <h4>Macros</h4>
    <button class="add" onclick={startNew}>+ New</button>
  </div>

  {#if $macros.length === 0}
    <p class="empty">No macros yet. Click <em>+ New</em> to add one.</p>
  {/if}

  {#each $macros as m (m.id)}
    {#if editingId === m.id}
      <div class="edit">
        <input placeholder="Name" bind:value={draft.name} />
        <div class="row">
          <select bind:value={draft.mode}>
            <option value="ascii">ASCII</option>
            <option value="hex">Hex</option>
          </select>
          {#if draft.mode === "ascii"}
            <select value={draft.line_ending ?? "none"} onchange={(e) => (draft.line_ending = (e.currentTarget as HTMLSelectElement).value as LineEnding)}>
              <option value="none">None</option>
              <option value="cr">\r</option>
              <option value="lf">\n</option>
              <option value="crlf">\r\n</option>
            </select>
          {/if}
          <select value={draft.hotkey ?? ""} onchange={(e) => (draft.hotkey = (e.currentTarget as HTMLSelectElement).value || null)}>
            {#each HOTKEYS as k}
              <option value={k}>{k || "no hotkey"}</option>
            {/each}
          </select>
        </div>
        <textarea rows="2" bind:value={draft.payload} placeholder={draft.mode === "ascii" ? "Message…" : "AA BB 0D 0A"}></textarea>
        <div class="row gap">
          <button class="primary" onclick={save}>Save</button>
          <button onclick={() => (editingId = null)}>Cancel</button>
          <button class="danger" onclick={() => del(m.id)}>Delete</button>
        </div>
      </div>
    {:else}
      <div class="item">
        <div class="meta">
          <strong>{m.name || "(unnamed)"}</strong>
          <span class="kind">{m.mode}{m.hotkey ? ` · ${m.hotkey}` : ""}</span>
        </div>
        <div class="actions">
          <button onclick={() => run(m)}>Run</button>
          <button onclick={() => startEdit(m)}>Edit</button>
        </div>
      </div>
    {/if}
  {/each}

  {#if editingId && !$macros.some((m) => m.id === editingId)}
    <div class="edit">
      <input placeholder="Name" bind:value={draft.name} />
      <div class="row">
        <select bind:value={draft.mode}>
          <option value="ascii">ASCII</option>
          <option value="hex">Hex</option>
        </select>
        {#if draft.mode === "ascii"}
          <select value={draft.line_ending ?? "none"} onchange={(e) => (draft.line_ending = (e.currentTarget as HTMLSelectElement).value as LineEnding)}>
            <option value="none">None</option>
            <option value="cr">\r</option>
            <option value="lf">\n</option>
            <option value="crlf">\r\n</option>
          </select>
        {/if}
        <select value={draft.hotkey ?? ""} onchange={(e) => (draft.hotkey = (e.currentTarget as HTMLSelectElement).value || null)}>
          {#each HOTKEYS as k}
            <option value={k}>{k || "no hotkey"}</option>
          {/each}
        </select>
      </div>
      <textarea rows="2" bind:value={draft.payload} placeholder={draft.mode === "ascii" ? "Message…" : "AA BB 0D 0A"}></textarea>
      <div class="row gap">
        <button class="primary" onclick={save}>Save</button>
        <button onclick={() => (editingId = null)}>Cancel</button>
      </div>
    </div>
  {/if}
</div>

<style>
  .panel { padding: 12px; height: 100%; box-sizing: border-box; display: flex; flex-direction: column; gap: 8px; overflow: auto; }
  .header { display: flex; align-items: center; justify-content: space-between; }
  h4 { font-size: 10px; letter-spacing: 0.1em; text-transform: uppercase; color: var(--fg-subtle); margin: 0; font-weight: 600; }
  .add { padding: 4px 8px; font-size: 10px; }
  .empty { color: var(--fg-subtle); font-size: 11px; }

  .item { display: flex; align-items: center; justify-content: space-between; gap: 8px; padding: 6px 8px; background: var(--bg-input); border-radius: var(--radius-md); }
  .meta { display: flex; flex-direction: column; min-width: 0; font-size: 11px; }
  .meta strong { color: var(--fg); font-weight: 500; }
  .kind { color: var(--fg-subtle); font-size: 9px; text-transform: uppercase; letter-spacing: 0.06em; }
  .actions { display: flex; gap: 4px; }
  .actions button { padding: 4px 8px; font-size: 10px; }

  .edit { display: flex; flex-direction: column; gap: 6px; padding: 8px; background: var(--bg-input); border-radius: var(--radius-md); border: 1px solid var(--accent); }
  .edit input, .edit select, .edit textarea { font-family: var(--font-mono); font-size: 11px; }
  .edit textarea { resize: vertical; min-height: 36px; padding: 6px 8px; border-radius: var(--radius-md); background: var(--bg); color: var(--fg); border: 1px solid transparent; }
  .edit textarea:focus { border-color: var(--accent); outline: none; }
  .row { display: flex; gap: 4px; min-width: 0; }
  .row.gap { gap: 6px; flex-wrap: wrap; }
  .row select { flex: 1 1 0; min-width: 0; }
  .edit input, .edit textarea { box-sizing: border-box; width: 100%; }
</style>
