<script lang="ts">
  import { goto } from "$app/navigation";
  import RaceControlShell from "$lib/components/layout/RaceControlShell.svelte";
  import LeagueSwitcher from "$lib/components/host/LeagueSwitcher.svelte";
  import TimingPanel from "$lib/components/racing/TimingPanel.svelte";
  import TimingRow from "$lib/components/racing/TimingRow.svelte";
  import Avatar from "$lib/components/ui/Avatar.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import { api } from "$lib/api";
  import type { ActiveLeague, LeagueRoster } from "$lib/types";
  import { RefreshCw, UserPlus, X } from "@lucide/svelte";
  import { getContext, onMount } from "svelte";
  import type { AppStore } from "$lib/stores/app.svelte";

  const store = getContext<AppStore>("app");
  let activeLeague = $state<ActiveLeague | null>(null);
  let roster = $state<LeagueRoster | null>(null);
  let steamInput = $state("");
  let inviting = $state(false);
  let refreshing = $state(false);
  let error = $state<string | null>(null);
  let message = $state<string | null>(null);

  async function loadRoster() {
    if (!activeLeague) return;
    roster = await api.listLeagueRoster(activeLeague.id);
  }

  onMount(async () => {
    activeLeague = await api.getActiveLeague();
    await loadRoster();
  });

  async function onLeagueChange(league: ActiveLeague) {
    activeLeague = league;
    await loadRoster();
  }

  async function sendInvite() {
    if (!activeLeague || !steamInput.trim()) return;
    inviting = true;
    error = null;
    message = null;
    try {
      await api.sendDriverInvite(activeLeague.id, steamInput.trim());
      steamInput = "";
      message = "Invite sent.";
      await loadRoster();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      inviting = false;
    }
  }

  async function refreshAvatars() {
    if (!activeLeague) return;
    refreshing = true;
    try {
      await api.refreshRosterAvatars(activeLeague.id);
      await loadRoster();
    } finally {
      refreshing = false;
    }
  }

  async function revokeInvite(id: number) {
    await api.revokeDriverInvite(id);
    await loadRoster();
  }

  async function logout() {
    await store.logout();
    goto("/login");
  }
</script>

{#snippet header()}
  <div class="flex flex-wrap items-end justify-between gap-4">
    <div>
      <p class="font-label text-[0.65rem] text-[var(--color-red)]">Entry list</p>
      <h1 class="font-display text-3xl leading-none text-white lg:text-4xl">GRID ENTRIES</h1>
    </div>
    <LeagueSwitcher onChange={onLeagueChange} />
  </div>
{/snippet}

{#if store.state?.session}
  <RaceControlShell
    mode="host"
    session={store.state.session}
    activeHref="/host/drivers"
    {header}
    onLogout={logout}
  >
    <div class="grid gap-4 lg:grid-cols-12">
      <div class="lg:col-span-5">
        <TimingPanel title="Send paddock pass" subtitle="Steam profile URL or SteamID64">
          <p class="mb-3 text-xs text-[var(--color-muted)]">
            Drivers must accept before they appear in standings and results matching.
          </p>
          <Input
            bind:value={steamInput}
            placeholder="https://steamcommunity.com/profiles/7656… or SteamID64"
            mono
          />
          {#if error}
            <p class="mt-2 text-xs text-[var(--color-red)]">{error}</p>
          {/if}
          {#if message}
            <p class="mt-2 text-xs text-[var(--color-green)]">{message}</p>
          {/if}
          <Button
            variant="primary"
            size="sm"
            class="mt-3 w-full"
            loading={inviting}
            disabled={!steamInput.trim()}
            onclick={sendInvite}
          >
            <UserPlus class="size-3.5" strokeWidth={2.5} />
            Send invite
          </Button>
        </TimingPanel>
      </div>

      <div class="lg:col-span-7">
        <TimingPanel title="Entry list" subtitle="Active drivers and pending invites">
          {#snippet actions()}
            <Button variant="ghost" size="sm" loading={refreshing} onclick={refreshAvatars}>
              <RefreshCw class="size-3.5" strokeWidth={2} />
              Refresh avatars
            </Button>
          {/snippet}

          {#if roster}
            {#if roster.members.length > 0}
              <div class="mb-4 space-y-2">
                <p class="font-label text-[0.55rem] text-[var(--color-dim)]">On grid</p>
                {#each roster.members as member, i (member.driverId)}
                  <div class="panel-timing flex items-center gap-3 rounded-md p-3">
                    <span class="font-mono text-xs text-[var(--color-dim)]">{i + 1}</span>
                    <Avatar src={member.avatarUrl} alt={member.personaname} size="sm" />
                    <div class="min-w-0 flex-1">
                      <p class="text-sm font-medium text-white">{member.personaname}</p>
                      <p class="font-mono text-[0.6rem] text-[var(--color-dim)]">{member.steamId64}</p>
                    </div>
                  </div>
                {/each}
              </div>
            {/if}

            {#if roster.pendingInvites.length > 0}
              <div class="space-y-2">
                <p class="font-label text-[0.55rem] text-[var(--color-dim)]">Pending invites</p>
                {#each roster.pendingInvites as invite (invite.id)}
                  <div class="flex items-center gap-3 rounded-md border border-dashed border-[var(--color-line)] p-3">
                    <Avatar src={invite.avatarUrl} alt={invite.personaname} size="sm" />
                    <div class="min-w-0 flex-1">
                      <p class="text-sm text-white">{invite.personaname}</p>
                      <p class="text-xs text-[var(--color-muted)]">
                        Sent {new Date(invite.invitedAt).toLocaleDateString()}
                      </p>
                    </div>
                    <Button variant="ghost" size="sm" onclick={() => revokeInvite(invite.id)}>
                      <X class="size-3.5" strokeWidth={2} />
                    </Button>
                  </div>
                {/each}
              </div>
            {:else if roster.members.length === 0}
              <p class="py-8 text-center text-sm text-[var(--color-muted)]">
                No drivers yet — send your first Steam invite.
              </p>
            {/if}
          {/if}
        </TimingPanel>
      </div>
    </div>
  </RaceControlShell>
{/if}
