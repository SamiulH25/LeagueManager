<script lang="ts">
  import { goto } from "$app/navigation";
  import RaceControlShell from "$lib/components/layout/RaceControlShell.svelte";
  import LeagueSwitcher from "$lib/components/host/LeagueSwitcher.svelte";
  import TimingPanel from "$lib/components/racing/TimingPanel.svelte";
  import TimingRow from "$lib/components/racing/TimingRow.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import { api } from "$lib/api";
  import type { ActiveLeague, ChampionshipRound, LeagueSummary } from "$lib/types";
  import { Download, Plus } from "@lucide/svelte";
  import { getContext, onMount } from "svelte";
  import type { AppStore } from "$lib/stores/app.svelte";

  const store = getContext<AppStore>("app");
  let leagues = $state<LeagueSummary[]>([]);
  let activeLeague = $state<ActiveLeague | null>(null);
  let rounds = $state<ChampionshipRound[]>([]);
  let newLeagueName = $state("");
  let creating = $state(false);
  let exporting = $state(false);

  onMount(async () => {
    leagues = await api.listLeagues();
    activeLeague = await api.getActiveLeague();
    await loadRounds();
  });

  async function loadRounds() {
    if (!activeLeague) return;
    rounds = await api.listChampionshipRounds(activeLeague.id);
  }

  async function onLeagueChange(league: ActiveLeague) {
    activeLeague = league;
    await loadRounds();
  }

  async function createLeague() {
    if (!newLeagueName.trim()) return;
    creating = true;
    try {
      const league = await api.createLeague(newLeagueName.trim());
      leagues = [league, ...leagues];
      activeLeague = await api.setActiveLeague(league.id);
      newLeagueName = "";
      await loadRounds();
    } finally {
      creating = false;
    }
  }

  async function exportCsv() {
    exporting = true;
    try {
      const csv = await api.exportStandingsCsv();
      await navigator.clipboard.writeText(csv);
    } finally {
      exporting = false;
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
      <p class="font-label text-[0.65rem] text-[var(--color-red)]">Championships</p>
      <h1 class="font-display text-3xl leading-none text-white lg:text-4xl">CHAMPIONSHIP REGISTRY</h1>
    </div>
    <LeagueSwitcher onChange={onLeagueChange} />
  </div>
{/snippet}

{#if store.state?.session}
  <RaceControlShell
    mode="host"
    session={store.state.session}
    activeHref="/host/leagues"
    {header}
    onLogout={logout}
  >
    <div class="grid gap-4 lg:grid-cols-12">
      <div class="lg:col-span-7">
        <TimingPanel title="All championships" subtitle="Manage your leagues">
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
            {#each leagues as league, i (league.id)}
              <TimingRow
                position={i + 1}
                name={league.name}
                meta={`Created ${new Date(league.createdAt).toLocaleDateString()}`}
                stat={String(league.memberCount)}
                statLabel="drivers"
                highlight={activeLeague?.id === league.id}
              />
            {:else}
              <div class="px-6 py-12 text-center text-sm text-[var(--color-muted)]">
                No championships yet. Register one above.
              </div>
            {/each}
          </div>
        </TimingPanel>
      </div>

      <div class="lg:col-span-5">
        <TimingPanel title="Season rounds" subtitle="Multi-round championship calendar">
          {#snippet actions()}
            <Button variant="secondary" size="sm" loading={exporting} onclick={exportCsv}>
              <Download class="size-3.5" strokeWidth={2} />
              Export CSV
            </Button>
          {/snippet}

          {#if rounds.length > 0}
            <ul class="space-y-2">
              {#each rounds as round (round.id)}
                <li class="rounded-md bg-[var(--color-asphalt)] px-3 py-2">
                  <div class="flex items-center justify-between gap-2">
                    <span class="font-label text-[0.6rem] text-[var(--color-yellow)]">
                      R{round.roundNumber}
                    </span>
                    <span
                      class="font-mono text-[0.55rem] uppercase {round.status === 'completed'
                        ? 'text-[var(--color-green)]'
                        : round.status === 'live'
                          ? 'text-[var(--color-red)]'
                          : 'text-[var(--color-dim)]'}"
                    >
                      {round.status}
                    </span>
                  </div>
                  <p class="mt-1 text-sm text-white">{round.name}</p>
                  <p class="font-mono text-xs text-[var(--color-muted)]">{round.track}</p>
                </li>
              {/each}
            </ul>
          {:else}
            <p class="py-8 text-center text-sm text-[var(--color-muted)]">
              Rounds are created when you launch a server from Race Control.
            </p>
          {/if}
        </TimingPanel>
      </div>
    </div>
  </RaceControlShell>
{/if}
