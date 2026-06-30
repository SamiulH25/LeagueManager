<script lang="ts">
  interface Props {
    size?: "sm" | "md" | "lg";
    showSubtitle?: boolean;
  }

  let { size = "md", showSubtitle = false }: Props = $props();

  const sizes = {
    sm: { mark: "h-8 w-10 text-xs", title: "text-xl", sub: "text-[0.6rem]" },
    md: { mark: "h-10 w-12 text-sm", title: "text-2xl", sub: "text-[0.65rem]" },
    lg: { mark: "h-14 w-16 text-base", title: "text-4xl", sub: "text-xs" },
  };
</script>

<div class="flex items-center gap-3">
  <div class="relative shrink-0 {sizes[size].mark}">
    <div class="checkered absolute inset-0 rounded-sm opacity-90"></div>
    <div
      class="absolute inset-0 flex items-center justify-center font-display font-bold text-black {sizes[size].mark}"
    >
      LM
    </div>
    <div
      class="absolute -bottom-0.5 -right-0.5 h-2 w-2 rounded-full bg-[var(--color-red)] shadow-[0_0_8px_var(--color-red-glow)]"
    ></div>
  </div>
  <div class="leading-none">
    <div class="font-display {sizes[size].title} tracking-wider text-white">
      LEAGUE<span class="text-[var(--color-red)]">MANAGER</span>
    </div>
    {#if showSubtitle}
      <p class="font-label mt-1 {sizes[size].sub} text-[var(--color-muted)]">
        Assetto Corsa · Content Manager
      </p>
    {/if}
  </div>
</div>
