<script setup lang="ts">
import type {CommandPaletteGroup} from '@nuxt/ui'

const props = defineProps<{
  open: boolean
}>()

const emit = defineEmits<{
  close: []
  action: [key: 'new' | 'open' | 'save' | 'bulk']
}>()

const groups = computed<CommandPaletteGroup[]>(() => [
  {
    id: 'workspace',
    label: 'Workspace',
    items: [
      {
        label: '新建项目 / New Project',
        icon: 'i-lucide-folder-plus',
        kbds: ['Ctrl', 'N'],
        suffix: 'Create project scaffold',
        onSelect: () => {
          emit('action', 'new')
          emit('close')
        }
      },
      {
        label: '打开项目 / Open Project',
        icon: 'i-lucide-folder-open',
        kbds: ['Ctrl', 'O'],
        suffix: 'Open demo or local project',
        onSelect: () => {
          emit('action', 'open')
          emit('close')
        }
      },
      {
        label: '保存当前项目 / Save Project',
        icon: 'i-lucide-save',
        kbds: ['Ctrl', 'S'],
        suffix: 'Persist current project state',
        onSelect: () => {
          emit('action', 'save')
          emit('close')
        }
      }
    ]
  },
  {
    id: 'editing',
    label: 'Editing',
    items: [
      {
        label: '批量替换 / Bulk Replace',
        icon: 'i-lucide-scan-search',
        kbds: ['Ctrl', 'Shift', 'R'],
        suffix: 'Apply replace to target values',
        onSelect: () => {
          emit('action', 'bulk')
          emit('close')
        }
      }
    ]
  }
])
</script>

<template>
  <UModal :open="open" title="Command Palette" description="Search and run workspace commands."
          @update:open="(value) => !value && emit('close')">
    <template #content>
      <UCommandPalette :groups="groups" class="h-96"/>
    </template>
  </UModal>
</template>
