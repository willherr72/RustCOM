<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import {
    scripts,
    runningScriptId,
    loadScripts,
    upsertScript,
    removeScript,
    runScript,
    stopScript,
  } from "$lib/stores/scripts";
  import { activeTab, activeTabId, patchActiveTab } from "$lib/stores/tabs";
  import { onScriptError, type Script } from "$lib/ipc";

  // Use the Monaco type-only import so the type is erased at build time and
  // we still get static checking without forcing a sync import of the heavy package.
  type MonacoEditor = import("monaco-editor/esm/vs/editor/editor.api").editor.IStandaloneCodeEditor;

  let selected = $state<Script | null>(null);
  let editorEl: HTMLDivElement | undefined = $state();
  let editor: MonacoEditor | undefined;
  let saving = $state(false);
  let panelError = $state<string | null>(null);
  let unErr: (() => void) | null = null;

  onMount(async () => {
    await loadScripts();
    if ($scripts.length > 0) selected = $scripts[0];
    await mountEditor();
    unErr = await onScriptError((e) => { panelError = `Script error: ${String(e.payload)}`; });
  });

  onDestroy(() => {
    editor?.dispose();
    if (unErr) unErr();
  });

  async function mountEditor() {
    if (!editorEl) return;
    const { monaco } = await import("$lib/monaco");
    editor = monaco.editor.create(editorEl, {
      value: selected?.source ?? "-- write Lua here\n",
      language: "lua",
      theme: "zed-cool",
      fontFamily: "JetBrains Mono, Cascadia Code, monospace",
      fontSize: 12,
      minimap: { enabled: false },
      scrollBeyondLastLine: false,
      automaticLayout: true,
    });
  }

  $effect(() => {
    if (!editor || !selected) return;
    if (editor.getValue() !== selected.source) {
      editor.setValue(selected.source);
    }
  });

  async function newScript() {
    const id = `script_${Date.now()}`;
    const item: Script = { id, source: "-- log('hello from RustCOM Lua')\n" };
    await upsertScript(item);
    selected = item;
  }

  async function save() {
    if (!selected || !editor) return;
    panelError = null;
    saving = true;
    try {
      const item: Script = { id: selected.id, source: editor.getValue() };
      await upsertScript(item);
      selected = item;
    } catch (e) {
      panelError = `Save failed: ${String(e)}`;
    } finally {
      saving = false;
    }
  }

  async function rename(newId: string) {
    if (!selected) return;
    if (!newId.match(/^[a-zA-Z0-9_-]+$/)) {
      panelError = "Name must be letters/digits/underscore/hyphen only";
      return;
    }
    if (newId === selected.id) return;
    const newItem: Script = { id: newId, source: selected.source };
    await upsertScript(newItem);
    if (newId !== selected.id) {
      await removeScript(selected.id);
    }
    selected = newItem;
  }

  async function del() {
    if (!selected) return;
    await removeScript(selected.id);
    selected = $scripts[0] ?? null;
    editor?.setValue(selected?.source ?? "");
  }

  async function run() {
    if (!selected || !editor) return;
    if ($activeTab.state !== "connected") {
      patchActiveTab({ errorMessage: "Connect a port before running a script" });
      return;
    }
    panelError = null;
    try {
      await runScript($activeTabId, { id: selected.id, source: editor.getValue() });
    } catch (e) {
      panelError = `Run failed: ${String(e)}`;
    }
  }

  async function stop() {
    try { await stopScript(); } catch (e) { panelError = `Stop failed: ${String(e)}`; }
  }
</script>

<div class="panel">
  <div class="header">
    <h4>Scripts</h4>
    <button class="add" onclick={newScript}>+ New</button>
  </div>

  <div class="list">
    {#each $scripts as s (s.id)}
      <button class="item" class:active={selected?.id === s.id} onclick={() => (selected = s)}>
        {s.id}
        {#if $runningScriptId === s.id}<span class="dot ok"></span>{/if}
      </button>
    {/each}
    {#if $scripts.length === 0}
      <p class="empty">No scripts. Click <em>+ New</em>.</p>
    {/if}
  </div>

  {#if selected}
    <input
      class="rename"
      value={selected.id}
      onchange={(e) => rename((e.currentTarget as HTMLInputElement).value)}
    />
  {/if}

  <div class="editor" bind:this={editorEl}></div>

  {#if panelError}<p class="err">{panelError}</p>{/if}

  <div class="actions">
    <button class="primary" onclick={run} disabled={!selected || $activeTab.state !== "connected"}>Run</button>
    <button onclick={stop} disabled={$runningScriptId === null}>Stop</button>
    <button onclick={save} disabled={!selected || saving}>{saving ? "Saving…" : "Save"}</button>
    <button class="danger" onclick={del} disabled={!selected}>Delete</button>
  </div>
</div>

<style>
  .panel { padding: 12px; height: 100%; box-sizing: border-box; display: flex; flex-direction: column; gap: 8px; min-height: 0; }
  .header { display: flex; align-items: center; justify-content: space-between; }
  h4 { font-size: 10px; letter-spacing: 0.1em; text-transform: uppercase; color: var(--fg-subtle); margin: 0; font-weight: 600; }
  .add { padding: 4px 8px; font-size: 10px; }

  .list { display: flex; flex-direction: column; gap: 2px; max-height: 120px; overflow-y: auto; }
  .item { display: flex; align-items: center; justify-content: space-between; padding: 4px 8px; font-size: 11px; background: transparent; color: var(--fg-muted); border-radius: var(--radius-sm); text-align: left; }
  .item:hover { background: var(--bg-input); color: var(--fg); }
  .item.active { background: var(--bg-input-hover); color: var(--accent); }
  .dot { width: 6px; height: 6px; border-radius: 50%; background: var(--ok); }
  .empty { color: var(--fg-subtle); font-size: 11px; margin: 4px 0; }

  .rename { width: 100%; font-family: var(--font-mono); font-size: 11px; }
  .editor { flex: 1; min-height: 200px; border: 1px solid var(--border-strong); border-radius: var(--radius-md); overflow: hidden; }

  .err { color: var(--err); font-size: 11px; margin: 0; }

  .actions { display: flex; gap: 4px; flex-wrap: wrap; }
  .actions button { padding: 4px 10px; font-size: 10px; flex: 1 1 auto; min-width: 0; }
</style>
