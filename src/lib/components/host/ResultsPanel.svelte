<script lang="ts">
  import TimingPanel from "$lib/components/racing/TimingPanel.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import { api } from "$lib/api";
  import type { ImportResult, ResultsFeed } from "$lib/types";
  import { AlertTriangle, FileJson, Upload, X } from "@lucide/svelte";
  import { onDestroy, onMount } from "svelte";

  let feed = $state<ResultsFeed | null>(null);
  let manualJson = $state("");
  let importing = $state(false);
  let importResult = $state<ImportResult | null>(null);
  let pollTimer: ReturnType<typeof setInterval> | undefined;

  async function refresh() {
    feed = await api.getResultsFeed();
  }

  onMount(() => {
    refresh();
    pollTimer = setInterval(refresh, 4000);
  });

  onDestroy(() => {
    if (pollTimer) clearInterval(pollTimer);
  });

  async function importManual() {
    if (!manualJson.trim()) return;
    importing = true;
    importResult = null;
    try {
      importResult = await api.importResultsJson(manualJson.trim());
      manualJson = "";
      await refresh();
    } finally {
      importing = false;
    }
  }

  async function dismissWarning(id: number) {
    await api.dismissResultsWarning(id);
    await refresh();
  }

  function sessionLabel(type: string) {
    if (type === "R") return "Race";
    if (type === "Q") return "Qualifying";
    if (type === "P") return "Practice";
    return type;
  }
</script>

<div class="flex flex-col gap-4">
  {#if feed && feed.warnings.length > 0}
    <div class="space-y-2">
      {#each feed.warnings as warning (warning.id)}
        <div
          class="flex items-start gap-3 rounded-md border border-[color-mix(in_srgb,var(--color-yellow)_35%,transparent)] bg-[color-mix(in_srgb,var(--color-yellow)_8%,transparent)] p-3"
        >
          <AlertTriangle class="mt-0.5 size-4 shrink-0 text-[var(--color-yellow)]" strokeWidth={2} />
          <div class="min-w-0 flex-1">
            <p class="text-sm text-white">{warning.message}</p>
            {#if warning.fileName}
              <p class="mt-0.5 font-mono text-xs text-[var(--color-dim)]">{warning.fileName}</p>
            {/if}
          </div>
          <button
            type="button"
            class="text-[var(--color-dim)] hover:text-white"
            onclick={() => dismissWarning(warning.id)}
            title="Dismiss"
          >
            <X class="size-4" strokeWidth={2} />
          </button>
        </div>
      {/each}
    </div>
  {/if}

  <TimingPanel title="Results import" subtitle="Auto-watch AssettoServer results/">
    <div class="mb-3 flex items-center gap-2 text-xs text-[var(--color-muted)]">
      <span
        class="status-led {feed?.watcherActive ? 'status-led--live' : 'status-led--idle'}"
      ></span>
      {feed?.watcherActive ? "Watching results folder" : "Watcher idle — launch server to start"}
    </div>

    {#if feed && feed.recent.length > 0}
      <ul class="mb-4 space-y-2">
        {#each feed.recent as item (item.id)}
          <li class="rounded-md bg-[var(--color-asphalt)] px-3 py-2 text-sm">
            <div class="flex items-center justify-between gap-2">
              <span class="font-label text-[0.6rem] text-[var(--color-green)]">
                {sessionLabel(item.sessionType)}
              </span>
              <span class="font-mono text-[0.6rem] text-[var(--color-dim)]">{item.source}</span>
            </div>
            <p class="mt-1 text-white">{item.track}</p>
            <p class="font-mono text-xs text-[var(--color-muted)]">
              {item.entryCount} entries · {new Date(item.importedAt).toLocaleString()}
            </p>
          </li>
        {/each}
      </ul>
    {:else}
      <p class="mb-4 text-sm text-[var(--color-muted)]">
        No imports yet. Results appear here after each session when AssettoServer writes JSON to
        its results folder.
      </p>
    {/if}

    <div class="border-t border-[var(--color-line)] pt-4">
      <p class="mb-2 font-label text-[0.6rem] text-[var(--color-dim)]">Manual fallback</p>
      <textarea
        bind:value={manualJson}
        placeholder="Paste AC results JSON here…"
        rows="4"
        class="w-full rounded-md border border-[var(--color-line)] bg-[var(--color-asphalt)] px-3 py-2 font-mono text-xs text-white placeholder:text-[var(--color-dim)] outline-none focus:border-[var(--color-red)]"
      ></textarea>
      <Button
        variant="secondary"
        size="sm"
        class="mt-2 w-full"
        loading={importing}
        disabled={!manualJson.trim()}
        onclick={importManual}
      >
        <Upload class="size-3.5" strokeWidth={2} />
        Import JSON
      </Button>

      {#if importResult}
        <div
          class="mt-3 rounded-md p-3 text-xs {importResult.success
            ? 'bg-[color-mix(in_srgb,var(--color-green)_8%,transparent)] text-[var(--color-green)]'
            : 'bg-[color-mix(in_srgb,var(--color-red)_8%,transparent)] text-[var(--color-red)]'}"
        >
          <div class="flex items-center gap-2">
            <FileJson class="size-3.5" strokeWidth={2} />
            {importResult.message}
          </div>
          {#if importResult.success && importResult.pointsAwarded > 0}
            <p class="mt-1 text-[var(--color-muted)]">
              +{importResult.pointsAwarded} championship points awarded
            </p>
          {/if}
        </div>
      {/if}
    </div>
  </TimingPanel>
</div>
