<script lang="ts" setup>
import type {EntrySummary, ProjectWorkspace} from '#shared/types/models'

const {t} = useI18n()

const props = defineProps<{
  activeProject?: ProjectWorkspace
  activeTabLabel: string
  selectedFileName?: string
  filteredEntries: EntrySummary[]
  lastSavedAt?: string
  isBusy?: boolean
}>()

const translatedRatio = computed(() => {
  if (!props.activeProject?.stats.total) return 0
  return Math.round((props.activeProject.stats.translated / props.activeProject.stats.total) * 100)
})
</script>

<template>
  <footer
      class="flex h-9 shrink-0 items-center justify-between gap-3 border-t border-default bg-elevated/30 px-3 text-xs text-muted">
    <div class="flex min-w-0 items-center gap-3 overflow-hidden">
      <span class="truncate">{{ t('labels.currentTab') }}: {{ activeTabLabel }}</span>
      <span class="truncate">{{ t('labels.currentFile') }}: {{ selectedFileName || t('labels.allFiles') }}</span>
      <span v-if="activeProject">{{ t('labels.filterHits') }}: {{ filteredEntries.length }}</span>
      <span v-if="activeProject">{{ t('labels.coverage') }}: {{ translatedRatio }}%</span>
    </div>
    <div class="flex items-center gap-3 whitespace-nowrap">
      <span>{{ isBusy ? t('app.busy') : 'Ready' }}</span>
      <span>{{
          lastSavedAt ? `${t('labels.lastSaved')}: ${new Date(lastSavedAt).toLocaleTimeString()}` : t('labels.notSavedYet')
        }}</span>
    </div>
  </footer>
</template>
