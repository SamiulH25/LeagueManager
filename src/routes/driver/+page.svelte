<script lang="ts">
  import { goto } from "$app/navigation";
  import RaceControlShell from "$lib/components/layout/RaceControlShell.svelte";
  import TimingPanel from "$lib/components/racing/TimingPanel.svelte";
  import TelemetryTile from "$lib/components/racing/TelemetryTile.svelte";
  import Avatar from "$lib/components/ui/Avatar.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import { api } from "$lib/api";
  import type { LeagueInvite } from "$lib/types";
  import { ExternalLink, Mail, Radio } from "@lucide/svelte";
  import { getContext, onMount } from "svelte";
  import type { AppStore } from "$lib/stores/app.svelte";

  const store = getContext<AppStore>("app");
  let invites = $state<LeagueInvite[]>([]);
  let hostAddress = $state("");

  onMount(async () => {
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

{#snippet header()}
  <div class="flex flex-wrap items-end justify-between gap-4">
    <div>
      <p class="font-label text-[0.65rem] text-[var(--color-green)]">Driver paddock</p>
      <h1 class="font-display text-3xl leading-none text-white lg:text-4xl">
        GRID READY <span class="text-[var(--color-dim)]">//</span>
        {store.state?.session?.personaname?.toUpperCase() ?? "DRIVER"}
      </h1>
    </div>
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
      <!-- Hero join strip — full width, racing billboard -->
      <div
        class="panel-timing relative overflow-hidden rounded-md lg:col-span-12"
      >
        <div class="pit-stripe absolute left-0 top-0 h-full w-2"></div>
        <div
          class="flex flex-col gap-4 p-5 sm:flex-row sm:items-center sm:justify-between lg:p-6"
        >
          <div class="flex items-center gap-4">
            <div
              class="grid size-14 place-items-center rounded-md border border-[var(--color-line)] bg-[var(--color-asphalt)]"
            >
              <Radio class="size-7 text-[var(--color-dim)]" strokeWidth={1.5} />
            </div>
            <div>
              <p class="font-label text-[0.65rem] text-[var(--color-dim)]">Live session</p>
              <p class="font-display text-2xl text-white">NO RACE IN PROGRESS</p>
              <p class="text-xs text-[var(--color-muted)]">
                When your host goes live, join via Content Manager in one click.
              </p>
            </div>
          </div>
          <Button variant="green" size="lg" class="shrink-0" disabled>
            <ExternalLink class="size-4" strokeWidth={2} />
            Join in Content Manager
          </Button>
        </div>
      </div>

      <!-- Telemetry -->
      <div class="grid grid-cols-3 gap-3 lg:col-span-12">
        <TelemetryTile label="Pending invites" value={invites.length} accent="yellow" />
        <TelemetryTile label="Leagues joined" value="0" accent="dim" />
        <TelemetryTile label="Championship pts" value="—" accent="dim" />
      </div>

      <!-- Paddock passes (invites) -->
      <div class="lg:col-span-7">
        <TimingPanel title="Paddock passes" subtitle="Steam invite-only entry">
          {#if invites.length > 0}
            <ul class="space-y-3">
              {#each invites as invite (invite.id)}
                <li
                  class="panel-timing flex items-center gap-4 rounded-md p-4"
                >
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
                    <Button variant="ghost" size="sm">Pass</Button>
                    <Button variant="primary" size="sm">Accept</Button>
                  </div>
                </li>
              {/each}
            </ul>
          {:else}
            <div class="flex flex-col items-center py-10 text-center">
              <Mail class="mb-3 size-10 text-[var(--color-dim)]" strokeWidth={1.25} />
              <p class="font-display text-xl text-[var(--color-dim)]">NO PADDOCK PASSES</p>
              <p class="mt-2 max-w-xs text-sm text-[var(--color-muted)]">
                Your host must invite your Steam account. Once accepted, standings and join links
                appear here.
              </p>
            </div>
          {/if}
        </TimingPanel>
      </div>

      <!-- Garage / host link -->
      <div class="lg:col-span-5">
        <TimingPanel title="Pit link" subtitle="Your league host's address">
          <p class="mb-3 text-xs text-[var(--color-muted)]">
            Enter the public IP your race chief shared. League data syncs over the pit link port.
          </p>
          <Input bind:value={hostAddress} placeholder="203.0.113.42" mono />
          <Button variant="secondary" size="sm" class="mt-3 w-full" onclick={saveHostAddress}>
            Save pit link
          </Button>

          <div
            class="mt-4 rounded-md border border-[color-mix(in_srgb,var(--color-green)_25%,transparent)] bg-[color-mix(in_srgb,var(--color-green)_6%,transparent)] p-3"
          >
            <p class="font-label text-[0.6rem] text-[var(--color-green)]">Phase 2</p>
            <p class="mt-1 text-sm text-[var(--color-muted)]">
              Standings sync and CM join button activate once connected to host.
            </p>
          </div>
        </TimingPanel>
      </div>
    </div>
  </RaceControlShell>
{/if}
