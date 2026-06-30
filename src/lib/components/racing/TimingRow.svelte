<script lang="ts">
  interface Props {
    position: number;
    name: string;
    meta?: string;
    stat?: string;
    statLabel?: string;
    highlight?: boolean;
  }

  let {
    position,
    name,
    meta = "",
    stat = "",
    statLabel = "",
    highlight = false,
  }: Props = $props();
</script>

<div class="timing-row px-3 py-2.5 {highlight ? 'bg-[color-mix(in_srgb,var(--color-yellow)_6%,transparent)]' : ''}">
  <span class="pos-badge {position === 1 ? 'pos-badge--lead' : ''}">P{position}</span>
  <div class="min-w-0">
    <p class="truncate font-medium">{name}</p>
    {#if meta}
      <p class="truncate font-mono text-xs text-[var(--color-dim)]">{meta}</p>
    {/if}
  </div>
  {#if stat}
    <div class="hidden text-right sm:block">
      <p class="font-mono text-sm tabular-nums text-[var(--color-text)]">{stat}</p>
      {#if statLabel}
        <p class="font-label text-[0.55rem] text-[var(--color-dim)]">{statLabel}</p>
      {/if}
    </div>
  {/if}
</div>
