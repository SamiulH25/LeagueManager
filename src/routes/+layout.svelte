<script lang="ts">
  import "../app.css";
  import PitLaneBackground from "$lib/components/effects/PitLaneBackground.svelte";
  import { getAppStore } from "$lib/stores/app.svelte";
  import { setContext } from "svelte";

  const store = getAppStore();
  setContext("app", store);

  let { children } = $props();

  $effect(() => {
    store.refresh();
  });
</script>

<svelte:head>
  <link rel="preconnect" href="https://fonts.googleapis.com" />
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="anonymous" />
  <link
    href="https://fonts.googleapis.com/css2?family=Barlow+Condensed:wght@500;600;700&family=Barlow:wght@400;500;600&family=Bebas+Neue&family=JetBrains+Mono:wght@500;700&display=swap"
    rel="stylesheet"
  />
</svelte:head>

<PitLaneBackground />
{#if store.loading && !store.state}
  <div class="grid min-h-screen place-items-center">
    <div class="font-label animate-pulse text-sm text-[var(--color-red)]">Initializing pit lane…</div>
  </div>
{:else}
  {@render children()}
{/if}
