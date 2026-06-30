<script lang="ts">
  import { goto } from "$app/navigation";
  import RaceControlShell from "$lib/components/layout/RaceControlShell.svelte";
  import TimingPanel from "$lib/components/racing/TimingPanel.svelte";
  import TimingRow from "$lib/components/racing/TimingRow.svelte";
  import TelemetryTile from "$lib/components/racing/TelemetryTile.svelte";
  import LaunchPanel from "$lib/components/host/LaunchPanel.svelte";
  import ResultsPanel from "$lib/components/host/ResultsPanel.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import { api } from "$lib/api";
  import type { LeagueSummary } from "$lib/types";
  import { Plus } from "@lucide/svelte";
  import { getContext, onMount } from "svelte";
  import type { AppStore } from "$lib/stores/app.svelte";

  const store = getContext<AppStore>("app");
  let leagues = $state<LeagueSummary[]>([]);
  let newLeagueName = $state("");
  let creating = $state(false);

  onMount(async () => {
    leagues = await api.listLeagues();
  });

  async function createLeague() {
    if (!newLeagueName.trim()) return;
    creating = true;
    try {
      const league = await api.createLeague(newLeagueName.trim());
      leagues = [league, ...leagues];
      newLeagueName = "";
    } finally {
      creating = false;
    }
  }

  async function logout() {
    await store.logout();
    goto("/login");
  }
</script>

{#snippet header()}
  <div class="flex flex-wrap items-end justify-between gap-4">
    <div>
      <p class="font-label text-[0.65rem] text-[var(--color-red)]">Race control · Host</p>
      <h1 class="font-display text-3xl leading-none text-white lg:text-4xl">
        PIT WALL <span class="text-[var(--color-dim)]">//</span>
        {store.state?.session?.personaname?.toUpperCase() ?? "CHIEF"}
      </h1>
    </div>
    <div class="flex items-center gap-2">
      <span class="status-led status-led--idle"></span>
      <span class="font-label text-[0.6rem] text-[var(--color-muted)]">Session control</span>
    </div>
  </div>
{/snippet}

{#if store.state?.session}
  <RaceControlShell
    mode="host"
    session={store.state.session}
    activeHref="/host"
    {header}
    onLogout={logout}
  >
    <div class="grid gap-4 xl:grid-cols-12">
      <div class="grid grid-cols-2 gap-3 sm:grid-cols-4 xl:col-span-12">
        <TelemetryTile label="Active leagues" value={leagues.length} accent="yellow" />
        <TelemetryTile label="Server state" value="LIVE" accent="dim" />
        <TelemetryTile label="Drivers online" value="—" accent="dim" />
        <TelemetryTile label="Next event" value="—" unit="min" accent="dim" />
      </div>

      <div class="xl:col-span-8">
        <TimingPanel
          title="Championship registry"
          subtitle="Active leagues on this pit wall"
          variant="tower"
        >
          {#snippet actions()}
            <div class="flex gap-2">
              <Input bind:value={newLeagueName} placeholder="New championship name…" class="w-48" />
              <Button variant="primary" size="sm" loading={creating} onclick={createLeague}>
                <Plus class="size-3.5" strokeWidth={2.5} />
                Register
              </Button>
            </div>
          {/snippet}

          <div class="panel-timing overflow-hidden rounded-md border border-[var(--color-line)]">
            <div
              class="timing-row bg-[var(--color-asphalt)] px-3 py-2 font-label text-[0.55rem] text-[var(--color-dim)]"
            >
              <span>POS</span>
              <span>CHAMPIONSHIP</span>
              <span class="hidden sm:block">ENTRANTS</span>
              <span class="hidden sm:block">STATUS</span>
            </div>
            {#each leagues as league, i (league.id)}
              <TimingRow
                position={i + 1}
                name={league.name}
                meta={`Registered ${new Date(league.createdAt).toLocaleDateString()}`}
                stat={String(league.memberCount)}
                statLabel="drivers"
                highlight={i === 0}
              />
            {:else}
              <div class="px-6 py-12 text-center">
                <p class="font-display text-2xl text-[var(--color-dim)]">NO CHAMPIONSHIPS</p>
                <p class="mt-2 text-sm text-[var(--color-muted)]">
                  Register your first league above — then invite drivers via Steam.
                </p>
              </div>
            {/each}
          </div>
        </TimingPanel>
      </div>

      <div class="xl:col-span-4">
        <LaunchPanel />
      </div>

      <div class="xl:col-span-12">
        <ResultsPanel />
      </div>
    </div>
  </RaceControlShell>
{/if}
