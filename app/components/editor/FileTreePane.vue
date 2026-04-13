<script lang="ts" setup>
import type {DropdownMenuItem} from '@nuxt/ui'
import type {ProjectWorkspace} from '#shared/types/models'
import type {WorkspaceContextMenuItem} from '~/utils/workspace-actions'
import {buildContextMenuItems, parseActionValue} from '~/utils/workspace-actions'
import type {FileTreeNode} from '~/utils/file-tree'
import {buildFileTree} from '~/utils/file-tree'

const {t} = useI18n()

const props = defineProps<{
  project: ProjectWorkspace
  selectedFileId: string | null
}>()

const emit = defineEmits<{
  select: [fileId: string | null]
  createEntry: [fileId: string]
  createFile: [localeCode?: string | null]
  renameFile: [fileId: string]
  deleteFile: [fileId: string]
  importFiles: []
  exportFiles: [fileId: string | null]
  batchExport: []
}>()

const filter = ref('')
const contextMenuOpen = ref(false)
const contextMenuItems = ref<WorkspaceContextMenuItem[]>([])
const contextX = ref(0)
const contextY = ref(0)
const expandedLocaleIds = ref<string[]>([])

const selectedFile = computed(() => props.project.files.find((file) => file.id === props.selectedFileId) || null)

const headerMenuItems = computed<DropdownMenuItem[][]>(() => [[
  {
    label: t('actions.renameFile'),
    icon: 'i-lucide-file-pen-line',
    disabled: !selectedFile.value,
    onSelect: () => selectedFile.value && emit('renameFile', selectedFile.value.id)
  },
  {
    label: t('actions.deleteFile'),
    icon: 'i-lucide-trash-2',
    disabled: !selectedFile.value,
    onSelect: () => selectedFile.value && emit('deleteFile', selectedFile.value.id)
  },
  {
    label: t('actions.importResources'),
    icon: 'i-lucide-file-down',
    onSelect: () => emit('importFiles')
  },
  {
    label: t('actions.exportResources'),
    icon: 'i-lucide-file-up',
    disabled: !selectedFile.value,
    onSelect: () => emit('exportFiles', props.selectedFileId)
  },
  {
    label: t('actions.batchExport'),
    icon: 'i-lucide-files',
    disabled: !props.project.files.length,
    onSelect: () => emit('batchExport')
  }
]])

const treeItems = computed(() => {
  const items = buildFileTree(props.project, props.selectedFileId, filter.value)
  if (items[0]) items[0].label = t('labels.allFiles')
  return items
})

watch(treeItems, (items) => {
  const localeIds = items.flatMap((item) => item.kind === 'locale' ? [item.id] : [])
  if (!expandedLocaleIds.value.length) {
    expandedLocaleIds.value = localeIds
    return
  }
  expandedLocaleIds.value = Array.from(new Set([...expandedLocaleIds.value, ...localeIds]))
}, {immediate: true})

function openMenu(event: MouseEvent, items: WorkspaceContextMenuItem[]) {
  if (!items.length) return
  event.preventDefault()
  event.stopPropagation()
  contextMenuItems.value = items
  contextX.value = event.clientX
  contextY.value = event.clientY
  contextMenuOpen.value = true
}

function rootMenu(): WorkspaceContextMenuItem[] {
  return buildContextMenuItems(t, [
    {id: 'file.new'},
    {id: 'file.import'},
    {id: 'file.export', disabled: !selectedFile.value, payload: selectedFile.value?.id},
    {id: 'file.exportBatch', disabled: !props.project.files.length}
  ])
}

function localeMenu(localeCode: string): WorkspaceContextMenuItem[] {
  return buildContextMenuItems(t, [
    {id: 'file.new', payload: localeCode},
    {id: 'file.import'},
    {id: 'file.exportBatch', disabled: !props.project.files.length}
  ])
}

function fileMenu(fileId: string, localeCode: string): WorkspaceContextMenuItem[] {
  return buildContextMenuItems(t, [
    {id: 'file.new', payload: localeCode},
    {id: 'file.rename', payload: fileId},
    {id: 'file.delete', payload: fileId},
    {id: 'entry.new', payload: fileId},
    {id: 'file.import'},
    {id: 'file.export', payload: fileId},
    {id: 'file.exportBatch', disabled: !props.project.files.length}
  ])
}

function onMenuSelect(value: string) {
  const {actionId, payload} = parseActionValue(value)
  if (actionId === 'file.new') emit('createFile', payload)
  if (actionId === 'file.rename' && payload) emit('renameFile', payload)
  if (actionId === 'file.delete' && payload) emit('deleteFile', payload)
  if (actionId === 'entry.new' && payload) emit('createEntry', payload)
  if (actionId === 'file.import') emit('importFiles')
  if (actionId === 'file.export') emit('exportFiles', payload)
  if (actionId === 'file.exportBatch') emit('batchExport')
}

function handleNodeSelect(node: FileTreeNode) {
  if (node.kind === 'root') {
    emit('select', null)
    return
  }
  if (node.kind === 'file' && node.fileId) emit('select', node.fileId)
}

function handleNodeToggle(nodeId: string) {
  if (expandedLocaleIds.value.includes(nodeId)) {
    expandedLocaleIds.value = expandedLocaleIds.value.filter((item) => item !== nodeId)
    return
  }
  expandedLocaleIds.value = [...expandedLocaleIds.value, nodeId]
}

function handleNodeMenu(payload: { event: MouseEvent; node: FileTreeNode }) {
  const {event, node} = payload
  if (node.kind === 'root') return openMenu(event, rootMenu())
  if (node.kind === 'locale' && node.localeCode) return openMenu(event, localeMenu(node.localeCode))
  if (node.kind === 'file' && node.fileId && node.localeCode) return openMenu(event, fileMenu(node.fileId, node.localeCode))
}
</script>

<template>
  <UCard :ui="{ root: 'h-full min-h-0 rounded-none border-0 flex flex-col', body: 'min-h-0 flex-1 overflow-auto p-0' }">
    <template #header>
      <div class="space-y-3" @mousedown.right.prevent.stop="openMenu($event, rootMenu())"
           @contextmenu.prevent.stop="openMenu($event, rootMenu())">
        <div class="flex items-start justify-between gap-3">
          <div class="space-y-1">
            <h2 class="font-semibold">{{ $t('labels.files') }}</h2>
            <p class="text-sm text-muted">{{ project.files.length }} {{ $t('labels.fileCount') }} ·
              {{ $t('labels.localeChain') }}</p>
          </div>

          <div class="flex items-center gap-1">
            <UButton color="primary" icon="i-lucide-file-plus" size="sm" variant="soft"
                     @click="emit('createFile', null)">{{ $t('actions.newFile') }}
            </UButton>
            <UDropdownMenu :items="headerMenuItems" :modal="false" :ui="{ content: 'w-56' }">
              <UButton color="neutral" icon="i-lucide-ellipsis" variant="ghost"/>
            </UDropdownMenu>
          </div>
        </div>

        <UInput v-model="filter" :placeholder="$t('labels.searchFiles') as string" icon="i-lucide-search"/>
      </div>
    </template>

    <div class="h-full overflow-auto px-2 py-2" @mousedown.right.prevent.stop="openMenu($event, rootMenu())"
         @contextmenu.prevent.stop="openMenu($event, rootMenu())">
      <div class="space-y-0.5">
        <FileTreeBranch
            v-for="node in treeItems"
            :key="node.id"
            :expanded-ids="expandedLocaleIds"
            :level="0"
            :node="node"
            @select="handleNodeSelect"
            @toggle="handleNodeToggle"
            @create-file="emit('createFile', $event)"
            @open-menu="handleNodeMenu"
        />
      </div>
    </div>
  </UCard>

  <ContextMenu v-model:open="contextMenuOpen" :items="contextMenuItems" :x="contextX" :y="contextY"
               @select="onMenuSelect"/>
</template>
