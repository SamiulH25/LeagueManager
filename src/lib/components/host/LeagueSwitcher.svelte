<script lang="ts">
  import { api } from "$lib/api";
  import type { ActiveLeague, LeagueSummary } from "$lib/types";
  import { onMount } from "svelte";

  interface Props {
    onChange?: (league: ActiveLeague) => void;
  }

  let { onChange }: Props = $props();
  let leagues = $state<LeagueSummary[]>([]);
  let active = $state<ActiveLeague | null>(null);
  let switching = $state(false);

  onMount(async () => {
    leagues = await api.listLeagues();
    active = await api.getActiveLeague();
  });

  async function select(leagueId: number) {
    if (active?.id === leagueId) return;
    switching = true;
    try {
      active = await api.setActiveLeague(leagueId);
      onChange?.(active);
    } finally {
      switching = false;
    }
  }
</script>

{#if leagues.length > 1}
  <div class="flex flex-wrap items-center gap-2">
    <span class="font-label text-[0.55rem] text-[var(--color-dim)]">Active championship</span>
    {#each leagues as league (league.id)}
      <button
        type="button"
        disabled={switching}
        class="rounded-md px-2.5 py-1 font-label text-[0.65rem] tracking-wider transition-colors {active?.id ===
        league.id
          ? 'bg-[color-mix(in_srgb,var(--color-red)_20%,transparent)] text-white'
          : 'bg-[var(--color-asphalt)] text-[var(--color-muted)] hover:text-white'}"
        onclick={() => select(league.id)}
      >
        {league.name}
      </button>
    {/each}
  </div>
{:else if active}
  <p class="font-label text-[0.6rem] text-[var(--color-dim)]">
    Active · <span class="text-white">{active.name}</span>
  </p>
{/if}
