// 玩家单元格：头像 + 名称 + SteamID64，列表/榜单/档案共用（第二个使用者出现后下沉）。

<script setup lang="ts">
import { computed } from 'vue'
import { NAvatar } from 'naive-ui'
import { RouterLink } from 'vue-router'
import { avatarUrl } from '@/utils/avatar'

const props = defineProps<{
  player: string
  steamId64: string
  avatar?: string
  /** 是否把名称链到个人资料页 */
  link?: boolean
}>()

// 头像：有真实头像用真实的，否则用按 SteamID64 生成的占位图；图片失败时退回昵称首字
const src = computed(() => avatarUrl(props.steamId64, props.avatar))
const initial = computed(() => (props.player ? props.player.trim().charAt(0) : '?'))
</script>

<template>
  <div class="player-cell">
    <NAvatar :src="src" :size="32" round>
      {{ initial }}
    </NAvatar>
    <div class="player-id">
      <div class="player-name">
        <RouterLink v-if="link !== false" :to="`/profile/${steamId64}`">{{ player }}</RouterLink>
        <template v-else>{{ player }}</template>
      </div>
      <div class="player-steam">{{ steamId64 }}</div>
    </div>
  </div>
</template>

<style scoped>
.player-cell {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
}

.player-id {
  min-width: 0;
}

.player-name {
  font-weight: 600;
  line-height: 1.4;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.player-name a {
  color: inherit;
  text-decoration: none;
}

.player-name a:hover {
  color: var(--accent);
}

.player-steam {
  font-size: 12px;
  color: var(--text-muted);
  font-family: var(--font-mono);
  white-space: nowrap;
}
</style>
