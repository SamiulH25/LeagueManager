<script lang="ts">
  interface Props {
    src: string;
    alt: string;
    size?: "sm" | "md" | "lg" | "xl";
    ring?: boolean;
  }

  let { src, alt, size = "md", ring = false }: Props = $props();

  const sizes = {
    sm: "size-8",
    md: "size-10",
    lg: "size-14",
    xl: "size-20",
  };
</script>

<img
  {src}
  {alt}
  class="rounded-full object-cover bg-[var(--color-carbon-elevated)] {sizes[size]} {ring
    ? 'ring-2 ring-[var(--color-racing)] ring-offset-2 ring-offset-[var(--color-carbon)]'
    : ''}"
/>
