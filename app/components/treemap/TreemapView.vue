<script lang="ts" setup>
import type {TreemapNode} from '#shared/types/models'

const {t} = useI18n()

const props = defineProps<{
  nodes: TreemapNode[]
}>()

const maxCount = computed(() => Math.max(...props.nodes.map(node => node.count), 1))
</script>

<template>
  <div class="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
    <UCard v-for="node in nodes" :key="node.id" variant="subtle">
      <div class="space-y-3">
        <div class="flex items-start justify-between gap-3">
          <div>
            <p class="font-medium">{{ node.label }}</p>
            <p class="text-xs text-muted">{{ node.path }}</p>
          </div>
          <UBadge color="primary" variant="soft">{{ node.count }}</UBadge>
        </div>

        <div class="space-y-2">
          <UProgress :model-value="Math.round((node.count / maxCount) * 100)"/>
          <div class="grid gap-2 sm:grid-cols-3 text-xs">
            <UCard variant="ghost">
              <div class="space-y-1">
                <p class="text-muted">{{ t('labels.translated') }}</p>
                <p class="font-semibold">{{ node.translatedCount }}</p>
              </div>
            </UCard>
            <UCard variant="ghost">
              <div class="space-y-1">
                <p class="text-muted">{{ t('labels.missing') }}</p>
                <p class="font-semibold">{{ node.missingCount }}</p>
              </div>
            </UCard>
            <UCard variant="ghost">
              <div class="space-y-1">
                <p class="text-muted">{{ t('labels.characters') }}</p>
                <p class="font-semibold">{{ node.charCount }}</p>
              </div>
            </UCard>
          </div>
        </div>
      </div>
    </UCard>
  </div>
</template>
