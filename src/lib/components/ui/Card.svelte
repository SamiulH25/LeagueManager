<script lang="ts">
  interface Props {
    class?: string;
    glow?: boolean;
    padding?: "sm" | "md" | "lg";
    children?: import("svelte").Snippet;
  }

  let {
    class: className = "",
    glow = false,
    padding = "md",
    children,
  }: Props = $props();

  const paddings = {
    sm: "p-4",
    md: "p-6",
    lg: "p-8",
  };
</script>

<div
  class="glass-card rounded-2xl {paddings[padding]} {glow ? 'glow-racing' : ''} {className}"
>
  {@render children?.()}
</div>
