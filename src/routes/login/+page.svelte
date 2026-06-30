<script lang="ts">
  import { goto } from "$app/navigation";
  import { dev } from "$app/environment";
  import Logo from "$lib/components/brand/Logo.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Card from "$lib/components/ui/Card.svelte";
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

<div class="mx-auto flex min-h-screen max-w-md flex-col justify-center px-6 py-12">
  <div class="mb-10 text-center">
    <div class="mb-6 flex justify-center">
      <Logo />
    </div>
    <h1 class="font-display text-3xl font-bold">Sign in with Steam</h1>
    <p class="mt-2 text-sm text-[var(--color-muted)]">
      Your Steam identity powers invites, standings, and result matching.
    </p>
  </div>

  <Card padding="lg" class="space-y-5">
    {#if error}
      <div
        class="rounded-xl border border-[color-mix(in_srgb,var(--color-racing)_40%,transparent)] bg-[color-mix(in_srgb,var(--color-racing)_10%,transparent)] px-4 py-3 text-sm text-[var(--color-racing)]"
      >
        {error}
      </div>
    {/if}

    <Button variant="steam" size="lg" class="w-full" {loading} onclick={loginSteam}>
      <svg class="size-5" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
        <path
          d="M12 2C6.48 2 2 6.28 2 11.63c0 3.07 1.58 5.77 4 7.33V20l3.68-2.02c.98.27 2.02.42 3.1.42 5.52 0 10-4.28 10-9.63S17.52 2 12 2zm5.2 13.3c-.66 0-1.2-.53-1.2-1.2s.54-1.2 1.2-1.2 1.2.53 1.2 1.2-.54 1.2-1.2 1.2zm-3.5-4.8c-1.66 0-3 1.34-3 3s1.34 3 3 3 3-1.34 3-3-1.34-3-3-3z"
        />
      </svg>
      Sign in with Steam
    </Button>

    <p class="text-center text-xs text-[var(--color-muted)]">
      Opens your browser for secure Steam OpenID login.
    </p>

    {#if dev}
      <div class="border-t border-[var(--color-carbon-border)] pt-5">
        <p class="mb-2 text-xs font-semibold uppercase tracking-wider text-[var(--color-muted)]">
          Dev login
        </p>
        <Input bind:value={devSteamId} placeholder="SteamID64" />
        <Button variant="ghost" size="sm" class="mt-2 w-full" {loading} onclick={loginDev}>
          Quick dev sign-in
        </Button>
      </div>
    {/if}
  </Card>
</div>
