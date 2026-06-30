<div class="pointer-events-none fixed inset-0 -z-10 overflow-hidden" aria-hidden="true">
  <!-- Asphalt gradient -->
  <div
    class="absolute inset-0"
    style="background: radial-gradient(ellipse 80% 60% at 70% 0%, color-mix(in srgb, var(--color-red) 8%, transparent), transparent 60%), radial-gradient(ellipse 50% 40% at 0% 100%, color-mix(in srgb, var(--color-blue) 5%, transparent), transparent), var(--color-pit);"
  ></div>

  <!-- Track map ghost lines -->
  <svg
    class="absolute right-0 top-0 h-full w-2/3 opacity-[0.04]"
    viewBox="0 0 400 600"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
  >
    <path
      d="M320 80 C380 120 360 200 300 240 C240 280 180 260 140 320 C100 380 120 460 200 500 C280 540 340 520 360 440"
      stroke="white"
      stroke-width="24"
      stroke-linecap="round"
    />
    <path
      d="M300 100 C350 130 340 190 290 220 C240 250 200 240 170 290 C140 340 150 400 210 430"
      stroke="white"
      stroke-width="3"
      stroke-dasharray="8 12"
    />
  </svg>

  <!-- Subtle scanlines -->
  <div class="scanlines absolute inset-0 opacity-30"></div>
</div>
