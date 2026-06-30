<script lang="ts">
  import { goto } from "$app/navigation";
  import RaceControlShell from "$lib/components/layout/RaceControlShell.svelte";
  import TimingPanel from "$lib/components/racing/TimingPanel.svelte";
  import TimingRow from "$lib/components/racing/TimingRow.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import { api } from "$lib/api";
  import { loadPitLink, savePitLink } from "$lib/pit-link";
  import type { PitLinkTestResult } from "$lib/types";
  import { Link, Wifi } from "@lucide/svelte";
  import { getContext, onMount } from "svelte";
  import type { AppStore } from "$lib/stores/app.svelte";

  const store = getContext<AppStore>("app");
  let host = $state("");
  let syncPort = $state("9847");
  let testing = $state(false);
  let testResult = $state<PitLinkTestResult | null>(null);
  let saved = $state(false);

  onMount(() => {
    const link = loadPitLink();
    host = link.host;
    syncPort = String(link.port);
  });

  function save() {
    savePitLink({ host, port: Number(syncPort) || 9847 });
    saved = true;
    setTimeout(() => (saved = false), 2000);
  }

  async function testConnection() {
    if (!host.trim()) return;
    testing = true;
    testResult = null;
    try {
      testResult = await api.testPitLink(host.trim(), Number(syncPort) || 9847);
    } finally {
      testing = false;
    }
  }

  async function logout() {
    await store.logout();
    goto("/login");
  }
</script>

{#snippet header()}
  <div>
    <p class="font-label text-[0.65rem] text-[var(--color-green)]">Garage</p>
    <h1 class="font-display text-3xl leading-none text-white lg:text-4xl">DRIVER SETTINGS</h1>
  </div>
{/snippet}

{#if store.state?.session}
  <RaceControlShell
    mode="driver"
    session={store.state.session}
    activeHref="/driver/settings"
    {header}
    onLogout={logout}
  >
    <div class="mx-auto max-w-lg space-y-4">
      <TimingPanel title="Pit link" subtitle="Your league host's address">
        <p class="mb-3 text-xs text-[var(--color-muted)]">
          Enter the public IP and sync port your race chief shared. League data syncs over TCP on
          this port (default 9847).
        </p>
        <div class="space-y-3">
          <div>
            <p class="mb-1 font-label text-[0.6rem] text-[var(--color-dim)]">Host IP / hostname</p>
            <Input bind:value={host} placeholder="203.0.113.42" mono />
          </div>
          <div>
            <p class="mb-1 font-label text-[0.6rem] text-[var(--color-dim)]">Sync port</p>
            <Input bind:value={syncPort} type="number" placeholder="9847" mono />
          </div>
        </div>

        <div class="mt-4 flex gap-2">
          <Button variant="secondary" size="sm" class="flex-1" onclick={save}>
            {saved ? "Saved!" : "Save pit link"}
          </Button>
          <Button
            variant="primary"
            size="sm"
            class="flex-1"
            loading={testing}
            disabled={!host.trim()}
            onclick={testConnection}
          >
            <Wifi class="size-3.5" strokeWidth={2} />
            Test connection
          </Button>
        </div>

        {#if testResult}
          <div
            class="mt-4 rounded-md border p-3 text-sm {testResult.connected
              ? 'border-[color-mix(in_srgb,var(--color-green)_30%,transparent)] bg-[color-mix(in_srgb,var(--color-green)_6%,transparent)]'
              : 'border-[color-mix(in_srgb,var(--color-red)_30%,transparent)] bg-[color-mix(in_srgb,var(--color-red)_6%,transparent)]'}"
          >
            <p class="font-label text-[0.6rem] {testResult.connected ? 'text-[var(--color-green)]' : 'text-[var(--color-red)]'}">
              {testResult.connected ? "Connected" : "Failed"}
            </p>
            <p class="mt-1 text-[var(--color-muted)]">{testResult.message}</p>
            {#if testResult.connected}
              <p class="mt-1 font-mono text-xs text-[var(--color-dim)]">
                {testResult.latencyMs}ms · v{testResult.version ?? "?"}
              </p>
            {/if}
          </div>
        {/if}
      </TimingPanel>

      <TimingPanel title="How it works" subtitle="Phase 2 sync">
        <ol class="space-y-2 text-sm text-[var(--color-muted)]">
          <li class="flex gap-2">
            <Link class="mt-0.5 size-4 shrink-0 text-[var(--color-dim)]" strokeWidth={1.75} />
            Host runs LeagueManager in Race Control mode with the league API on the sync port.
          </li>
          <li class="flex gap-2">
            <Link class="mt-0.5 size-4 shrink-0 text-[var(--color-dim)]" strokeWidth={1.75} />
            You connect here; Paddock shows live races and Standings pulls points tables.
          </li>
        </ol>
      </TimingPanel>
    </div>
  </RaceControlShell>
{/if}
