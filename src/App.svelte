<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import TitleBar from "$lib/components/TitleBar.svelte";
  import ActivityBar, { type ActivityKey } from "$lib/components/ActivityBar.svelte";
  import StatusBar from "$lib/components/StatusBar.svelte";
  import TabStrip from "$lib/components/TabStrip.svelte";
  import Terminal from "$lib/components/Terminal.svelte";
  import SendRow from "$lib/components/SendRow.svelte";
  import ConnectionPanel from "$lib/components/panels/ConnectionPanel.svelte";
  import { activeTab } from "$lib/stores/tabs";
  import { startPolling, stopPolling } from "$lib/stores/ports";

  let active = $state<ActivityKey>("connection");

  onMount(() => startPolling());
  onDestroy(() => stopPolling());
</script>

<div class="app">
  <TitleBar connected={$activeTab.state === "connected"} />
  <div class="body">
    <ActivityBar bind:active />
    <aside class="side">
      {#if active === "connection"}
        <ConnectionPanel />
      {:else}
        <div class="muted-pad">Coming in a later plan.</div>
      {/if}
    </aside>
    <main class="main">
      <TabStrip
        label={$activeTab.config.port_name ? $activeTab.config.port_name : "COM —"}
        active
      />
      <Terminal />
      <SendRow />
    </main>
  </div>
  <StatusBar />
</div>

<style>
  .app { display: grid; grid-template-rows: var(--titlebar-h) 1fr var(--statusbar-h); height: 100vh; }
  .body { display: grid; grid-template-columns: var(--activity-bar-w) var(--side-panel-w) 1fr; min-height: 0; }
  .side { background: var(--bg); border-right: 1px solid var(--border); overflow: hidden; }
  .main { display: flex; flex-direction: column; min-height: 0; background: var(--bg); }
  .muted-pad { padding: 14px; color: var(--fg-subtle); font-size: 11px; }
</style>
