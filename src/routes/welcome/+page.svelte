<script lang="ts">
  import { goto } from "$app/navigation";
  import Logo from "$lib/components/brand/Logo.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Card from "$lib/components/ui/Card.svelte";
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

<div class="mx-auto flex min-h-screen max-w-5xl flex-col justify-center px-6 py-12">
  <div class="mb-12 text-center">
    <div class="mb-6 flex justify-center">
      <Logo size="lg" />
    </div>
    <h1 class="font-display text-4xl font-bold tracking-tight md:text-5xl">
      How will you <span class="text-gradient-racing">race</span>?
    </h1>
    <p class="mx-auto mt-4 max-w-lg text-[var(--color-muted)]">
      Host your own Assetto Corsa league from your PC, or join as a driver and connect through
      Content Manager.
    </p>
  </div>

  <div class="grid gap-6 md:grid-cols-2">
    <button
      class="group text-left"
      disabled={saving}
      onclick={() => choose("host")}
    >
      <Card glow class="h-full transition-transform duration-300 group-hover:-translate-y-1">
        <div
          class="mb-4 flex size-14 items-center justify-center rounded-2xl bg-gradient-to-br from-[var(--color-racing)] to-[var(--color-amber)] text-2xl shadow-lg"
        >
          🏁
        </div>
        <h2 class="font-display text-2xl font-bold">Host a league</h2>
        <p class="mt-2 text-sm leading-relaxed text-[var(--color-muted)]">
          Configure championships, invite drivers via Steam, launch AssettoServer, and auto-score
          results.
        </p>
        <ul class="mt-4 space-y-2 text-sm text-[var(--color-muted)]">
          <li>• Unlimited leagues</li>
          <li>• One-click server launch</li>
          <li>• Points & standings</li>
        </ul>
        <div class="mt-6">
          <Button variant="primary" class="pointer-events-none w-full">Continue as host</Button>
        </div>
      </Card>
    </button>

    <button
      class="group text-left"
      disabled={saving}
      onclick={() => choose("driver")}
    >
      <Card class="h-full transition-transform duration-300 group-hover:-translate-y-1">
        <div
          class="mb-4 flex size-14 items-center justify-center rounded-2xl bg-[var(--color-carbon-elevated)] border border-[var(--color-carbon-border)] text-2xl"
        >
          🏎️
        </div>
        <h2 class="font-display text-2xl font-bold">Join as driver</h2>
        <p class="mt-2 text-sm leading-relaxed text-[var(--color-muted)]">
          Accept league invites, track standings, and jump into race night through Content Manager.
        </p>
        <ul class="mt-4 space-y-2 text-sm text-[var(--color-muted)]">
          <li>• Steam invite only</li>
          <li>• Live race alerts</li>
          <li>• CM join button</li>
        </ul>
        <div class="mt-6">
          <Button variant="secondary" class="pointer-events-none w-full">Continue as driver</Button>
        </div>
      </Card>
    </button>
  </div>
</div>
