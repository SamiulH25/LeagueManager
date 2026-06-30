<script lang="ts">
  import { goto } from "$app/navigation";
  import Logo from "$lib/components/brand/Logo.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import { Flag, Radio, Trophy, Users } from "@lucide/svelte";
  import { getContext } from "svelte";
  import type { AppStore } from "$lib/stores/app.svelte";

  const store = getContext<AppStore>("app");
  let saving = $state(false);

  $effect(() => {
    if (!store.loading && store.state && !store.state.session) {
      goto("/login");
    }
  });

  async function choose(mode: "host" | "driver") {
    saving = true;
    try {
      await store.setMode(mode);
      goto(mode === "host" ? "/host" : "/driver");
    } finally {
      saving = false;
    }
  }
</script>

<div class="mx-auto flex min-h-screen max-w-4xl flex-col justify-center px-4 py-10">
  <div class="mb-10 text-center">
    <Logo size="lg" showSubtitle />
    <h1 class="font-display mt-8 text-5xl tracking-wide text-white md:text-6xl">
      SELECT YOUR <span class="text-stroke-red">ROLE</span>
    </h1>
    <p class="mx-auto mt-3 max-w-md text-sm text-[var(--color-muted)]">
      Run the pit wall or join the grid. Assetto Corsa leagues through Content Manager.
    </p>
  </div>

  <div class="grid gap-4 md:grid-cols-2">
    <!-- Host — race control card -->
    <button
      type="button"
      class="group panel-timing relative overflow-hidden rounded-md text-left transition-transform hover:-translate-y-0.5 disabled:opacity-50"
      disabled={saving}
      onclick={() => choose("host")}
    >
      <div class="pit-stripe absolute left-0 top-0 h-full w-1.5"></div>
      <div class="border-b border-[var(--color-line)] bg-[color-mix(in_srgb,var(--color-red)_12%,var(--color-panel))] px-5 py-3">
        <p class="font-label text-[0.6rem] text-[var(--color-red)]">01 · Race control</p>
      </div>
      <div class="p-5">
        <Radio class="mb-4 size-10 text-[var(--color-red)]" strokeWidth={1.5} />
        <h2 class="font-display text-3xl text-white">PIT WALL CHIEF</h2>
        <p class="mt-2 text-sm leading-relaxed text-[var(--color-muted)]">
          Launch AssettoServer, manage championships, invite drivers, auto-score results.
        </p>
        <ul class="mt-4 space-y-1.5 font-mono text-xs text-[var(--color-dim)]">
          <li class="flex items-center gap-2"><Trophy class="size-3.5" /> Unlimited leagues</li>
          <li class="flex items-center gap-2"><Users class="size-3.5" /> Steam invites</li>
        </ul>
        <Button variant="primary" class="pointer-events-none mt-6 w-full">Enter race control</Button>
      </div>
    </button>

    <!-- Driver — paddock card -->
    <button
      type="button"
      class="group panel-timing relative overflow-hidden rounded-md text-left transition-transform hover:-translate-y-0.5 disabled:opacity-50"
      disabled={saving}
      onclick={() => choose("driver")}
    >
      <div class="border-b border-[var(--color-line)] bg-[color-mix(in_srgb,var(--color-green)_8%,var(--color-panel))] px-5 py-3">
        <p class="font-label text-[0.6rem] text-[var(--color-green)]">02 · Driver paddock</p>
      </div>
      <div class="p-5">
        <Flag class="mb-4 size-10 text-[var(--color-green)]" strokeWidth={1.5} />
        <h2 class="font-display text-3xl text-white">GRID DRIVER</h2>
        <p class="mt-2 text-sm leading-relaxed text-[var(--color-muted)]">
          Accept invites, track standings, join race night through Content Manager.
        </p>
        <ul class="mt-4 space-y-1.5 font-mono text-xs text-[var(--color-dim)]">
          <li>Invite-only leagues</li>
          <li>One-click CM join</li>
        </ul>
        <Button variant="green" class="pointer-events-none mt-6 w-full">Enter paddock</Button>
      </div>
    </button>
  </div>
</div>
