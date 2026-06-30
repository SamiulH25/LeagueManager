<script lang="ts">
  interface Props {
    variant?: "primary" | "secondary" | "ghost" | "steam" | "green";
    size?: "sm" | "md" | "lg";
    disabled?: boolean;
    loading?: boolean;
    type?: "button" | "submit";
    class?: string;
    onclick?: (e: MouseEvent) => void;
    children?: import("svelte").Snippet;
  }

  let {
    variant = "primary",
    size = "md",
    disabled = false,
    loading = false,
    type = "button",
    class: className = "",
    onclick,
    children,
  }: Props = $props();

  const variants = {
    primary:
      "bg-[var(--color-red)] text-white hover:bg-[var(--color-red-glow)] border border-[color-mix(in_srgb,var(--color-red)_80%,white)] shadow-[0_0_20px_color-mix(in_srgb,var(--color-red)_25%,transparent)]",
    green:
      "bg-[color-mix(in_srgb,var(--color-green)_20%,var(--color-panel))] text-[var(--color-green)] border border-[color-mix(in_srgb,var(--color-green)_40%,transparent)] hover:bg-[color-mix(in_srgb,var(--color-green)_30%,var(--color-panel))]",
    secondary:
      "bg-[var(--color-panel-2)] border border-[var(--color-line)] text-[var(--color-text)] hover:border-[var(--color-line-bright)]",
    ghost: "bg-transparent text-[var(--color-muted)] hover:text-white hover:bg-white/5",
    steam:
      "bg-[#171a21] border border-[#2a475e] text-white hover:bg-[#1b2838] hover:border-[#66c0f4]",
  };

  const sizes = {
    sm: "px-3 py-1.5 text-xs rounded font-label tracking-wider",
    md: "px-4 py-2 text-sm rounded-md font-label tracking-wider",
    lg: "px-6 py-3 text-sm rounded-md font-label tracking-widest",
  };
</script>

<button
  {type}
  class="inline-flex items-center justify-center gap-2 font-semibold uppercase transition-all duration-150 active:scale-[0.98] disabled:opacity-40 disabled:pointer-events-none {variants[variant]} {sizes[size]} {className}"
  disabled={disabled || loading}
  {onclick}
>
  {#if loading}
    <span
      class="size-3.5 animate-spin rounded-full border-2 border-white/20 border-t-white"
    ></span>
  {/if}
  {@render children?.()}
</button>
