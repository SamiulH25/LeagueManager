<script lang="ts">
  import { goto } from "$app/navigation";
  import RaceControlShell from "$lib/components/layout/RaceControlShell.svelte";
  import TimingPanel from "$lib/components/racing/TimingPanel.svelte";
  import TimingRow from "$lib/components/racing/TimingRow.svelte";
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
  <div>
    <p class="font-label text-[0.65rem] text-[var(--color-red)]">Championships</p>
    <h1 class="font-display text-3xl leading-none text-white lg:text-4xl">CHAMPIONSHIP REGISTRY</h1>
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
    <TimingPanel title="All championships" subtitle="Manage your leagues">
      {#snippet actions()}
        <div class="flex gap-2">
          <Input bind:value={newLeagueName} placeholder="New championship name…" class="w-56" />
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
            highlight={i === 0}
          />
        {:else}
          <div class="px-6 py-12 text-center text-sm text-[var(--color-muted)]">
            No championships yet. Register one above.
          </div>
        {/each}
      </div>
    </TimingPanel>
  </RaceControlShell>
{/if}
