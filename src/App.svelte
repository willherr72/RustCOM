<script lang="ts">
  import TitleBar from "$lib/components/TitleBar.svelte";
  import ActivityBar, { type ActivityKey } from "$lib/components/ActivityBar.svelte";
  import StatusBar from "$lib/components/StatusBar.svelte";
  import TabStrip from "$lib/components/TabStrip.svelte";
  import Terminal from "$lib/components/Terminal.svelte";
  import SendRow from "$lib/components/SendRow.svelte";
  import ConnectionPanel from "$lib/components/panels/ConnectionPanel.svelte";

  let active = $state<ActivityKey>("connection");
</script>

<div class="app">
  <TitleBar />
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
      <TabStrip />
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
