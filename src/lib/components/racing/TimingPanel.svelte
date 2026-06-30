<script lang="ts">
  interface Props {
    title: string;
    subtitle?: string;
    children?: import("svelte").Snippet;
    actions?: import("svelte").Snippet;
    variant?: "default" | "tower";
  }

  let { title, subtitle = "", children, actions, variant = "default" }: Props = $props();
</script>

<section
  class="panel-timing overflow-hidden rounded-md {variant === 'tower' ? 'col-span-full xl:col-span-2' : ''}"
>
  <header
    class="flex items-center justify-between gap-3 border-b border-[var(--color-line)] px-4 py-2.5"
  >
    <div>
      <h2 class="font-label text-sm tracking-widest text-white">{title}</h2>
      {#if subtitle}
        <p class="mt-0.5 text-xs text-[var(--color-muted)]">{subtitle}</p>
      {/if}
    </div>
    {#if actions}
      <div class="shrink-0">
        {@render actions()}
      </div>
    {/if}
  </header>
  <div class="p-4">
    {@render children?.()}
  </div>
</section>
