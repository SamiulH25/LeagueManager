<script lang="ts">
  import { goto } from "$app/navigation";
  import RaceControlShell from "$lib/components/layout/RaceControlShell.svelte";
  import TimingPanel from "$lib/components/racing/TimingPanel.svelte";
  import { LayoutGrid } from "@lucide/svelte";
  import { getContext } from "svelte";
  import type { AppStore } from "$lib/stores/app.svelte";

  const store = getContext<AppStore>("app");

  async function logout() {
    await store.logout();
    goto("/login");
  }
</script>

{#snippet header()}
  <div>
    <p class="font-label text-[0.65rem] text-[var(--color-green)]">Standings</p>
    <h1 class="font-display text-3xl leading-none text-white lg:text-4xl">CHAMPIONSHIP TABLE</h1>
  </div>
{/snippet}

{#if store.state?.session}
  <RaceControlShell
    mode="driver"
    session={store.state.session}
    activeHref="/driver/leagues"
    {header}
    onLogout={logout}
  >
    <TimingPanel title="Your championships" subtitle="Points and standings">
      <div class="flex flex-col items-center py-16 text-center">
        <LayoutGrid class="mb-4 size-12 text-[var(--color-dim)]" strokeWidth={1.25} />
        <p class="font-display text-2xl text-[var(--color-dim)]">PHASE 2</p>
        <p class="mt-2 max-w-md text-sm text-[var(--color-muted)]">
          Standings sync from your host's pit link once you accept a league invite and connect.
        </p>
      </div>
    </TimingPanel>
  </RaceControlShell>
{/if}
