<script lang="ts" setup>
import type {EntryDetail, EntrySummary, ProjectWorkspace, TranslationStatus, ViewMode} from '#shared/types/models'
import {storeToRefs} from 'pinia'

const props = defineProps<{
  activeProject: ProjectWorkspace
  filteredEntries: EntrySummary[]
  selectedFileId: string | null
  selectedEntryId: string | null
  selectedEntryIds: string[]
  selectedEntry?: EntryDetail
  searchText: string
  statusFilter: TranslationStatus | 'all'
  sortBy: 'updatedDesc' | 'keyAsc' | 'status'
  currentView: ViewMode
  showOnlyMissing: boolean
}>()

const emit = defineEmits<{
  selectFile: [fileId: string | null]
  selectEntry: [payload: { entryId: string; append?: boolean; range?: boolean; orderedIds?: string[] }]
  selectAllEntries: [entryIds: string[]]
  changeView: [view: ViewMode]
  changeShowOnlyMissing: [value: boolean]
  updateSortBy: [value: 'updatedDesc' | 'keyAsc' | 'status']
  updateStatusFilter: [value: TranslationStatus | 'all']
  saveEntry: [payload: { targetValue: string; note: string; status: TranslationStatus }]
  createEntry: [fileId: string]
  createFile: [localeCode?: string | null]
  renameFile: [fileId: string]
  deleteFile: [fileId: string]
  deleteEntry: [entryId: string]
  deleteEntries: [entryIds: string[]]
  importFiles: []
  exportFiles: [fileId: string | null]
  batchExport: []
}>()

const editor = useEditorStore()
const {panelVisibility} = storeToRefs(editor)

const leftWidth = ref(320)
const rightWidth = ref(420)
const minLeftWidth = 260
const maxLeftWidth = 520
const minRightWidth = 320
const maxRightWidth = 620
const dividerWidth = 8

const resizeState = reactive({
  side: null as 'left' | 'right' | null,
  startX: 0,
  startLeftWidth: 320,
  startRightWidth: 420
})

const selectedFile = computed(() => props.activeProject.files.find((file) => file.id === props.selectedFileId) || null)
const showFiles = computed(() => panelVisibility.value.files)
const showEntries = computed(() => panelVisibility.value.entries)
const showInspector = computed(() => panelVisibility.value.inspector)
const visiblePanelCount = computed(() => Number(showFiles.value) + Number(showEntries.value) + Number(showInspector.value))
const filesWidthStyle = computed(() => visiblePanelCount.value === 1 ? undefined : ({width: `${leftWidth.value}px`}))
const inspectorWidthStyle = computed(() => visiblePanelCount.value === 1 ? undefined : ({width: `${rightWidth.value}px`}))
const showLeftDivider = computed(() => showFiles.value && (showEntries.value || showInspector.value))
const showRightDivider = computed(() => showEntries.value && showInspector.value)
const filesPanelClass = computed(() => visiblePanelCount.value === 1 ? 'min-h-0 min-w-0 flex-1 overflow-hidden' : 'min-h-0 shrink-0 overflow-hidden')
const entriesPanelClass = computed(() => visiblePanelCount.value === 1 ? 'min-h-0 min-w-0 flex-1 overflow-hidden' : 'min-h-0 min-w-0 flex-1 overflow-hidden')
const inspectorPanelClass = computed(() => visiblePanelCount.value === 1 ? 'min-h-0 min-w-0 flex-1 overflow-hidden' : (showEntries.value ? 'min-h-0 shrink-0 overflow-hidden' : 'min-h-0 min-w-0 flex-1 overflow-hidden'))

const panelButtons = computed(() => [
  {id: 'files', icon: 'i-lucide-folder-tree', active: showFiles.value, title: 'Project Files'},
  {id: 'entries', icon: 'i-lucide-list-tree', active: showEntries.value, title: 'Entries'},
  {id: 'inspector', icon: 'i-lucide-panel-right', active: showInspector.value, title: 'Inspector'}
])

function clamp(value: number, min: number, max: number) {
  return Math.min(max, Math.max(min, value))
}

function startResize(side: 'left' | 'right', event: MouseEvent) {
  resizeState.side = side
  resizeState.startX = event.clientX
  resizeState.startLeftWidth = leftWidth.value
  resizeState.startRightWidth = rightWidth.value
  document.body.style.userSelect = 'none'
  window.addEventListener('mousemove', onResize)
  window.addEventListener('mouseup', stopResize)
}

function onResize(event: MouseEvent) {
  if (!resizeState.side) return
  const delta = event.clientX - resizeState.startX
  if (resizeState.side === 'left') {
    leftWidth.value = clamp(resizeState.startLeftWidth + delta, minLeftWidth, maxLeftWidth)
    return
  }
  rightWidth.value = clamp(resizeState.startRightWidth - delta, minRightWidth, maxRightWidth)
}

function stopResize() {
  resizeState.side = null
  document.body.style.userSelect = ''
  window.removeEventListener('mousemove', onResize)
  window.removeEventListener('mouseup', stopResize)
}

onBeforeUnmount(() => {
  stopResize()
})
</script>

<template>
  <div class="flex h-full min-h-0">
    <aside class="flex w-12 shrink-0 flex-col items-center gap-2 border-r border-default bg-elevated/20 px-1 py-3">
      <UTooltip v-for="button in panelButtons" :key="button.id" :text="button.title">
        <UButton
            :color="button.active ? 'primary' : 'neutral'"
            :icon="button.icon"
            :variant="button.active ? 'soft' : 'ghost'"
            class="h-9 w-9 justify-center"
            @click="editor.togglePanel(button.id as 'files' | 'entries' | 'inspector')"
        />
      </UTooltip>
    </aside>

    <div class="flex min-w-0 flex-1 min-h-0">
      <div v-if="showFiles" :class="filesPanelClass" :style="filesWidthStyle">
        <FileTreePane
            :project="activeProject"
            :selected-file-id="selectedFileId"
            class="h-full"
            @select="emit('selectFile', $event)"
            @create-entry="emit('createEntry', $event)"
            @create-file="emit('createFile', $event)"
            @rename-file="emit('renameFile', $event)"
            @delete-file="emit('deleteFile', $event)"
            @import-files="emit('importFiles')"
            @export-files="emit('exportFiles', $event)"
            @batch-export="emit('batchExport')"
        />
      </div>

      <div
          v-if="showLeftDivider"
          class="group flex min-h-0 w-2 shrink-0 cursor-col-resize items-stretch justify-center bg-elevated/20 transition hover:bg-primary/10"
          @mousedown.prevent="startResize('left', $event)"
      >
        <div class="my-4 w-px bg-default transition group-hover:bg-primary"/>
      </div>

      <div v-if="showEntries" :class="entriesPanelClass">
        <StringListPane
            :entries="filteredEntries"
            :project="activeProject"
            :search-text="searchText"
            :selected-entry-id="selectedEntryId"
            :selected-entry-ids="selectedEntryIds"
            :selected-file="selectedFile"
            :selected-file-id="selectedFileId"
            :show-only-missing="showOnlyMissing"
            :sort-by="sortBy"
            :status-filter="statusFilter"
            :view-mode="currentView"
            class="h-full"
            @change-show-only-missing="emit('changeShowOnlyMissing', $event)"
            @change-view="emit('changeView', $event)"
            @select-entry="emit('selectEntry', $event)"
            @select-all-entries="emit('selectAllEntries', $event)"
            @update-sort-by="emit('updateSortBy', $event)"
            @update-status-filter="emit('updateStatusFilter', $event)"
            @create-entry="emit('createEntry', $event)"
            @delete-entry="emit('deleteEntry', $event)"
            @delete-entries="emit('deleteEntries', $event)"
        />
      </div>

      <div
          v-if="showRightDivider"
          class="group flex min-h-0 w-2 shrink-0 cursor-col-resize items-stretch justify-center bg-elevated/20 transition hover:bg-primary/10"
          @mousedown.prevent="startResize('right', $event)"
      >
        <div class="my-4 w-px bg-default transition group-hover:bg-primary"/>
      </div>

      <div v-if="showInspector" :class="inspectorPanelClass" :style="showEntries ? inspectorWidthStyle : undefined">
        <EntryInspector :entry="selectedEntry" class="h-full" @save="emit('saveEntry', $event)"/>
      </div>
    </div>
  </div>
</template>
