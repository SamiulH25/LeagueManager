<script lang="ts">
  import { goto } from "$app/navigation";
  import { getContext } from "svelte";
  import type { AppStore } from "$lib/stores/app.svelte";

  const store = getContext<AppStore>("app");

  $effect(() => {
    if (store.loading || !store.state) return;

    const s = store.state;
    if (!s.session) {
      goto("/login");
      return;
    }
    if (!s.onboardingComplete || !s.appMode) {
      goto("/welcome");
      return;
    }
    goto(s.appMode === "host" ? "/host" : "/driver");
  });
</script>

<div class="grid min-h-screen place-items-center">
  <p class="text-sm text-[var(--color-muted)]">Loading LeagueManager…</p>
</div>
