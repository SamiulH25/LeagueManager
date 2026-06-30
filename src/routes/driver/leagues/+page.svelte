<script lang="ts">
  import { goto } from "$app/navigation";
  import RaceControlShell from "$lib/components/layout/RaceControlShell.svelte";
  import TimingPanel from "$lib/components/racing/TimingPanel.svelte";
  import TimingRow from "$lib/components/racing/TimingRow.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import { api } from "$lib/api";
  import { loadPitLink } from "$lib/pit-link";
  import type { StandingsResponse } from "$lib/types";
  import { RefreshCw } from "@lucide/svelte";
  import { getContext, onDestroy, onMount } from "svelte";
  import type { AppStore } from "$lib/stores/app.svelte";

  const store = getContext<AppStore>("app");
  let standings = $state<StandingsResponse | null>(null);
  let error = $state<string | null>(null);
  let loading = $state(false);
  let pollTimer: ReturnType<typeof setInterval> | undefined;
  const pitLink = $derived(loadPitLink());

  async function loadStandings() {
    if (!pitLink.host.trim()) {
      error = "Configure your pit link in Garage first.";
      standings = null;
      return;
    }
    loading = true;
    error = null;
    try {
      standings = await api.fetchRemoteStandings(pitLink.host, pitLink.port, 1);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
      standings = null;
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    loadStandings();
    pollTimer = setInterval(loadStandings, 15000);
  });

  onDestroy(() => {
    if (pollTimer) clearInterval(pollTimer);
  });

  async function logout() {
    await store.logout();
    goto("/login");
  }
</script>

{#snippet header()}
  <div class="flex flex-wrap items-end justify-between gap-4">
    <div>
      <p class="font-label text-[0.65rem] text-[var(--color-green)]">Standings</p>
      <h1 class="font-display text-3xl leading-none text-white lg:text-4xl">CHAMPIONSHIP TABLE</h1>
    </div>
    <Button variant="secondary" size="sm" loading={loading} onclick={loadStandings}>
      <RefreshCw class="size-3.5" strokeWidth={2} />
      Refresh
    </Button>
  </div>
{/snippet}

{#if store.state?.session}
  <RaceControlShell
    mode="driver"
    session={store.state.session}
    activeHref="/driver/leagues"
    {header}
    onLogout={logout}
  >
    <TimingPanel
      title={standings?.championshipName ?? "Championship standings"}
      subtitle="Synced from host pit link"
    >
      {#if error}
        <p class="py-8 text-center text-sm text-[var(--color-red)]">{error}</p>
      {:else if standings && standings.rows.length > 0}
        <div class="panel-timing overflow-hidden rounded-md border border-[var(--color-line)]">
          <div
            class="timing-row bg-[var(--color-asphalt)] px-3 py-2 font-label text-[0.55rem] text-[var(--color-dim)]"
          >
            <span>POS</span>
            <span>DRIVER</span>
            <span class="hidden sm:block">TEAM</span>
            <span>PTS</span>
          </div>
          {#each standings.rows as row (row.position)}
            <TimingRow
              position={row.position}
              name={row.driverName}
              meta={row.team ?? ""}
              stat={String(row.points)}
              statLabel="pts"
              highlight={row.position === 1}
            />
          {/each}
        </div>
      {:else if loading}
        <p class="py-8 text-center text-sm text-[var(--color-muted)]">Loading standings…</p>
      {:else}
        <p class="py-8 text-center text-sm text-[var(--color-muted)]">
          No standings available from host.
        </p>
      {/if}
    </TimingPanel>
  </RaceControlShell>
{/if}
