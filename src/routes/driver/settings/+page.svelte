<script lang="ts">
  import { goto } from "$app/navigation";
  import RaceControlShell from "$lib/components/layout/RaceControlShell.svelte";
  import TimingPanel from "$lib/components/racing/TimingPanel.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import { onMount } from "svelte";
  import { getContext } from "svelte";
  import type { AppStore } from "$lib/stores/app.svelte";

  const store = getContext<AppStore>("app");
  let hostAddress = $state("");
  let saved = $state(false);

  onMount(() => {
    hostAddress = localStorage.getItem("lm_host_address") ?? "";
  });

  function saveHostAddress() {
    localStorage.setItem("lm_host_address", hostAddress);
    saved = true;
    setTimeout(() => (saved = false), 2000);
  }

  async function logout() {
    await store.logout();
    goto("/login");
  }
</script>

{#snippet header()}
  <div>
    <p class="font-label text-[0.65rem] text-[var(--color-green)]">Garage</p>
    <h1 class="font-display text-3xl leading-none text-white lg:text-4xl">DRIVER SETTINGS</h1>
  </div>
{/snippet}

{#if store.state?.session}
  <RaceControlShell
    mode="driver"
    session={store.state.session}
    activeHref="/driver/settings"
    {header}
    onLogout={logout}
  >
    <div class="mx-auto max-w-lg">
      <TimingPanel title="Pit link" subtitle="Your league host's address">
        <p class="mb-3 text-xs text-[var(--color-muted)]">
          Enter the public IP your race chief shared. League data syncs over the pit link port in
          Phase 2.
        </p>
        <Input bind:value={hostAddress} placeholder="203.0.113.42" mono />
        <Button variant="secondary" size="sm" class="mt-3 w-full" onclick={saveHostAddress}>
          {saved ? "Saved!" : "Save pit link"}
        </Button>
      </TimingPanel>
    </div>
  </RaceControlShell>
{/if}
