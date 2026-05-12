<script lang="ts">
  import { ipc, type DataBits, type FlowControl, type Parity, type StopBits } from "$lib/ipc";
  import { availablePorts } from "$lib/stores/ports";
  import { activeTab, activeTabId, patchActiveTab } from "$lib/stores/tabs";
  import { persistDefaults, settings, persistSessionRestore } from "$lib/stores/settings";

  const BAUD_RATES = [300, 1200, 2400, 4800, 9600, 19200, 38400, 57600, 115200, 230400, 460800, 921600];

  let connecting = $state(false);

  async function persistAsDefault() {
    const c = $activeTab.config;
    await persistDefaults({
      baud_rate: c.baud_rate,
      data_bits: c.data_bits,
      stop_bits: c.stop_bits,
      parity: c.parity,
      flow_control: c.flow_control,
      line_ending: $activeTab.lineEnding,
      send_mode: $activeTab.sendMode,
      auto_reconnect: c.auto_reconnect ?? false,
      reconnect_delay_ms: c.reconnect_delay_ms ?? 2000,
    });
  }

  async function connect() {
    const tab = $activeTab;
    if (!tab.config.port_name) {
      patchActiveTab({ errorMessage: "Pick a port first." });
      return;
    }
    connecting = true;
    patchActiveTab({ state: "connecting", errorMessage: null });
    try {
      await ipc.openPort($activeTabId, tab.config);
      patchActiveTab({ state: "connected" });
    } catch (e) {
      patchActiveTab({ state: "disconnected", errorMessage: String(e) });
    } finally {
      connecting = false;
    }
  }

  async function disconnect() {
    try {
      await ipc.closePort($activeTabId);
    } catch (e) {
      patchActiveTab({ errorMessage: String(e) });
    }
    patchActiveTab({ state: "disconnected", dtr: false, rts: false });
  }

  function setBaud(value: string) {
    const n = Number.parseInt(value, 10);
    if (Number.isFinite(n) && n > 0) {
      patchActiveTab({ config: { ...$activeTab.config, baud_rate: n } });
    }
  }

  async function toggleDtr() {
    const next = !$activeTab.dtr;
    try {
      await ipc.setDtr($activeTabId, next);
      patchActiveTab({ dtr: next });
    } catch (e) {
      patchActiveTab({ errorMessage: String(e) });
    }
  }

  async function toggleRts() {
    const next = !$activeTab.rts;
    try {
      await ipc.setRts($activeTabId, next);
      patchActiveTab({ rts: next });
    } catch (e) {
      patchActiveTab({ errorMessage: String(e) });
    }
  }

  function patchConfig<K extends keyof typeof $activeTab.config>(key: K, value: (typeof $activeTab.config)[K]) {
    patchActiveTab({ config: { ...$activeTab.config, [key]: value } });
  }
</script>

<div class="panel">
  <h4>Connection</h4>

  <label class="lbl">
    Port
    <select
      value={$activeTab.config.port_name}
      onchange={(e) => patchConfig("port_name", (e.currentTarget as HTMLSelectElement).value)}
      disabled={$activeTab.state === "connected"}
    >
      <option value="" disabled>Select port…</option>
      {#each $availablePorts as p (p.name)}
        <option value={p.name}>{p.name} {p.kind ? `(${p.kind})` : ""}</option>
      {/each}
    </select>
  </label>

  <label class="lbl">
    Baud
    <select
      value={$activeTab.config.baud_rate.toString()}
      onchange={(e) => setBaud((e.currentTarget as HTMLSelectElement).value)}
      disabled={$activeTab.state === "connected"}
    >
      {#each BAUD_RATES as b}
        <option value={b.toString()}>{b}</option>
      {/each}
    </select>
  </label>

  <div class="row2">
    <div>
      <label class="lbl">
        Data
        <select
          value={$activeTab.config.data_bits}
          onchange={(e) => patchConfig("data_bits", (e.currentTarget as HTMLSelectElement).value as DataBits)}
          disabled={$activeTab.state === "connected"}
        >
          <option value="five">5</option>
          <option value="six">6</option>
          <option value="seven">7</option>
          <option value="eight">8</option>
        </select>
      </label>
    </div>
    <div>
      <label class="lbl">
        Stop
        <select
          value={$activeTab.config.stop_bits}
          onchange={(e) => patchConfig("stop_bits", (e.currentTarget as HTMLSelectElement).value as StopBits)}
          disabled={$activeTab.state === "connected"}
        >
          <option value="one">1</option>
          <option value="two">2</option>
        </select>
      </label>
    </div>
  </div>

  <label class="lbl">
    Parity
    <select
      value={$activeTab.config.parity}
      onchange={(e) => patchConfig("parity", (e.currentTarget as HTMLSelectElement).value as Parity)}
      disabled={$activeTab.state === "connected"}
    >
      <option value="none">None</option>
      <option value="even">Even</option>
      <option value="odd">Odd</option>
    </select>
  </label>

  <label class="lbl">
    Flow
    <select
      value={$activeTab.config.flow_control}
      onchange={(e) => patchConfig("flow_control", (e.currentTarget as HTMLSelectElement).value as FlowControl)}
      disabled={$activeTab.state === "connected"}
    >
      <option value="none">None</option>
      <option value="software">Software</option>
      <option value="hardware">Hardware</option>
    </select>
  </label>

  <label class="toggle">
    <input
      type="checkbox"
      checked={$activeTab.config.auto_reconnect ?? false}
      onchange={(e) => patchActiveTab({
        config: { ...$activeTab.config, auto_reconnect: (e.currentTarget as HTMLInputElement).checked },
      })}
      disabled={$activeTab.state === "connected"}
    />
    Auto-reconnect
  </label>

  {#if $activeTab.state === "connected"}
    <button class="primary danger" onclick={disconnect}>Disconnect</button>
  {:else}
    <button class="primary" onclick={connect} disabled={connecting}>
      {connecting ? "Connecting…" : "Connect"}
    </button>
  {/if}

  <button
    onclick={persistAsDefault}
    title="Save current settings as defaults for new tabs"
    class="save-default"
  >
    Save as default
  </button>

  {#if $activeTab.errorMessage}
    <p class="err">{$activeTab.errorMessage}</p>
  {/if}

  {#if $activeTab.state === "connected"}
    <h4 class="signals">Signals</h4>
    <div class="signals-row">
      <button class:active={$activeTab.dtr} onclick={toggleDtr}>DTR</button>
      <button class:active={$activeTab.rts} onclick={toggleRts}>RTS</button>
    </div>
  {/if}

  <h4 class="signals">Session</h4>
  <label class="toggle">
    <input
      type="checkbox"
      checked={$settings.session_restore_enabled}
      onchange={(e) => persistSessionRestore((e.currentTarget as HTMLInputElement).checked)}
    />
    Restore tabs on next launch
  </label>
</div>

<style>
  .panel { padding: 12px; height: 100%; box-sizing: border-box; display: flex; flex-direction: column; gap: 6px; overflow: auto; }
  h4 { font-size: 10px; letter-spacing: 0.1em; text-transform: uppercase; color: var(--fg-subtle); margin: 0; font-weight: 600; }
  h4.signals { margin-top: 14px; }
  .lbl { font-size: 9px; letter-spacing: 0.06em; text-transform: uppercase; color: var(--fg-subtle); display: flex; flex-direction: column; gap: 4px; }
  select { width: 100%; }
  .row2 { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; }
  button.primary { width: 100%; margin-top: 8px; padding: 8px 0; }
  .save-default { width: 100%; margin-top: 4px; padding: 4px 0; font-size: 10px; }
  .signals-row { display: flex; gap: 6px; }
  .signals-row button { flex: 1; }
  .signals-row button.active { background: var(--ok-bg); color: var(--ok); }
  .err { color: var(--err); font-size: 11px; margin: 4px 0 0; }
  .toggle { display: flex; align-items: center; gap: 6px; font-size: 11px; color: var(--fg-muted); margin-top: 6px; }
</style>
