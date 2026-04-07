<script setup lang="ts">
import type {TreemapNode} from '~/shared/types/models'

const props = defineProps<{
  nodes: TreemapNode[]
}>()

function ratio(node: TreemapNode) {
  if (!node.count) return 0
  return Math.round((node.translatedCount / node.count) * 100)
}
</script>

<template>
  <div class="grid gap-4 md:grid-cols-2 2xl:grid-cols-3">
    <UCard v-for="node in nodes" :key="node.id">
      <div class="space-y-4">
        <div class="flex items-start justify-between gap-3">
          <div class="min-w-0">
            <p class="font-medium truncate">{{ node.label }}</p>
            <p class="text-xs text-muted truncate">{{ node.path }}</p>
          </div>
          <UBadge color="neutral" variant="subtle">
            {{ node.count }}
          </UBadge>
        </div>

        <UProgress :model-value="ratio(node)"/>

        <div class="grid grid-cols-3 gap-3 text-sm">
          <div>
            <p class="text-xs text-muted">{{ $t('labels.translated') }}</p>
            <p class="font-semibold">{{ node.translatedCount }}</p>
          </div>
          <div>
            <p class="text-xs text-muted">{{ $t('labels.missing') }}</p>
            <p class="font-semibold">{{ node.missingCount }}</p>
          </div>
          <div>
            <p class="text-xs text-muted">{{ $t('labels.charVolume') }}</p>
            <p class="font-semibold">{{ node.charCount }}</p>
          </div>
        </div>
      </div>
    </UCard>
  </div>
</template>
