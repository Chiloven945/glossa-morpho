<script lang="ts" setup>
import type {CommandPaletteGroup} from '@nuxt/ui'
import type {ProjectWorkspace} from '#shared/types/models'

const {t} = useI18n()

const props = defineProps<{
  open: boolean
  projects: ProjectWorkspace[]
  recentProjects: string[]
  activeProjectId: string | null
}>()

const emit = defineEmits<{
  close: []
  create: []
  openProject: [path?: string]
  save: []
  importFiles: []
  exportFiles: []
  openHome: []
  selectProject: [projectId: string]
}>()

const groups = computed<CommandPaletteGroup[]>(() => [
  {
    id: 'global',
    label: t('labels.globalActions'),
    items: [
      {
        label: t('actions.goHome'),
        icon: 'i-lucide-house',
        onSelect() {
          emit('openHome')
          emit('close')
        }
      },
      {
        label: t('actions.newProject'),
        icon: 'i-lucide-folder-plus',
        onSelect() {
          emit('create')
          emit('close')
        }
      },
      {
        label: t('actions.openProject'),
        icon: 'i-lucide-folder-open',
        onSelect() {
          emit('openProject')
          emit('close')
        }
      },
      {
        label: t('actions.save'),
        icon: 'i-lucide-save',
        disabled: !props.activeProjectId,
        onSelect() {
          emit('save')
          emit('close')
        }
      },
      {
        label: t('actions.import'),
        icon: 'i-lucide-download',
        disabled: !props.activeProjectId,
        onSelect() {
          emit('importFiles')
          emit('close')
        }
      },
      {
        label: t('actions.export'),
        icon: 'i-lucide-upload',
        disabled: !props.activeProjectId,
        onSelect() {
          emit('exportFiles')
          emit('close')
        }
      }
    ]
  },
  {
    id: 'opened',
    label: t('labels.openedProjects'),
    items: props.projects.map(project => ({
      label: project.name,
      suffix: project.path,
      icon: 'i-lucide-folder-kanban',
      onSelect() {
        emit('selectProject', project.id)
        emit('close')
      }
    }))
  },
  {
    id: 'recent',
    label: t('labels.recentProjects'),
    items: props.recentProjects.map(path => ({
      label: path.split('/').pop()?.replace(/\.gmproj$/i, '') || path,
      suffix: path,
      icon: 'i-lucide-history',
      onSelect() {
        emit('openProject', path)
        emit('close')
      }
    }))
  }
])
</script>

<template>
  <UModal :open="open" :title="t('actions.commandPalette')" @update:open="value => !value && emit('close')">
    <template #content>
      <div class="p-4">
        <UCommandPalette
            :groups="groups"
            :placeholder="t('labels.commandPlaceholder')"
            class="h-96"
        />
      </div>
    </template>
  </UModal>
</template>

