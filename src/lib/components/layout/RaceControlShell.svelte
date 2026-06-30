<script lang="ts">
  import Logo from "$lib/components/brand/Logo.svelte";
  import Avatar from "$lib/components/ui/Avatar.svelte";
  import type { DriverProfile } from "$lib/types";
  import { Flag, LayoutGrid, Radio, Server, Settings, Trophy, Users } from "@lucide/svelte";
  import type { Component } from "svelte";

  interface NavItem {
    href: string;
    label: string;
    code: string;
    icon: Component;
  }

  interface Props {
    mode: "host" | "driver";
    session: DriverProfile | null;
    activeHref?: string;
    children?: import("svelte").Snippet;
    header?: import("svelte").Snippet;
    onLogout?: () => void;
  }

  let {
    mode,
    session,
    activeHref = "",
    children,
    header,
    onLogout,
  }: Props = $props();

  const hostNav: NavItem[] = [
    { href: "/host", label: "Race Control", code: "RC", icon: Radio },
    { href: "/host/leagues", label: "Championships", code: "CH", icon: Trophy },
    { href: "/host/drivers", label: "Entry List", code: "EL", icon: Users },
    { href: "/host/settings", label: "Pit Config", code: "PC", icon: Settings },
  ];

  const driverNav: NavItem[] = [
    { href: "/driver", label: "Paddock", code: "PD", icon: Flag },
    { href: "/driver/leagues", label: "Standings", code: "ST", icon: LayoutGrid },
    { href: "/driver/settings", label: "Garage", code: "GR", icon: Settings },
  ];

  const nav = $derived(mode === "host" ? hostNav : driverNav);
</script>

<div class="flex min-h-screen flex-col">
  <!-- Top timing bar -->
  <header
    class="panel-timing-header flex shrink-0 items-center justify-between gap-4 border-b border-[var(--color-line)] px-4 py-2"
  >
    <div class="flex items-center gap-4">
      <div class="pit-stripe h-8 w-1.5 rounded-full"></div>
      <Logo size="sm" />
      <div class="hidden h-6 w-px bg-[var(--color-line)] sm:block"></div>
      <span class="font-label hidden text-xs text-[var(--color-muted)] sm:inline">
        {mode === "host" ? "Race Control · Host" : "Driver Paddock"}
      </span>
    </div>

    <div class="flex items-center gap-6">
      <div class="hidden items-center gap-2 md:flex">
        <span class="status-led status-led--idle"></span>
        <span class="font-label text-[0.65rem] text-[var(--color-muted)]">Server idle</span>
      </div>
      <div class="font-mono text-sm tabular-nums text-[var(--color-dim)]">
        {new Date().toLocaleTimeString([], { hour: "2-digit", minute: "2-digit", second: "2-digit" })}
      </div>
      {#if session}
        <div class="flex items-center gap-2 border-l border-[var(--color-line)] pl-4">
          <Avatar src={session.avatarUrl} alt={session.personaname} size="sm" />
          <span class="hidden max-w-[8rem] truncate text-sm font-medium sm:inline"
            >{session.personaname}</span
          >
        </div>
      {/if}
    </div>
  </header>

  <div class="flex min-h-0 flex-1">
    <!-- Pit lane nav -->
    <aside
      class="flex w-16 shrink-0 flex-col border-r border-[var(--color-line)] bg-[var(--color-asphalt)] lg:w-52"
    >
      <nav class="flex flex-1 flex-col gap-0.5 p-2">
        {#each nav as item}
          {@const Icon = item.icon}
          {@const active =
            activeHref === item.href ||
            (item.href !== (mode === "host" ? "/host" : "/driver") &&
              activeHref.startsWith(item.href))}
          <a
            href={item.href}
            class="group flex items-center gap-3 rounded-md px-2 py-2.5 transition-colors {active
              ? 'bg-[color-mix(in_srgb,var(--color-red)_15%,transparent)] text-white'
              : 'text-[var(--color-muted)] hover:bg-white/5 hover:text-white'}"
            title={item.label}
          >
            <span
              class="font-mono text-[0.6rem] font-bold {active
                ? 'text-[var(--color-red)]'
                : 'text-[var(--color-dim)]'} hidden lg:inline">{item.code}</span
            >
            <Icon class="size-5 shrink-0 {active ? 'text-[var(--color-red)]' : ''}" strokeWidth={1.75} />
            <span class="hidden truncate text-sm font-medium lg:inline">{item.label}</span>
          </a>
        {/each}
      </nav>

      <div class="border-t border-[var(--color-line)] p-2">
        <button
          type="button"
          class="flex w-full items-center justify-center gap-2 rounded-md px-2 py-2 text-xs text-[var(--color-muted)] transition-colors hover:bg-white/5 hover:text-[var(--color-red)] lg:justify-start"
          onclick={onLogout}
        >
          <Server class="size-4" strokeWidth={1.75} />
          <span class="hidden lg:inline">Disconnect</span>
        </button>
      </div>
    </aside>

    <!-- Main race control viewport -->
    <main class="flex min-w-0 flex-1 flex-col overflow-hidden">
      {#if header}
        <div class="shrink-0 border-b border-[var(--color-line)] px-4 py-3 lg:px-6">
          {@render header()}
        </div>
      {/if}
      <div class="flex-1 overflow-y-auto p-4 lg:p-6">
        {@render children?.()}
      </div>
    </main>
  </div>
</div>
