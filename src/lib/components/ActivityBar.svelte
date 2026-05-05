<script lang="ts">
  export type ActivityKey = "connection" | "macros" | "scripts" | "filter" | "logs";

  let {
    active = $bindable<ActivityKey>("connection"),
  }: { active?: ActivityKey } = $props();

  const items: Array<{ key: ActivityKey; icon: string; title: string }> = [
    { key: "connection", icon: "⚙", title: "Connection" },
    { key: "macros",     icon: "▶", title: "Macros" },
    { key: "scripts",    icon: "{}", title: "Scripts (Plan 3)" },
    { key: "filter",     icon: "⌕", title: "Filter (Plan 2)" },
    { key: "logs",       icon: "≡", title: "Logs (Plan 2)" },
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
      {item.icon}
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
  .ico:hover { background: var(--bg-input); color: var(--fg); }
  .ico.active {
    background: var(--bg-input);
    color: var(--accent);
    box-shadow: inset 2px 0 0 var(--accent);
  }
</style>
