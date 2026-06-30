<script lang="ts">
  import { goto } from "$app/navigation";
  import { dev } from "$app/environment";
  import Logo from "$lib/components/brand/Logo.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import { getContext } from "svelte";
  import type { AppStore } from "$lib/stores/app.svelte";

  const store = getContext<AppStore>("app");
  let loading = $state(false);
  let error = $state<string | null>(null);
  let devSteamId = $state("76561198000000000");

  $effect(() => {
    if (!store.loading && store.state?.session) {
      routeAfterLogin();
    }
  });

  async function loginSteam() {
    loading = true;
    error = null;
    try {
      await store.loginWithSteam();
      routeAfterLogin();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  async function loginDev() {
    loading = true;
    error = null;
    try {
      await store.loginDev(devSteamId);
      routeAfterLogin();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  function routeAfterLogin() {
    const s = store.state;
    if (!s?.onboardingComplete || !s.appMode) {
      goto("/welcome");
    } else {
      goto(s.appMode === "host" ? "/host" : "/driver");
    }
  }
</script>

<div class="flex min-h-screen items-center justify-center px-4 py-10">
  <div class="w-full max-w-md">
    <div class="mb-8 text-center">
      <Logo showSubtitle />
      <h1 class="font-display mt-6 text-4xl tracking-wide text-white">DRIVER LICENCE</h1>
      <p class="mt-2 text-sm text-[var(--color-muted)]">
        Steam ID required for invites, standings, and results matching.
      </p>
    </div>

    <div class="panel-timing overflow-hidden rounded-md">
      <div class="checkered h-2 w-full opacity-90"></div>
      <div class="space-y-4 p-5">
        {#if error}
          <div
            class="rounded-md border border-[var(--color-red)] bg-[color-mix(in_srgb,var(--color-red)_12%,transparent)] px-3 py-2 font-mono text-xs text-[var(--color-red)]"
          >
            ERR: {error}
          </div>
        {/if}

        <Button variant="steam" size="lg" class="w-full" {loading} onclick={loginSteam}>
          Authenticate via Steam
        </Button>

        <p class="text-center font-mono text-[0.65rem] text-[var(--color-dim)]">
          Opens browser · OpenID 2.0
        </p>

        {#if dev}
          <div class="border-t border-[var(--color-line)] pt-4">
            <p class="font-label mb-2 text-[0.6rem] text-[var(--color-dim)]">Dev bypass</p>
            <Input bind:value={devSteamId} placeholder="SteamID64" mono />
            <Button variant="ghost" size="sm" class="mt-2 w-full" {loading} onclick={loginDev}>
              Quick pit entry
            </Button>
          </div>
        {/if}
      </div>
    </div>
  </div>
</div>
