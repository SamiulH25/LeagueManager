<script lang="ts">
  interface Props {
    size?: "sm" | "md" | "lg";
  }

  let { size = "md" }: Props = $props();

  const sizes = {
    sm: "text-lg gap-2",
    md: "text-2xl gap-3",
    lg: "text-3xl gap-3",
  };
</script>

<div class="flex items-center {sizes[size]} font-display font-bold tracking-tight">
  <div
    class="grid size-9 place-items-center rounded-lg bg-gradient-to-br from-[var(--color-racing)] to-[var(--color-amber)] text-sm font-black text-white shadow-lg"
    aria-hidden="true"
  >
    LM
  </div>
  <span class="text-gradient-racing">LeagueManager</span>
</div>
