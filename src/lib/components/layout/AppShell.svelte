<script lang="ts">
  import Logo from "$lib/components/brand/Logo.svelte";
  import Avatar from "$lib/components/ui/Avatar.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import type { DriverProfile } from "$lib/types";

  interface NavItem {
    href: string;
    label: string;
    icon: string;
  }

  interface Props {
    mode: "host" | "driver";
    session: DriverProfile | null;
    nav: NavItem[];
    children?: import("svelte").Snippet;
    onLogout?: () => void;
  }

  let { mode, session, nav, children, onLogout }: Props = $props();
</script>

<div class="flex min-h-screen">
  <aside
    class="flex w-64 shrink-0 flex-col border-r border-[var(--color-carbon-border)] bg-[color-mix(in_srgb,var(--color-carbon-elevated)_80%,transparent)] backdrop-blur-xl"
  >
    <div class="border-b border-[var(--color-carbon-border)] p-6">
      <Logo size="sm" />
      <p class="mt-3 text-xs font-semibold uppercase tracking-widest text-[var(--color-muted)]">
        {mode === "host" ? "Host console" : "Driver hub"}
      </p>
    </div>

    <nav class="flex flex-1 flex-col gap-1 p-4">
      {#each nav as item}
        <a
          href={item.href}
          class="flex items-center gap-3 rounded-xl px-3 py-2.5 text-sm text-[var(--color-muted)] transition-colors hover:bg-white/5 hover:text-white"
        >
          <span class="text-base opacity-70">{item.icon}</span>
          {item.label}
        </a>
      {/each}
    </nav>

    {#if session}
      <div class="border-t border-[var(--color-carbon-border)] p-4">
        <div class="mb-3 flex items-center gap-3">
          <Avatar src={session.avatarUrl} alt={session.personaname} size="sm" />
          <div class="min-w-0 flex-1">
            <p class="truncate text-sm font-medium">{session.personaname}</p>
            <p class="truncate text-xs text-[var(--color-muted)]">Steam connected</p>
          </div>
        </div>
        <Button variant="ghost" size="sm" class="w-full" onclick={onLogout}>
          Sign out
        </Button>
      </div>
    {/if}
  </aside>

  <main class="flex-1 overflow-y-auto p-8">
    {@render children?.()}
  </main>
</div>
