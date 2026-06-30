<script lang="ts">
  interface Props {
    variant?: "primary" | "secondary" | "ghost" | "steam";
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
      "bg-gradient-to-r from-[var(--color-racing)] to-[var(--color-amber)] text-white hover:brightness-110 shadow-lg shadow-[color-mix(in_srgb,var(--color-racing)_30%,transparent)]",
    secondary:
      "bg-[var(--color-carbon-elevated)] border border-[var(--color-carbon-border)] text-[var(--color-text)] hover:border-[color-mix(in_srgb,var(--color-racing)_40%,var(--color-carbon-border))]",
    ghost: "bg-transparent text-[var(--color-muted)] hover:text-white hover:bg-white/5",
    steam:
      "bg-[#171a21] border border-[#2a475e] text-white hover:bg-[#1b2838] hover:border-[#66c0f4]",
  };

  const sizes = {
    sm: "px-3 py-1.5 text-sm rounded-lg",
    md: "px-5 py-2.5 text-sm rounded-xl",
    lg: "px-6 py-3.5 text-base rounded-xl",
  };
</script>

<button
  {type}
  class="inline-flex items-center justify-center gap-2 font-medium transition-all duration-200 active:scale-[0.98] disabled:opacity-50 disabled:pointer-events-none {variants[variant]} {sizes[size]} {className}"
  disabled={disabled || loading}
  {onclick}
>
  {#if loading}
    <span
      class="size-4 animate-spin rounded-full border-2 border-white/30 border-t-white"
    ></span>
  {/if}
  {@render children?.()}
</button>
