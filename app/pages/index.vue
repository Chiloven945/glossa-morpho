<script setup lang="ts">
import {storeToRefs} from 'pinia'
import type { EntrySummary } from '#shared/types/models'
import { APP_LOCALES } from '#shared/constants/locales'

const {t} = useI18n()
const toast = useToast()
const workspace = useWorkspaceStore()
const editor = useEditorStore()

const {
  activeProject,
  projects,
  activeProjectId,
  isBusy,
  isCommandPaletteOpen,
  lastSavedAt,
  recentProjects
} = storeToRefs(workspace)
const {
  searchText,
  selectedFileId,
  selectedEntryId,
  currentView,
  showOnlyMissing,
  bulkSearch,
  bulkReplacement,
  bulkUseRegex
} = storeToRefs(editor)

const createModalOpen = ref(false)
const openModalOpen = ref(false)
const createProjectForm = reactive({
  name: 'Starter Project',
  sourceLocale: 'en-US',
  targetLocale: 'zh-CN'
})
const openPath = ref('/Users/you/projects/demo.gmproj')

const localeOptions = APP_LOCALES.map((item) => ({
  label: `${item.code} · ${item.label}`,
  value: item.code
}))

useHead({title: 'glossa-morpho'})

onMounted(async () => {
  await workspace.bootstrap()
  window.addEventListener('gm:toggle-command-palette', onToggleCommandPalette)
})

onBeforeUnmount(() => {
  window.removeEventListener('gm:toggle-command-palette', onToggleCommandPalette)
})

watch(activeProjectId, () => {
  editor.resetForProject()
})

const filteredEntries = computed<EntrySummary[]>(() => {
  const project = activeProject.value
  if (!project) return []

  return project.entries.filter((entry) => {
    const matchesFile = !selectedFileId.value || entry.fileId === selectedFileId.value
    const matchesSearch = !searchText.value.trim() || `${entry.key} ${entry.sourceValue} ${entry.targetValue}`.toLowerCase().includes(searchText.value.toLowerCase())
    const matchesMissing = !showOnlyMissing.value || !entry.targetValue
    return matchesFile && matchesSearch && matchesMissing
  })
})

const selectedEntry = computed(() => {
  const project = activeProject.value
  if (!project || !selectedEntryId.value) return undefined
  return project.details[selectedEntryId.value]
})

async function onCreateProject() {
  await workspace.createProject({
    name: createProjectForm.name,
    sourceLocale: createProjectForm.sourceLocale,
    targetLocale: createProjectForm.targetLocale
  })
  createModalOpen.value = false
  toast.add({title: t('feedback.projectCreated'), color: 'success'})
}

async function onOpenProject(path = openPath.value) {
  await workspace.openProject(path)
  openModalOpen.value = false
  toast.add({title: t('feedback.projectOpened'), color: 'success'})
}

async function onSaveProject() {
  await workspace.saveActiveProject()
  toast.add({title: t('feedback.projectSaved'), color: 'success'})
}

async function onSaveEntry(payload: {
  targetValue: string
  note: string
  status: 'new' | 'translated' | 'reviewed' | 'approved' | 'stale'
}) {
  if (!activeProject.value || !selectedEntry.value) return

  await workspace.updateEntry({
    projectId: activeProject.value.id,
    entryId: selectedEntry.value.id,
    ...payload
  })

  toast.add({title: t('feedback.entrySaved'), color: 'success'})
}

function updateSearch(value: string) {
  editor.searchText = value
}

function changeView(value: 'list' | 'treemap') {
  editor.currentView = value
}

function changeShowOnlyMissing(value: boolean) {
  editor.showOnlyMissing = value
}

function updateBulkSearch(value: string) {
  editor.bulkSearch = value
}

function updateBulkReplacement(value: string) {
  editor.bulkReplacement = value
}

function updateBulkUseRegex(value: boolean) {
  editor.bulkUseRegex = value
}

async function onApplyBulkReplace() {
  if (!activeProject.value || !bulkSearch.value) return

  const result = await workspace.bulkReplace({
    projectId: activeProject.value.id,
    search: bulkSearch.value,
    replacement: bulkReplacement.value,
    useRegex: bulkUseRegex.value,
    targetScope: 'targetOnly'
  })

  if (result.changedEntryIds.length > 0 && !selectedEntryId.value) {
    editor.selectEntry(result.changedEntryIds[0])
  }

  toast.add({
    title: t('feedback.bulkReplaceDone'),
    description: `${result.changedEntryIds.length} ${t('labels.entries')}`,
    color: 'success'
  })
}

function onToggleCommandPalette() {
  workspace.toggleCommandPalette()
}

function runCommandPaletteAction(key: 'new' | 'open' | 'save' | 'bulk') {
  if (key === 'new') {
    createModalOpen.value = true
    return
  }
  if (key === 'open') {
    openModalOpen.value = true
    return
  }
  if (key === 'save') return onSaveProject()
  if (key === 'bulk') return onApplyBulkReplace()
}
</script>

<template>
  <div class="min-h-screen bg-default">
    <UContainer class="py-6 space-y-4">
      <AppHeader
          :project-name="activeProject?.name"
          :is-busy="isBusy"
          @create-project="createModalOpen = true"
          @open-project="openModalOpen = true"
          @save-project="onSaveProject"
          @open-commands="onToggleCommandPalette"
      />

      <ProjectTabs :tabs="projects" :active-project-id="activeProjectId" @select="workspace.setActiveProject"/>

      <template v-if="activeProject">
        <div class="grid gap-4 xl:grid-cols-[18rem_minmax(0,1fr)_24rem]">
          <FileTreePane
              :project="activeProject"
              :selected-file-id="selectedFileId"
              @select="editor.selectFile"
          />

          <StringListPane
              :project="activeProject"
              :entries="filteredEntries"
              :selected-entry-id="selectedEntryId"
              :search-text="searchText"
              :view-mode="currentView"
              :show-only-missing="showOnlyMissing"
              :bulk-search="bulkSearch"
              :bulk-replacement="bulkReplacement"
              :bulk-use-regex="bulkUseRegex"
              @update-search="updateSearch"
              @select-entry="editor.selectEntry"
              @change-view="changeView"
              @change-show-only-missing="changeShowOnlyMissing"
              @update-bulk-search="updateBulkSearch"
              @update-bulk-replacement="updateBulkReplacement"
              @update-bulk-use-regex="updateBulkUseRegex"
              @apply-bulk-replace="onApplyBulkReplace"
          />

          <EntryInspector :entry="selectedEntry" @save="onSaveEntry"/>
        </div>

        <UCard>
          <div class="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
            <div class="flex flex-wrap gap-2">
              <UBadge color="neutral" variant="subtle">{{ activeProject.stats.total }} {{ $t('labels.total') }}</UBadge>
              <UBadge color="primary" variant="soft">{{ activeProject.stats.translated }} {{
                  $t('labels.translated')
                }}
              </UBadge>
              <UBadge color="warning" variant="soft">{{ activeProject.stats.missing }} {{
                  $t('labels.missing')
                }}
              </UBadge>
              <UBadge color="info" variant="soft">{{ activeProject.stats.reviewed }} {{
                  $t('labels.reviewed')
                }}
              </UBadge>
            </div>
            <p class="text-sm text-muted">
              {{
                lastSavedAt ? `${t('labels.lastSaved')}: ${new Date(lastSavedAt).toLocaleString()}` : t('labels.notSavedYet')
              }}
            </p>
          </div>
        </UCard>
      </template>

      <UCard v-else>
        <div class="flex min-h-[24rem] items-center justify-center">
          <div class="max-w-xl space-y-5 text-center">
            <div class="space-y-2">
              <h2 class="text-2xl font-semibold">{{ t('empty.title') }}</h2>
              <p class="text-sm text-muted">{{ t('empty.description') }}</p>
            </div>

            <div class="flex flex-wrap justify-center gap-3">
              <UButton icon="i-lucide-folder-plus" @click="createModalOpen = true">
                {{ t('actions.newProject') }}
              </UButton>
              <UButton color="neutral" variant="outline" icon="i-lucide-folder-open" @click="openModalOpen = true">
                {{ t('actions.openProject') }}
              </UButton>
            </div>

            <div v-if="recentProjects.length" class="space-y-3 text-left">
              <p class="text-sm font-medium">{{ t('labels.recentProjects') }}</p>
              <div class="grid gap-2">
                <UButton
                    v-for="path in recentProjects"
                    :key="path"
                    color="neutral"
                    variant="soft"
                    class="justify-start"
                    @click="onOpenProject(path)"
                >
                  {{ path }}
                </UButton>
              </div>
            </div>
          </div>
        </div>
      </UCard>
    </UContainer>

    <UModal v-model:open="createModalOpen" :title="t('actions.newProject')"
            :description="t('descriptions.createProject')" :ui="{ footer: 'justify-end' }">
      <template #content>
        <div class="p-4">
          <UForm :state="createProjectForm" class="space-y-4">
            <UFormField :label="t('labels.projectName')" name="name">
              <UInput v-model="createProjectForm.name" class="w-full"/>
            </UFormField>
            <div class="grid gap-4 md:grid-cols-2">
              <UFormField :label="t('labels.sourceLocale')" name="sourceLocale">
                <USelectMenu v-model="createProjectForm.sourceLocale" :items="localeOptions" value-key="value"
                             class="w-full"/>
              </UFormField>
              <UFormField :label="t('labels.targetLocale')" name="targetLocale">
                <USelectMenu v-model="createProjectForm.targetLocale" :items="localeOptions" value-key="value"
                             class="w-full"/>
              </UFormField>
            </div>
          </UForm>
        </div>
      </template>
      <template #footer>
        <UButton color="neutral" variant="outline" @click="createModalOpen = false">
          {{ t('actions.cancel') }}
        </UButton>
        <UButton icon="i-lucide-folder-plus" @click="onCreateProject">
          {{ t('actions.create') }}
        </UButton>
      </template>
    </UModal>

    <UModal v-model:open="openModalOpen" :title="t('actions.openProject')" :description="t('descriptions.openProject')"
            :ui="{ footer: 'justify-end' }">
      <template #content>
        <div class="p-4 space-y-4">
          <UFormField :label="t('labels.path')" name="path">
            <UInput v-model="openPath" class="w-full"/>
          </UFormField>

          <div v-if="recentProjects.length" class="space-y-2">
            <p class="text-sm font-medium">{{ t('labels.recentProjects') }}</p>
            <div class="grid gap-2">
              <UButton
                  v-for="path in recentProjects"
                  :key="path"
                  color="neutral"
                  variant="soft"
                  class="justify-start"
                  @click="openPath = path"
              >
                {{ path }}
              </UButton>
            </div>
          </div>
        </div>
      </template>
      <template #footer>
        <UButton color="neutral" variant="outline" @click="openModalOpen = false">
          {{ t('actions.cancel') }}
        </UButton>
        <UButton icon="i-lucide-folder-open" @click="onOpenProject()">
          {{ t('actions.openProject') }}
        </UButton>
      </template>
    </UModal>

    <CommandPalette
        :open="isCommandPaletteOpen"
        @close="workspace.toggleCommandPalette(false)"
        @action="runCommandPaletteAction"
    />
  </div>
</template>
