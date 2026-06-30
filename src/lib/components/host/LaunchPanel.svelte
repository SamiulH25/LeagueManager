<script lang="ts">
  import TimingPanel from "$lib/components/racing/TimingPanel.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import { api } from "$lib/api";
  import type { RaceLaunchConfig, ServerStatus } from "$lib/types";
  import { Copy, ExternalLink, Play, Square, Wifi } from "@lucide/svelte";
  import { onDestroy, onMount } from "svelte";

  let status = $state<ServerStatus | null>(null);
  let launching = $state(false);
  let stopping = $state(false);
  let error = $state<string | null>(null);
  let copied = $state(false);

  let serverName = $state("LeagueManager Race");
  let track = $state("ks_nordschleife");
  let trackConfig = $state("");
  let cars = $state("abarth500");
  let password = $state("");
  let practiceMinutes = $state("10");
  let qualifyMinutes = $state("15");
  let raceMinutes = $state("20");
  let maxClients = $state("16");

  let pollTimer: ReturnType<typeof setInterval> | undefined;

  async function refreshStatus() {
    try {
      status = await api.getServerStatus();
      error = status.error ?? null;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  onMount(() => {
    refreshStatus();
    pollTimer = setInterval(refreshStatus, 3000);
  });

  onDestroy(() => {
    if (pollTimer) clearInterval(pollTimer);
  });

  function buildConfig(): RaceLaunchConfig {
    return {
      serverName: serverName.trim() || "LeagueManager Race",
      track: track.trim() || "ks_nordschleife",
      trackConfig: trackConfig.trim(),
      cars: cars
        .split(/[;,]/)
        .map((c) => c.trim())
        .filter(Boolean),
      password: password.trim(),
      practiceMinutes: Number(practiceMinutes) || 0,
      qualifyMinutes: Number(qualifyMinutes) || 15,
      raceMinutes: Number(raceMinutes) || 20,
      maxClients: Number(maxClients) || 16,
      aiSlots: 0,
    };
  }

  async function startServer() {
    launching = true;
    error = null;
    try {
      await api.startRaceServer(buildConfig());
      await refreshStatus();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      launching = false;
    }
  }

  async function stopServer() {
    stopping = true;
    error = null;
    try {
      await api.stopRaceServer();
      await refreshStatus();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      stopping = false;
    }
  }

  async function copyJoinLink() {
    const link = status?.cmJoinLink;
    if (!link) return;
    await navigator.clipboard.writeText(link);
    copied = true;
    setTimeout(() => (copied = false), 2000);
  }

  async function openInCm() {
    try {
      await api.openCmJoinLink();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  const live = $derived(status?.running ?? false);
  const publicIp = $derived(status?.publicIp ?? "— — —");
  const httpPort = $derived(status?.httpPort ?? 8081);
  const gamePort = $derived(status?.gamePort ?? 9600);
</script>

<div class="flex flex-col gap-4">
  <TimingPanel title="Broadcast" subtitle="Share with your grid">
    <div class="space-y-3">
      <div class="rounded-md border border-dashed border-[var(--color-line)] p-4">
        <p class="font-label text-[0.6rem] text-[var(--color-dim)]">Public IP</p>
        <p class="font-mono text-lg {live ? 'text-[var(--color-green)]' : 'text-[var(--color-dim)]'}">
          {publicIp}
        </p>
        {#if status?.info}
          <p class="mt-1 text-xs text-[var(--color-muted)]">
            {status.info.name} · {status.info.track} · {status.info.clients}/{status.info.maxClients}
            on track
          </p>
        {:else}
          <p class="mt-1 text-xs text-[var(--color-muted)]">
            {live ? "Waiting for /INFO…" : "Detected when server launches"}
          </p>
        {/if}
      </div>
      <div class="flex items-center gap-2 rounded-md bg-[var(--color-asphalt)] px-3 py-2">
        <Wifi class="size-4 {live ? 'text-[var(--color-green)]' : 'text-[var(--color-dim)]'}" strokeWidth={1.75} />
        <span class="font-mono text-xs text-[var(--color-muted)]">
          HTTP :{httpPort} · UDP :{gamePort}
        </span>
      </div>
      {#if status?.cmJoinLink}
        <div class="flex gap-2">
          <Button variant="secondary" size="sm" class="flex-1" onclick={copyJoinLink}>
            <Copy class="size-3.5" strokeWidth={2} />
            {copied ? "Copied!" : "Copy CM link"}
          </Button>
          <Button variant="green" size="sm" class="flex-1" onclick={openInCm}>
            <ExternalLink class="size-3.5" strokeWidth={2} />
            Open in CM
          </Button>
        </div>
      {/if}
    </div>
  </TimingPanel>

  <TimingPanel title="Race launch" subtitle="Spin up AssettoServer">
    <div class="space-y-3">
      <Input bind:value={serverName} placeholder="Server name" />
      <Input bind:value={track} placeholder="Track ID (e.g. ks_nordschleife)" mono />
      <Input bind:value={trackConfig} placeholder="Track config (optional)" mono />
      <Input bind:value={cars} placeholder="Cars (semicolon-separated)" mono />
      <Input bind:value={password} placeholder="Server password (optional)" />
      <div class="grid grid-cols-3 gap-2">
        <div>
          <p class="mb-1 font-label text-[0.55rem] text-[var(--color-dim)]">Practice</p>
          <Input bind:value={practiceMinutes} type="number" mono />
        </div>
        <div>
          <p class="mb-1 font-label text-[0.55rem] text-[var(--color-dim)]">Qualify</p>
          <Input bind:value={qualifyMinutes} type="number" mono />
        </div>
        <div>
          <p class="mb-1 font-label text-[0.55rem] text-[var(--color-dim)]">Race</p>
          <Input bind:value={raceMinutes} type="number" mono />
        </div>
      </div>
      <Input bind:value={maxClients} type="number" placeholder="Max clients" mono />

      {#if error}
        <p class="text-xs text-[var(--color-red)]">{error}</p>
      {/if}

      {#if live}
        <Button variant="secondary" class="w-full" loading={stopping} onclick={stopServer}>
          <Square class="size-3.5" strokeWidth={2.5} />
          Stop server
        </Button>
      {:else}
        <Button variant="primary" class="w-full" loading={launching} onclick={startServer}>
          <Play class="size-3.5" strokeWidth={2.5} />
          Launch server
        </Button>
      {/if}

      <p class="text-xs text-[var(--color-muted)]">
        Configure AssettoServer path in
        <a href="/host/settings" class="text-[var(--color-yellow)] hover:underline">Pit Config</a>
        before launching.
      </p>
    </div>
  </TimingPanel>
</div>
