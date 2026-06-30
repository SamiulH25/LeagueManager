<script lang="ts">
  interface Props {
    value?: string;
    placeholder?: string;
    type?: string;
    class?: string;
    mono?: boolean;
  }

  let {
    value = $bindable(""),
    placeholder = "",
    type = "text",
    class: className = "",
    mono = false,
  }: Props = $props();
</script>

<input
  {type}
  bind:value
  {placeholder}
  class="w-full rounded-md border border-[var(--color-line)] bg-[var(--color-asphalt)] px-3 py-2.5 text-sm text-white placeholder:text-[var(--color-dim)] outline-none transition-colors focus:border-[var(--color-red)] focus:ring-1 focus:ring-[color-mix(in_srgb,var(--color-red)_30%,transparent)] {mono
    ? 'font-mono'
    : ''} {className}"
/>
