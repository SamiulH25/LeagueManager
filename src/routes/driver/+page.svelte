<script lang="ts">
  import { goto } from "$app/navigation";
  import AppShell from "$lib/components/layout/AppShell.svelte";
  import Avatar from "$lib/components/ui/Avatar.svelte";
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Card from "$lib/components/ui/Card.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import { api } from "$lib/api";
  import type { LeagueInvite } from "$lib/types";
  import { getContext, onMount } from "svelte";
  import type { AppStore } from "$lib/stores/app.svelte";

  const store = getContext<AppStore>("app");
  let invites = $state<LeagueInvite[]>([]);
  let hostAddress = $state("");

  const nav = [
    { href: "/driver", label: "Home", icon: "◉" },
    { href: "/driver/leagues", label: "My leagues", icon: "▦" },
    { href: "/driver/settings", label: "Settings", icon: "⚙" },
  ];

  onMount(async () => {
    const s = store.state;
    if (!s?.session) {
      goto("/login");
      return;
    }
    if (s.appMode !== "driver") {
      goto("/host");
      return;
    }
    invites = await api.listMyInvites();
    hostAddress = localStorage.getItem("lm_host_address") ?? "";
  });

  function saveHostAddress() {
    localStorage.setItem("lm_host_address", hostAddress);
  }

  async function logout() {
    await store.logout();
    goto("/login");
  }
</script>

{#if store.state?.session}
  <AppShell mode="driver" session={store.state.session} {nav} onLogout={logout}>
    <header class="mb-8">
      <Badge variant="live">Driver hub</Badge>
      <h1 class="mt-3 font-display text-3xl font-bold tracking-tight">Race ready</h1>
      <p class="mt-1 text-[var(--color-muted)]">
        Accept invites, follow standings, join through Content Manager.
      </p>
    </header>

    <div class="grid gap-6 lg:grid-cols-3">
      <Card class="lg:col-span-2" padding="lg">
        <h2 class="font-display text-xl font-semibold">Pending invites</h2>
        <p class="mt-1 text-sm text-[var(--color-muted)]">
          Leagues are invite-only — accept to see standings and join races.
        </p>

        <ul class="mt-6 space-y-3">
          {#each invites as invite (invite.id)}
            <li
              class="flex items-center gap-4 rounded-xl border border-[var(--color-carbon-border)] bg-[var(--color-carbon-elevated)] p-4"
            >
              {#if invite.hostAvatarUrl}
                <Avatar
                  src={invite.hostAvatarUrl}
                  alt={invite.hostPersonaname ?? "Host"}
                  size="md"
                />
              {/if}
              <div class="flex-1">
                <p class="font-medium">{invite.leagueName}</p>
                <p class="text-xs text-[var(--color-muted)]">
                  From {invite.hostPersonaname ?? "host"}
                </p>
              </div>
              <div class="flex gap-2">
                <Button variant="ghost" size="sm">Decline</Button>
                <Button variant="primary" size="sm">Accept</Button>
              </div>
            </li>
          {:else}
            <li
              class="rounded-xl border border-dashed border-[var(--color-carbon-border)] p-10 text-center"
            >
              <p class="text-sm text-[var(--color-muted)]">No pending invites</p>
              <p class="mt-1 text-xs text-[var(--color-muted)]">
                Ask your host to invite your Steam account.
              </p>
            </li>
          {/each}
        </ul>
      </Card>

      <Card padding="lg">
        <h3 class="font-display font-semibold">Host connection</h3>
        <p class="mt-1 text-xs text-[var(--color-muted)]">
          Your league host's public IP (Phase 2 sync).
        </p>
        <Input bind:value={hostAddress} placeholder="203.0.113.42" class="mt-4" />
        <Button variant="secondary" size="sm" class="mt-3 w-full" onclick={saveHostAddress}>
          Save address
        </Button>

        <div class="mt-6 rounded-xl bg-black/30 p-4">
          <Badge variant="muted">Coming Phase 2</Badge>
          <p class="mt-2 text-sm font-medium">Join race in Content Manager</p>
          <Button variant="primary" class="mt-3 w-full" disabled>Join live race</Button>
        </div>
      </Card>
    </div>
  </AppShell>
{/if}
