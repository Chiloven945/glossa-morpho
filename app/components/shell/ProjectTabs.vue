<script lang="ts" setup>
import type {ProjectWorkspace} from '#shared/types/models'

const props = defineProps<{
  tabs: ProjectWorkspace[]
  activeTabId: string
  homeTabId: string
}>()

const emit = defineEmits<{
  select: [tabId: string]
  close: [projectId: string]
  reorder: [projectId: string, targetProjectId: string]
}>()

const draggedTabId = ref<string | null>(null)

function isActive(tabId: string) {
  return props.activeTabId === tabId
}

function onDragStart(projectId: string, event: DragEvent) {
  draggedTabId.value = projectId
  event.dataTransfer?.setData('text/plain', projectId)
  if (event.dataTransfer) event.dataTransfer.effectAllowed = 'move'
}

function onDragEnd() {
  draggedTabId.value = null
}

function onDragOver(event: DragEvent) {
  event.preventDefault()
  if (event.dataTransfer) event.dataTransfer.dropEffect = 'move'
}

function onDrop(targetProjectId: string, event: DragEvent) {
  event.preventDefault()
  const projectId = draggedTabId.value || event.dataTransfer?.getData('text/plain') || ''
  if (!projectId || projectId === targetProjectId) return
  emit('reorder', projectId, targetProjectId)
  draggedTabId.value = null
}
</script>

<template>
  <div class="shrink-0 border-b border-default bg-default/95">
    <div class="flex min-w-0 items-end gap-1 overflow-x-auto px-2 pt-2">
      <button
          :class="[
          'flex h-11 w-11 shrink-0 items-center justify-center rounded-t-lg border border-b-0 transition',
          isActive(homeTabId) ? 'border-default bg-default text-primary' : 'border-transparent bg-elevated/60 text-muted hover:bg-elevated'
        ]"
          type="button"
          @click="emit('select', homeTabId)"
      >
        <UIcon class="size-5" name="i-lucide-house"/>
      </button>

      <button
          v-for="tab in tabs"
          :key="tab.id"
          :class="[
          'group flex h-11 min-w-0 max-w-80 shrink-0 items-center gap-2 rounded-t-lg border border-b-0 px-3 text-sm transition',
          isActive(tab.id) ? 'border-default bg-default text-highlighted' : 'border-transparent bg-elevated/60 text-toned hover:bg-elevated',
          draggedTabId === tab.id ? 'opacity-60' : ''
        ]"
          draggable="true"
          type="button"
          @click="emit('select', tab.id)"
          @dragend="onDragEnd"
          @dragover="onDragOver"
          @dragstart="onDragStart(tab.id, $event)"
          @drop="onDrop(tab.id, $event)"
      >
        <UIcon class="size-4 shrink-0" name="i-lucide-folder-kanban"/>
        <span class="truncate font-medium">{{ tab.name }}</span>
        <span v-if="tab.dirty" class="text-warning">•</span>
        <UButton
            class="ml-auto opacity-70 group-hover:opacity-100"
            color="neutral"
            icon="i-lucide-x"
            size="xs"
            square
            variant="ghost"
            @click.stop="emit('close', tab.id)"
        />
      </button>

      <div class="min-w-8 flex-1 border-b border-default"/>
    </div>
  </div>
</template>
