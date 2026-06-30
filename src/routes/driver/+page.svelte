<script lang="ts">
  import { goto } from "$app/navigation";
  import RaceControlShell from "$lib/components/layout/RaceControlShell.svelte";
  import TimingPanel from "$lib/components/racing/TimingPanel.svelte";
  import TimingRow from "$lib/components/racing/TimingRow.svelte";
  import TelemetryTile from "$lib/components/racing/TelemetryTile.svelte";
  import Avatar from "$lib/components/ui/Avatar.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import { api } from "$lib/api";
  import { loadPitLink } from "$lib/pit-link";
  import type { CurrentEvent, LeagueInvite } from "$lib/types";
  import { ExternalLink, Mail, Radio } from "@lucide/svelte";
  import { getContext, onDestroy, onMount } from "svelte";
  import type { AppStore } from "$lib/stores/app.svelte";

  const store = getContext<AppStore>("app");
  let invites = $state<LeagueInvite[]>([]);
  let pitLink = $state(loadPitLink());
  let currentEvent = $state<CurrentEvent | null>(null);
  let fetchError = $state<string | null>(null);
  let joining = $state(false);
  let pollTimer: ReturnType<typeof setInterval> | undefined;

  const live = $derived(currentEvent?.status === "live");
  const configured = $derived(Boolean(pitLink.host.trim()));

  async function refreshEvent() {
    if (!configured) {
      currentEvent = null;
      fetchError = null;
      return;
    }
    try {
      currentEvent = await api.fetchRemoteCurrentEvent(pitLink.host, pitLink.port);
      fetchError = null;
    } catch (e) {
      fetchError = e instanceof Error ? e.message : String(e);
      currentEvent = null;
    }
  }

  async function loadInvites() {
    const link = loadPitLink();
    if (link.host.trim()) {
      try {
        invites = await api.fetchRemoteInvites(link.host, link.port);
        return;
      } catch {
        invites = [];
        return;
      }
    }
    invites = await api.listMyInvites();
  }

  onMount(async () => {
    pitLink = loadPitLink();
    await loadInvites();
    await refreshEvent();
    pollTimer = setInterval(refreshEvent, 5000);
  });

  onDestroy(() => {
    if (pollTimer) clearInterval(pollTimer);
  });

  async function joinInCm() {
    if (!configured) return;
    joining = true;
    try {
      await api.openRemoteCmJoinLink(pitLink.host, pitLink.port);
    } catch (e) {
      fetchError = e instanceof Error ? e.message : String(e);
    } finally {
      joining = false;
    }
  }

  async function acceptInvite(inviteId: number) {
    const link = loadPitLink();
    if (!link.host.trim()) return;
    await api.acceptRemoteInvite(link.host, link.port, inviteId);
    await loadInvites();
  }

  async function declineInvite(inviteId: number) {
    const link = loadPitLink();
    if (!link.host.trim()) return;
    await api.declineRemoteInvite(link.host, link.port, inviteId);
    await loadInvites();
  }

  async function logout() {
    await store.logout();
    goto("/login");
  }

  function formatTime(seconds: number | null | undefined) {
    if (seconds == null) return "—";
    const m = Math.floor(seconds / 60);
    const s = seconds % 60;
    return `${m}:${String(s).padStart(2, "0")}`;
  }
</script>

{#snippet header()}
  <div class="flex flex-wrap items-end justify-between gap-4">
    <div>
      <p class="font-label text-[0.65rem] text-[var(--color-green)]">Driver paddock</p>
      <h1 class="font-display text-3xl leading-none text-white lg:text-4xl">
        GRID READY <span class="text-[var(--color-dim)]">//</span>
        {store.state?.session?.personaname?.toUpperCase() ?? "DRIVER"}
      </h1>
    </div>
    {#if configured}
      <div class="flex items-center gap-2">
        <span class="status-led {live ? 'status-led--live' : 'status-led--idle'}"></span>
        <span class="font-label text-[0.6rem] text-[var(--color-muted)]">
          {live ? "Race live" : fetchError ? "Pit link offline" : "Host idle"}
        </span>
      </div>
    {/if}
  </div>
{/snippet}

{#if store.state?.session}
  <RaceControlShell
    mode="driver"
    session={store.state.session}
    activeHref="/driver"
    {header}
    onLogout={logout}
  >
    <div class="grid gap-4 lg:grid-cols-12">
      <div class="panel-timing relative overflow-hidden rounded-md lg:col-span-12">
        <div class="pit-stripe absolute left-0 top-0 h-full w-2"></div>
        <div
          class="flex flex-col gap-4 p-5 sm:flex-row sm:items-center sm:justify-between lg:p-6"
        >
          <div class="flex items-center gap-4">
            <div
              class="grid size-14 place-items-center rounded-md border border-[var(--color-line)] bg-[var(--color-asphalt)]"
            >
              <Radio
                class="size-7 {live ? 'text-[var(--color-green)]' : 'text-[var(--color-dim)]'}"
                strokeWidth={1.5}
              />
            </div>
            <div>
              <p class="font-label text-[0.65rem] text-[var(--color-dim)]">Live session</p>
              {#if live && currentEvent}
                <p class="font-display text-2xl text-[var(--color-green)]">
                  {currentEvent.serverName?.toUpperCase() ?? "RACE LIVE"}
                </p>
                <p class="text-xs text-[var(--color-muted)]">
                  {currentEvent.track ?? "Unknown track"} · {currentEvent.clients ?? 0}/{currentEvent.maxClients ?? "?"}
                  on track · {formatTime(currentEvent.timeLeft)} left
                </p>
              {:else if !configured}
                <p class="font-display text-2xl text-white">CONFIGURE PIT LINK</p>
                <p class="text-xs text-[var(--color-muted)]">
                  Set your host address in Garage before joining races.
                </p>
              {:else if fetchError}
                <p class="font-display text-2xl text-white">HOST UNREACHABLE</p>
                <p class="text-xs text-[var(--color-red)]">{fetchError}</p>
              {:else}
                <p class="font-display text-2xl text-white">NO RACE IN PROGRESS</p>
                <p class="text-xs text-[var(--color-muted)]">
                  Connected to host — waiting for the green flag.
                </p>
              {/if}
            </div>
          </div>
          <Button
            variant="green"
            size="lg"
            class="shrink-0"
            disabled={!live || joining}
            loading={joining}
            onclick={joinInCm}
          >
            <ExternalLink class="size-4" strokeWidth={2} />
            Join in Content Manager
          </Button>
        </div>
      </div>

      <div class="grid grid-cols-3 gap-3 lg:col-span-12">
        <TelemetryTile label="Pending invites" value={invites.length} accent="yellow" />
        <TelemetryTile
          label="Pit link"
          value={configured ? "OK" : "—"}
          accent={configured ? "green" : "dim"}
        />
        <TelemetryTile
          label="Session"
          value={live ? "LIVE" : "IDLE"}
          accent={live ? "green" : "dim"}
        />
      </div>

      <div class="lg:col-span-7">
        <TimingPanel title="Paddock passes" subtitle="Steam invite-only entry">
          {#if invites.length > 0}
            <ul class="space-y-3">
              {#each invites as invite (invite.id)}
                <li class="panel-timing flex items-center gap-4 rounded-md p-4">
                  <div class="checkered h-12 w-10 shrink-0 rounded-sm opacity-80"></div>
                  {#if invite.hostAvatarUrl}
                    <Avatar
                      src={invite.hostAvatarUrl}
                      alt={invite.hostPersonaname ?? "Host"}
                      size="md"
                    />
                  {/if}
                  <div class="min-w-0 flex-1">
                    <p class="font-display text-lg tracking-wide">{invite.leagueName}</p>
                    <p class="font-mono text-xs text-[var(--color-dim)]">
                      Chief · {invite.hostPersonaname ?? "Unknown host"}
                    </p>
                  </div>
                  <div class="flex shrink-0 gap-2">
                    <Button variant="ghost" size="sm" onclick={() => declineInvite(invite.id)}>Pass</Button>
                    <Button variant="primary" size="sm" onclick={() => acceptInvite(invite.id)}>Accept</Button>
                  </div>
                </li>
              {/each}
            </ul>
          {:else}
            <div class="flex flex-col items-center py-10 text-center">
              <Mail class="mb-3 size-10 text-[var(--color-dim)]" strokeWidth={1.25} />
              <p class="font-display text-xl text-[var(--color-dim)]">NO PADDOCK PASSES</p>
              <p class="mt-2 max-w-xs text-sm text-[var(--color-muted)]">
                Your host must invite your Steam account. Once accepted, standings appear under
                Standings.
              </p>
            </div>
          {/if}
        </TimingPanel>
      </div>

      <div class="lg:col-span-5">
        <TimingPanel title="Pit link" subtitle="Host sync address">
          <p class="mb-2 font-mono text-sm text-white">
            {pitLink.host || "—"}:{pitLink.port}
          </p>
          <p class="text-xs text-[var(--color-muted)]">
            Configure in <a href="/driver/settings" class="text-[var(--color-yellow)] hover:underline">Garage</a>.
            Drivers pull live events and standings over this port.
          </p>
        </TimingPanel>
      </div>
    </div>
  </RaceControlShell>
{/if}
