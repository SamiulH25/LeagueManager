<script lang="ts">
  interface Props {
    value?: string;
    placeholder?: string;
    type?: string;
    class?: string;
  }

  let {
    value = $bindable(""),
    placeholder = "",
    type = "text",
    class: className = "",
  }: Props = $props();
</script>

<input
  {type}
  bind:value
  {placeholder}
  class="w-full rounded-xl border border-[var(--color-carbon-border)] bg-[var(--color-carbon-elevated)] px-4 py-3 text-sm text-white placeholder:text-[var(--color-muted)] outline-none transition-colors focus:border-[color-mix(in_srgb,var(--color-racing)_50%,var(--color-carbon-border))] focus:ring-2 focus:ring-[color-mix(in_srgb,var(--color-racing)_20%,transparent)] {className}"
/>
