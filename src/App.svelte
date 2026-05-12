<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import TitleBar from "$lib/components/TitleBar.svelte";
  import ActivityBar, { type ActivityKey } from "$lib/components/ActivityBar.svelte";
  import StatusBar from "$lib/components/StatusBar.svelte";
  import TabStrip from "$lib/components/TabStrip.svelte";
  import Terminal from "$lib/components/Terminal.svelte";
  import SendRow from "$lib/components/SendRow.svelte";
  import ConnectionPanel from "$lib/components/panels/ConnectionPanel.svelte";
  import MacrosPanel from "$lib/components/panels/MacrosPanel.svelte";
  import FilterPanel from "$lib/components/panels/FilterPanel.svelte";
  import LogsPanel from "$lib/components/panels/LogsPanel.svelte";
  import ScriptsPanel from "$lib/components/panels/ScriptsPanel.svelte";
  import PlotPanel from "$lib/components/panels/PlotPanel.svelte";
  import PlotView from "$lib/components/PlotView.svelte";
  import ToastStack from "$lib/components/ToastStack.svelte";
  import { activeTab, activeTabId, patchActiveTab } from "$lib/stores/tabs";
  import { startPolling, stopPolling } from "$lib/stores/ports";
  import { startEventRouter, stopEventRouter } from "$lib/eventRouter";
  import { macros } from "$lib/stores/macros";
  import { loadSettings, persistSession, settings as settingsStore } from "$lib/stores/settings";
  import { restoreTabs, snapshotTabsForSession } from "$lib/stores/tabs";
  import { onToast } from "$lib/ipc";
  import { pushToast } from "$lib/stores/toasts";
  import { get } from "svelte/store";

  let active = $state<ActivityKey>("connection");
  let unToast: (() => void) | null = null;
  let unBeforeUnload: (() => void) | null = null;

  onMount(async () => {
    await loadSettings();
    const s = get(settingsStore);
    if (s.session_restore_enabled && s.last_session.tabs.length > 0) {
      restoreTabs(s.last_session.tabs.map((t) => t.config));
    }
    startPolling();
    startEventRouter();
    window.addEventListener("keydown", onWindowKey);
    unToast = await onToast((e) => {
      pushToast(e.payload.level, e.payload.msg, e.payload.ts);
    });
    const handler = () => {
      if (get(settingsStore).session_restore_enabled) {
        const snap = { tabs: snapshotTabsForSession().map((c) => ({ config: c })) };
        void persistSession(snap);
      }
    };
    window.addEventListener("beforeunload", handler);
    unBeforeUnload = () => window.removeEventListener("beforeunload", handler);
  });
  onDestroy(() => {
    if (unBeforeUnload) unBeforeUnload();
    if (unToast) unToast();
    window.removeEventListener("keydown", onWindowKey);
    stopEventRouter();
    stopPolling();
  });

  function dataLabel(d: string) {
    return ({ five: "5", six: "6", seven: "7", eight: "8" } as const)[d as "five" | "six" | "seven" | "eight"];
  }

  function stopLabel(s: string) {
    return ({ one: "1", two: "2" } as const)[s as "one" | "two"];
  }

  function parityLabel(p: string) {
    return ({ none: "N", even: "E", odd: "O" } as const)[p as "none" | "even" | "odd"];
  }

  function onWindowKey(e: KeyboardEvent) {
    // Ctrl+F (or Cmd+F on macOS) toggles the search overlay for the active tab.
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "f") {
      e.preventDefault();
      const open = !get(activeTab).searchOpen;
      patchActiveTab({ searchOpen: open, searchMatchIndex: -1 });
      return;
    }

    // F1–F12 trigger macros if assigned. Skip when focus is in an input/textarea
    // so users can still rebind in OS-controlled forms etc.
    if (!/^F([1-9]|1[0-2])$/.test(e.key)) return;
    const tag = (e.target as HTMLElement | null)?.tagName?.toLowerCase();
    if (tag === "input" || tag === "textarea" || tag === "select") return;

    const m = get(macros).find((x) => x.hotkey === e.key);
    if (!m) return;
    e.preventDefault();
    void runMacro(m);
  }

  async function runMacro(m: { mode: "ascii" | "hex"; payload: string; line_ending?: import("$lib/ipc").LineEnding | null }) {
    const tab = get(activeTab);
    if (tab.state !== "connected") return;
    const { ipc } = await import("$lib/ipc");
    const id = get(activeTabId);
    if (m.mode === "ascii") {
      await ipc.sendText(id, m.payload, (m.line_ending ?? "none"));
    } else {
      await ipc.sendHex(id, m.payload);
    }
  }
</script>

<div class="app">
  <TitleBar connected={$activeTab.state === "connected"} />
  <div class="body">
    <ActivityBar bind:active />
    <aside class="side">
      {#if active === "connection"}
        <ConnectionPanel />
      {:else if active === "macros"}
        <MacrosPanel />
      {:else if active === "filter"}
        <FilterPanel />
      {:else if active === "logs"}
        <LogsPanel />
      {:else if active === "scripts"}
        <ScriptsPanel />
      {:else if active === "plot"}
        <PlotPanel />
      {/if}
    </aside>
    <main class="main">
      <TabStrip />
      {#if active === "plot"}
        <PlotView />
      {:else}
        <Terminal />
        <SendRow />
      {/if}
    </main>
  </div>
  <StatusBar
    connected={$activeTab.state === "connected"}
    portName={$activeTab.config.port_name}
    rxBytes={$activeTab.rxBytes}
    txBytes={$activeTab.txBytes}
    settingsLabel={`${$activeTab.config.baud_rate} ${dataLabel($activeTab.config.data_bits)}-${parityLabel($activeTab.config.parity)}-${stopLabel($activeTab.config.stop_bits)}`}
  />
  <ToastStack />
</div>

<style>
  .app { display: grid; grid-template-rows: var(--titlebar-h) 1fr var(--statusbar-h); height: 100vh; }
  .body { display: grid; grid-template-columns: var(--activity-bar-w) var(--side-panel-w) 1fr; min-height: 0; }
  .side { background: var(--bg); border-right: 1px solid var(--border); overflow: hidden; }
  .main { display: flex; flex-direction: column; min-height: 0; background: var(--bg); }
</style>
