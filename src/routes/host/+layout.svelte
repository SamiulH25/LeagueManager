<script lang="ts">
  import { guardHost } from "$lib/session-guard";
  import type { AppStore } from "$lib/stores/app.svelte";
  import { getContext, onMount } from "svelte";

  const store = getContext<AppStore>("app");
  let { children } = $props();
  let ready = $state(false);

  onMount(() => {
    ready = guardHost(store);
  });
</script>

{#if ready}
  {@render children()}
{/if}
