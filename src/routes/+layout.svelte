<script lang="ts">
  import "../app.css";
  import AmbientBackground from "$lib/components/effects/AmbientBackground.svelte";
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
    href="https://fonts.googleapis.com/css2?family=DM+Sans:ital,opsz,wght@0,9..40,400;0,9..40,500;0,9..40,600;0,9..40,700;1,9..40,400&family=Syne:wght@600;700;800&display=swap"
    rel="stylesheet"
  />
</svelte:head>

<AmbientBackground />
{#if store.loading && !store.state}
  <div class="grid min-h-screen place-items-center">
    <div
      class="size-10 animate-spin rounded-full border-2 border-[var(--color-carbon-border)] border-t-[var(--color-racing)]"
    ></div>
  </div>
{:else}
  {@render children()}
{/if}
