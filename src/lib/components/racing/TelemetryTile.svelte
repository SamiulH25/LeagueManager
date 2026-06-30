<script lang="ts">
  interface Props {
    label: string;
    value: string | number;
    unit?: string;
    accent?: "red" | "yellow" | "green" | "blue" | "dim";
    large?: boolean;
  }

  let { label, value, unit = "", accent = "dim", large = false }: Props = $props();

  const accentColors = {
    red: "text-[var(--color-red)]",
    yellow: "text-[var(--color-yellow)]",
    green: "text-[var(--color-green)]",
    blue: "text-[var(--color-blue)]",
    dim: "text-[var(--color-muted)]",
  };
</script>

<div class="panel-timing rounded-md p-3">
  <p class="font-label text-[0.6rem] text-[var(--color-dim)]">{label}</p>
  <p
    class="font-mono font-bold tabular-nums {large ? 'text-3xl' : 'text-xl'} {accentColors[accent]}"
  >
    {value}{#if unit}<span class="ml-1 text-sm font-normal text-[var(--color-dim)]">{unit}</span
      >{/if}
  </p>
</div>
