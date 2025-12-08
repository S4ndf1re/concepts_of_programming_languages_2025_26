<template>
  <footer
    v-if="$slidev.nav.currentLayout !== 'cover' && $slidev.nav.currentLayout !== 'end'"
    class="absolute bottom-0 w-full flex justify-between p-4 text-xs"
  >
  <div class="opacity-75">
      16.12.2025 | Jan Kleinmann, Marius Braun
    </div>
    <div class="opacity-50">
      <SlideCurrentNo /> / <SlidesTotal />
    </div>
  </footer>
</template>