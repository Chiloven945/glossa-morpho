<script lang="ts" setup>
import TreemapView from '~/components/treemap/TreemapView.vue'
import type {DropdownMenuItem, TabsItem} from '@nuxt/ui'
import type {WorkspaceContextMenuItem} from '~/utils/workspace-actions'
import {buildContextMenuItems, parseActionValue} from '~/utils/workspace-actions'
import type {EntrySummary, ProjectWorkspace, ResourceFileNode, TranslationStatus, ViewMode} from '#shared/types/models'

const {t} = useI18n()

const props = defineProps<{
  project: ProjectWorkspace
  entries: EntrySummary[]
  selectedEntryId: string | null
  selectedEntryIds: string[]
  statusFilter: TranslationStatus | 'all'
  viewMode: ViewMode
  showOnlyMissing: boolean
  sortBy: 'updatedDesc' | 'keyAsc' | 'status'
  searchText: string
  selectedFileId?: string | null
  selectedFile?: ResourceFileNode | null
}>()

const emit = defineEmits<{
  updateStatusFilter: [value: TranslationStatus | 'all']
  selectEntry: [payload: { entryId: string; append?: boolean; range?: boolean; orderedIds?: string[] }]
  selectAllEntries: [entryIds: string[]]
  changeView: [view: ViewMode]
  changeShowOnlyMissing: [value: boolean]
  updateSortBy: [value: 'updatedDesc' | 'keyAsc' | 'status']
  createEntry: [fileId: string]
  deleteEntry: [entryId: string]
  deleteEntries: [entryIds: string[]]
}>()

const contextMenuOpen = ref(false)
const contextMenuItems = ref<WorkspaceContextMenuItem[]>([])
const contextX = ref(0)
const contextY = ref(0)

const hasFileSelection = computed(() => Boolean(props.selectedFileId && props.selectedFile))
const visibleStats = computed(() => {
  const total = props.entries.length
  const translated = props.entries.filter((entry) => !!entry.targetValue).length
  const missing = total - translated
  const reviewed = props.entries.filter((entry) => entry.status === 'reviewed' || entry.status === 'approved').length
  return {total, translated, missing, reviewed}
})

const headerMenuItems = computed<DropdownMenuItem[][]>(() => [[
  {
    label: t('actions.newEntry'),
    icon: 'i-lucide-file-plus-2',
    disabled: !props.selectedFileId,
    onSelect: () => props.selectedFileId && emit('createEntry', props.selectedFileId)
  },
  {
    label: t('actions.deleteSelectedEntries'),
    icon: 'i-lucide-trash-2',
    disabled: !props.selectedEntryIds.length,
    onSelect: () => props.selectedEntryIds.length && emit('deleteEntries', props.selectedEntryIds)
  }
]])

const viewItems = computed<TabsItem[]>(() => [
  {label: t('actions.list'), value: 'list', icon: 'i-lucide-list'},
  {label: t('actions.treemap'), value: 'treemap', icon: 'i-lucide-chart-no-axes-column'}
])

const statusItems = computed(() => [
  {label: t('labels.allStatuses'), value: 'all'},
  {label: t('status.new'), value: 'new'},
  {label: t('status.translated'), value: 'translated'},
  {label: t('status.reviewed'), value: 'reviewed'},
  {label: t('status.approved'), value: 'approved'},
  {label: t('status.stale'), value: 'stale'}
])

const sortItems = computed(() => [
  {label: t('labels.sortUpdatedDesc'), value: 'updatedDesc'},
  {label: t('labels.sortKeyAsc'), value: 'keyAsc'},
  {label: t('labels.sortStatus'), value: 'status'}
])

const allVisibleSelected = computed(() => Boolean(props.entries.length) && props.entries.every((entry) => props.selectedEntryIds.includes(entry.id)))

function statusColor(status: EntrySummary['status']) {
  if (status === 'approved') return 'success'
  if (status === 'reviewed') return 'info'
  if (status === 'translated') return 'primary'
  if (status === 'stale') return 'warning'
  return 'neutral'
}

function openMenu(event: MouseEvent, items: WorkspaceContextMenuItem[]) {
  if (!items.length) return
  event.preventDefault()
  event.stopPropagation()
  contextMenuItems.value = items
  contextX.value = event.clientX
  contextY.value = event.clientY
  contextMenuOpen.value = true
}

function entryMenu(entry: EntrySummary): WorkspaceContextMenuItem[] {
  const multiple = props.selectedEntryIds.length > 1 && props.selectedEntryIds.includes(entry.id)
  return buildContextMenuItems(t, [
    {id: 'entry.new', payload: entry.fileId},
    multiple
        ? {id: 'entry.deleteSelected', payload: props.selectedEntryIds.join(',')}
        : {id: 'entry.delete', payload: entry.id}
  ])
}

function canvasMenu(): WorkspaceContextMenuItem[] {
  return buildContextMenuItems(t, [
    {id: 'entry.new', payload: props.selectedFileId || undefined, disabled: !props.selectedFileId},
    {id: 'entry.deleteSelected', payload: props.selectedEntryIds.join(','), disabled: !props.selectedEntryIds.length}
  ])
}

function onMenuSelect(value: string) {
  const {actionId, payload} = parseActionValue(value)
  if (actionId === 'entry.new' && payload) emit('createEntry', payload)
  if (actionId === 'entry.delete' && payload) emit('deleteEntry', payload)
  if (actionId === 'entry.deleteSelected' && payload) emit('deleteEntries', payload.split(',').filter(Boolean))
}

function toggleEntrySelection(entry: EntrySummary) {
  emit('selectEntry', {
    entryId: entry.id,
    append: true,
    orderedIds: props.entries.map((item) => item.id)
  })
}

function handleRowPrimaryAction(entry: EntrySummary, event?: MouseEvent) {
  emit('selectEntry', {
    entryId: entry.id,
    append: Boolean(event?.ctrlKey || event?.metaKey),
    range: Boolean(event?.shiftKey),
    orderedIds: props.entries.map((item) => item.id)
  })
}

function onRowContextMenu(entry: EntrySummary, event: MouseEvent) {
  if (!props.selectedEntryIds.includes(entry.id)) {
    emit('selectEntry', {entryId: entry.id, orderedIds: props.entries.map((item) => item.id)})
  }
  openMenu(event, entryMenu(entry))
}

function toggleSelectAll(value: boolean | 'indeterminate') {
  if (value) emit('selectAllEntries', props.entries.map((entry) => entry.id))
  else emit('selectAllEntries', [])
}
</script>

<template>
  <UCard
      :ui="{ root: 'h-full min-h-0 rounded-none border-0 flex flex-col', body: 'min-h-0 flex-1 overflow-hidden p-0' }">
    <template #header>
      <div v-if="hasFileSelection" class="space-y-3" @mousedown.right.prevent.stop="openMenu($event, canvasMenu())"
           @contextmenu.prevent.stop="openMenu($event, canvasMenu())">
        <div class="flex items-start justify-between gap-3">
          <div class="min-w-0">
            <h2 class="font-semibold">{{ t('labels.entries') }}</h2>
            <p class="truncate text-sm text-muted">{{ selectedFile?.name }} · {{ entries.length }}
              {{ t('labels.visibleRows') }}<span v-if="selectedEntryIds.length"> · {{
                  selectedEntryIds.length
                }} {{ t('labels.selected') }}</span></p>
          </div>

          <div class="flex shrink-0 items-center gap-2">
            <UTabs :content="false" :items="viewItems" :model-value="viewMode"
                   @update:model-value="value => emit('changeView', String(value) as ViewMode)"/>
            <UDropdownMenu :items="headerMenuItems" :modal="false" :ui="{ content: 'w-56' }">
              <UButton color="neutral" icon="i-lucide-ellipsis" variant="ghost"/>
            </UDropdownMenu>
          </div>
        </div>

        <div class="flex flex-wrap items-center gap-2">
          <div class="w-40 min-w-0">
            <USelectMenu :items="statusItems" :model-value="statusFilter" value-key="value"
                         @update:model-value="value => emit('updateStatusFilter', String(value) as TranslationStatus | 'all')"/>
          </div>
          <div class="w-40 min-w-0">
            <USelectMenu :items="sortItems" :model-value="sortBy" value-key="value"
                         @update:model-value="value => emit('updateSortBy', String(value) as 'updatedDesc' | 'keyAsc' | 'status')"/>
          </div>
          <UCheckbox :label="t('labels.missingOnly')" :model-value="showOnlyMissing"
                     @update:model-value="value => emit('changeShowOnlyMissing', Boolean(value))"/>
          <UBadge v-if="searchText" color="neutral" variant="subtle">{{ t('labels.find') }}: {{ searchText }}</UBadge>
          <UBadge color="neutral" variant="subtle">{{ visibleStats.total }} {{ t('labels.total') }}</UBadge>
          <UBadge color="primary" variant="soft">{{ visibleStats.translated }} {{ t('labels.translated') }}</UBadge>
          <UBadge color="warning" variant="soft">{{ visibleStats.missing }} {{ t('labels.missing') }}</UBadge>
          <UBadge color="info" variant="soft">{{ visibleStats.reviewed }} {{ t('labels.reviewed') }}</UBadge>
        </div>
      </div>
    </template>

    <div v-if="!hasFileSelection" class="flex h-full items-center justify-center px-8 py-10">
      <div class="max-w-sm space-y-2 text-center">
        <UIcon class="mx-auto h-8 w-8 text-muted" name="i-lucide-list-tree"/>
        <p class="text-base font-medium">{{ $t('empty.noFileSelectedTitle') }}</p>
        <p class="text-sm text-muted">{{ $t('empty.noFileSelectedDescription') }}</p>
      </div>
    </div>

    <template v-else>
      <div v-if="viewMode === 'list'" class="min-h-0 flex-1 overflow-auto"
           @mousedown.right.prevent.stop="openMenu($event, canvasMenu())"
           @contextmenu.prevent.stop="openMenu($event, canvasMenu())">
        <table class="w-full table-fixed border-collapse text-sm">
          <colgroup>
            <col class="w-[44px]"/>
            <col class="w-[31%]"/>
            <col class="w-[22%]"/>
            <col class="w-[22%]"/>
            <col class="w-[11%]"/>
            <col class="w-[14%]"/>
          </colgroup>
          <thead class="sticky top-0 z-10 bg-default/95 backdrop-blur">
          <tr class="border-b border-default text-left text-xs uppercase tracking-wide text-muted">
            <th class="px-3 py-3 font-medium">
              <UCheckbox :model-value="allVisibleSelected" @update:model-value="toggleSelectAll"/>
            </th>
            <th class="px-4 py-3 font-medium">{{ t('labels.key') }}</th>
            <th class="px-4 py-3 font-medium">{{ t('labels.sourceValue') }}</th>
            <th class="px-4 py-3 font-medium">{{ t('labels.targetValue') }}</th>
            <th class="px-4 py-3 font-medium">{{ t('labels.status') }}</th>
            <th class="px-4 py-3 font-medium">{{ t('labels.updated') }}</th>
          </tr>
          </thead>
          <tbody>
          <tr v-if="!entries.length">
            <td class="px-4 py-8 text-sm text-muted" colspan="6">{{ $t('empty.noEntriesForFile') }}</td>
          </tr>
          <tr
              v-for="entry in entries"
              :key="entry.id"
              :class="[
                'cursor-default border-b border-default/60 align-top transition-colors hover:bg-elevated/60',
                selectedEntryIds.includes(entry.id) ? 'bg-elevated/80' : ''
              ]"
              @click="handleRowPrimaryAction(entry, $event)"
              @mousedown.right.prevent.stop="onRowContextMenu(entry, $event)"
              @contextmenu.prevent.stop="onRowContextMenu(entry, $event)"
          >
            <td class="px-3 py-3 align-top">
              <UCheckbox :model-value="selectedEntryIds.includes(entry.id)" @click.stop="toggleEntrySelection(entry)"/>
            </td>
            <td class="px-4 py-3">
              <div class="min-w-0 space-y-1">
                <div class="truncate font-medium">{{ entry.key }}</div>
                <div class="flex flex-wrap gap-1.5">
                  <UBadge v-if="entry.noteCount" color="neutral" size="sm" variant="subtle">{{
                      entry.noteCount
                    }}N
                  </UBadge>
                  <UBadge v-if="entry.candidateCount" color="primary" size="sm" variant="subtle">{{
                      entry.candidateCount
                    }}C
                  </UBadge>
                </div>
              </div>
            </td>
            <td class="px-4 py-3">
              <div class="line-clamp-3 whitespace-pre-wrap text-muted">{{ entry.sourceValue || '—' }}</div>
            </td>
            <td class="px-4 py-3">
              <div class="line-clamp-3 whitespace-pre-wrap">{{ entry.targetValue || '—' }}</div>
            </td>
            <td class="px-4 py-3">
              <UBadge :color="statusColor(entry.status)" variant="subtle">{{ t(`status.${entry.status}`) }}</UBadge>
            </td>
            <td class="px-4 py-3 text-muted">{{ new Date(entry.updatedAt).toLocaleString() }}</td>
          </tr>
          </tbody>
        </table>
      </div>

      <div v-else class="h-full min-h-0 overflow-hidden">
        <TreemapView :entries="entries" :search-text="searchText" :show-only-missing="showOnlyMissing" :sort-by="sortBy"
                     :status-filter="statusFilter"/>
      </div>
    </template>
  </UCard>

  <ContextMenu v-model:open="contextMenuOpen" :items="contextMenuItems" :x="contextX" :y="contextY"
               @select="onMenuSelect"/>
</template>
