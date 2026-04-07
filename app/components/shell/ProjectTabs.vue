<script setup lang="ts">
import type {TabsItem} from '@nuxt/ui'
import type {ProjectWorkspace} from '~/shared/types/models'

const props = defineProps<{
  tabs: ProjectWorkspace[]
  activeProjectId: string | null
}>()

const emit = defineEmits<{
  select: [projectId: string]
}>()

const items = computed<TabsItem[]>(() =>
    props.tabs.map((tab) => ({
      label: tab.name,
      value: tab.id,
      icon: 'i-lucide-languages',
      badge: tab.dirty ? '•' : undefined
    }))
)
</script>

<template>
  <UCard>
    <div class="flex items-center gap-3">
      <span class="text-sm font-medium text-muted whitespace-nowrap">
        {{ $t('labels.projects') }}
      </span>

      <div class="min-w-0 flex-1">
        <UTabs
            v-if="items.length"
            :model-value="activeProjectId || items[0]?.value"
            :items="items"
            :content="false"
            variant="link"
            class="w-full"
            @update:model-value="(value) => emit('select', String(value))"
        />
        <span v-else class="text-sm text-muted">
          {{ $t('empty.noProjects') }}
        </span>
      </div>
    </div>
  </UCard>
</template>
