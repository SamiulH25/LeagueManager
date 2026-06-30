<script lang="ts">
  interface Props {
    variant?: "default" | "live" | "amber" | "muted";
    class?: string;
    children?: import("svelte").Snippet;
  }

  let { variant = "default", class: className = "", children }: Props = $props();

  const variants = {
    default: "bg-white/5 text-[var(--color-muted)] border-[var(--color-carbon-border)]",
    live: "bg-[color-mix(in_srgb,var(--color-live)_15%,transparent)] text-[var(--color-live)] border-[color-mix(in_srgb,var(--color-live)_35%,transparent)]",
    amber: "bg-[color-mix(in_srgb,var(--color-amber)_12%,transparent)] text-[var(--color-amber)] border-[color-mix(in_srgb,var(--color-amber)_30%,transparent)]",
    muted: "bg-black/20 text-[var(--color-muted)] border-transparent",
  };
</script>

<span
  class="inline-flex items-center gap-1.5 rounded-full border px-2.5 py-0.5 text-xs font-semibold uppercase tracking-wider {variants[variant]} {className}"
>
  {@render children?.()}
</span>
