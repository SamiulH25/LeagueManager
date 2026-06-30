<script lang="ts">
  import { goto } from "$app/navigation";
  import RaceControlShell from "$lib/components/layout/RaceControlShell.svelte";
  import TimingPanel from "$lib/components/racing/TimingPanel.svelte";
  import { Users } from "@lucide/svelte";
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
    <p class="font-label text-[0.65rem] text-[var(--color-red)]">Entry list</p>
    <h1 class="font-display text-3xl leading-none text-white lg:text-4xl">GRID ENTRIES</h1>
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
    <TimingPanel title="Driver roster" subtitle="Steam invite-only entry">
      <div class="flex flex-col items-center py-16 text-center">
        <Users class="mb-4 size-12 text-[var(--color-dim)]" strokeWidth={1.25} />
        <p class="font-display text-2xl text-[var(--color-dim)]">PHASE 2</p>
        <p class="mt-2 max-w-md text-sm text-[var(--color-muted)]">
          Invite drivers by Steam ID and manage your entry list here. For now, register
          championships and launch races from Race Control.
        </p>
      </div>
    </TimingPanel>
  </RaceControlShell>
{/if}
