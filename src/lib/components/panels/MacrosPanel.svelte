<script lang="ts">
  import { onMount } from "svelte";
  import { macros, loadMacros, upsertMacro, removeMacro, newMacroId } from "$lib/stores/macros";
  import { activeTab, activeTabId, patchActiveTab } from "$lib/stores/tabs";
  import { ipc, type LineEnding, type Macro } from "$lib/ipc";
  import MacroEditor from "$lib/components/panels/MacroEditor.svelte";

  let editingId = $state<string | null>(null);
  let draft = $state<Macro>(blankMacro());
  let panelError = $state<string | null>(null);
  let saving = $state(false);

  function blankMacro(): Macro {
    return { id: newMacroId(), name: "", mode: "ascii", payload: "", line_ending: "crlf", hotkey: null };
  }

  onMount(() => { void loadMacros(); });

  function startNew() {
    draft = blankMacro();
    editingId = draft.id;
    panelError = null;
  }

  function startEdit(m: Macro) {
    draft = { ...m };
    editingId = m.id;
    panelError = null;
  }

  async function save() {
    panelError = null;
    if (!draft.name.trim()) {
      panelError = "Macro needs a name";
      return;
    }
    saving = true;
    try {
      await upsertMacro({ ...draft, name: draft.name.trim() });
      editingId = null;
    } catch (e) {
      panelError = `Save failed: ${String(e)}`;
    } finally {
      saving = false;
    }
  }

  async function del(id: string) {
    try {
      await removeMacro(id);
      if (editingId === id) editingId = null;
    } catch (e) {
      panelError = `Delete failed: ${String(e)}`;
    }
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

  {#if $macros.length === 0 && !editingId}
    <p class="empty">No macros yet. Click <em>+ New</em> to add one.</p>
  {/if}

  {#each $macros as m (m.id)}
    {#if editingId === m.id}
      <MacroEditor
        bind:draft
        {saving}
        {panelError}
        onSave={save}
        onCancel={() => { editingId = null; panelError = null; }}
        onDelete={() => del(m.id)}
      />
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
    <MacroEditor
      bind:draft
      {saving}
      {panelError}
      onSave={save}
      onCancel={() => { editingId = null; panelError = null; }}
    />
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
</style>
