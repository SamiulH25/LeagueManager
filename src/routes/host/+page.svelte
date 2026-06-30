<script lang="ts">
  import { goto } from "$app/navigation";
  import AppShell from "$lib/components/layout/AppShell.svelte";
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Card from "$lib/components/ui/Card.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import { api } from "$lib/api";
  import type { LeagueSummary } from "$lib/types";
  import { getContext, onMount } from "svelte";
  import type { AppStore } from "$lib/stores/app.svelte";

  const store = getContext<AppStore>("app");
  let leagues = $state<LeagueSummary[]>([]);
  let newLeagueName = $state("");
  let creating = $state(false);

  const nav = [
    { href: "/host", label: "Overview", icon: "◉" },
    { href: "/host/leagues", label: "Leagues", icon: "▦" },
    { href: "/host/drivers", label: "Drivers", icon: "◎" },
    { href: "/host/settings", label: "Settings", icon: "⚙" },
  ];

  onMount(async () => {
    const s = store.state;
    if (!s?.session) {
      goto("/login");
      return;
    }
    if (s.appMode !== "host") {
      goto("/driver");
      return;
    }
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

{#if store.state?.session}
  <AppShell mode="host" session={store.state.session} {nav} onLogout={logout}>
    <header class="mb-8">
      <Badge variant="amber">Phase 0 · Host</Badge>
      <h1 class="mt-3 font-display text-3xl font-bold tracking-tight">
        Welcome back, {store.state.session.personaname}
      </h1>
      <p class="mt-1 text-[var(--color-muted)]">
        Your command center for leagues, servers, and results.
      </p>
    </header>

    <div class="grid gap-6 lg:grid-cols-3">
      <Card class="lg:col-span-2" padding="lg">
        <h2 class="font-display text-xl font-semibold">Your leagues</h2>
        <p class="mt-1 text-sm text-[var(--color-muted)]">
          Manage unlimited championships from one install.
        </p>

        <div class="mt-6 flex gap-3">
          <Input bind:value={newLeagueName} placeholder="New league name…" class="flex-1" />
          <Button variant="primary" loading={creating} onclick={createLeague}>Create</Button>
        </div>

        <ul class="mt-6 space-y-3">
          {#each leagues as league (league.id)}
            <li
              class="flex items-center justify-between rounded-xl border border-[var(--color-carbon-border)] bg-[var(--color-carbon-elevated)] px-4 py-3"
            >
              <div>
                <p class="font-medium">{league.name}</p>
                <p class="text-xs text-[var(--color-muted)]">
                  {league.memberCount} member{league.memberCount === 1 ? "" : "s"}
                </p>
              </div>
              <Badge variant="muted">Setup</Badge>
            </li>
          {:else}
            <li class="rounded-xl border border-dashed border-[var(--color-carbon-border)] p-8 text-center text-sm text-[var(--color-muted)]">
              No leagues yet — create your first one above.
            </li>
          {/each}
        </ul>
      </Card>

      <div class="space-y-6">
        <Card>
          <h3 class="font-display font-semibold">Server status</h3>
          <p class="mt-2 text-3xl font-bold text-[var(--color-muted)]">Idle</p>
          <p class="mt-1 text-xs text-[var(--color-muted)]">AssettoServer launch in Phase 1</p>
        </Card>
        <Card>
          <h3 class="font-display font-semibold">Next up</h3>
          <ul class="mt-3 space-y-2 text-sm text-[var(--color-muted)]">
            <li>→ Invite drivers via Steam</li>
            <li>→ Configure race weekend</li>
            <li>→ Launch server & share IP</li>
          </ul>
        </Card>
      </div>
    </div>
  </AppShell>
{/if}
