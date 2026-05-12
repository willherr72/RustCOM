<script lang="ts">
  export type ActivityKey = "connection" | "macros" | "scripts" | "filter" | "logs" | "plot";

  let {
    active = $bindable<ActivityKey>("connection"),
  }: { active?: ActivityKey } = $props();

  const items: Array<{ key: ActivityKey; icon: string; title: string }> = [
    { key: "connection", icon: "⚙", title: "Connection" },
    { key: "macros",     icon: "▶", title: "Macros" },
    { key: "scripts",    icon: "{}", title: "Scripts" },
    { key: "filter",     icon: "⌕", title: "Filter" },
    { key: "logs",       icon: "≡", title: "Logs" },
    { key: "plot",       icon: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 3 3 21 21 21"/><polyline points="6 17 10 11 14 14 21 6"/></svg>', title: "Plot" },
  ];
</script>

<nav>
  {#each items as item (item.key)}
    <button
      class="ico"
      class:active={active === item.key}
      title={item.title}
      onclick={() => (active = item.key)}
    >
      {@html item.icon}
    </button>
  {/each}
</nav>

<style>
  nav {
    width: var(--activity-bar-w);
    background: var(--bg-titlebar);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 8px 0;
    gap: 4px;
  }
  .ico {
    width: 32px;
    height: 32px;
    background: transparent;
    color: var(--fg-subtle);
    font-size: 14px;
    border-radius: var(--radius-md);
    padding: 0;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .ico :global(svg) { width: 18px; height: 18px; display: block; }
  .ico:hover { background: var(--bg-input); color: var(--fg); }
  .ico.active {
    background: var(--bg-input);
    color: var(--accent);
    box-shadow: inset 2px 0 0 var(--accent);
  }
</style>
