<script lang="ts">
  interface Props {
    src: string;
    alt: string;
    size?: "sm" | "md" | "lg" | "xl";
    number?: number | null;
  }

  let { src, alt, size = "md", number = null }: Props = $props();

  const sizes = {
    sm: "size-8",
    md: "size-10",
    lg: "size-14",
    xl: "size-20",
  };
</script>

<div class="relative shrink-0">
  <img
    {src}
    {alt}
    class="rounded-sm object-cover bg-[var(--color-asphalt)] ring-1 ring-[var(--color-line)] {sizes[size]}"
  />
  {#if number !== null}
    <span
      class="absolute -bottom-1 -right-1 grid min-w-[1.25rem] place-items-center rounded-sm bg-[var(--color-red)] px-1 font-mono text-[0.6rem] font-bold text-white"
    >
      {number}
    </span>
  {/if}
</div>
