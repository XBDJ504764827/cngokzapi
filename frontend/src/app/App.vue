<script setup lang="ts">
import { NConfigProvider, NMessageProvider } from 'naive-ui'
import DefaultLayout from '@/layouts/DefaultLayout.vue'
import { themeOverrides } from './theme'
</script>

<template>
  <NConfigProvider :theme-overrides="themeOverrides">
    <NMessageProvider>
      <DefaultLayout />
    </NMessageProvider>
  </NConfigProvider>
</template>
