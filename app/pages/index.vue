<script lang="ts" setup>
import type {Ref} from 'vue'
import {storeToRefs} from 'pinia'
import type {
  BatchExportInput,
  CreateEntryInput,
  CreateResourceFileInput,
  DeleteResourceFileInput,
  EntrySummary,
  ImportFileInput,
  ImportPreviewResponse,
  RenameResourceFileInput,
  TranslationStatus,
  UpdateProjectMetadataInput
} from '#shared/types/models'
import {HOME_TAB_ID} from '~/stores/workspace'
import type {EntrySortBy} from '~/stores/editor'

const {t} = useI18n()
const toast = useToast()
const dialogs = useSystemDialogs()
const workspace = useWorkspaceStore()
const editor = useEditorStore()

const {
  activeProject,
  projects,
  activeProjectId,
  activeTabId,
  isBusy,
  isCommandPaletteOpen,
  lastSavedAt,
  recentProjects
} = storeToRefs(workspace)
const {
  searchText,
  statusFilter,
  sortBy,
  selectedFileId,
  selectedEntryId,
  selectedEntryIds,
  currentView,
  showOnlyMissing,
  bulkSearch,
  bulkReplacement,
  bulkUseRegex
} = storeToRefs(editor)

const createModalOpen = ref(false)
const createEntryModalOpen = ref(false)
const createFileModalOpen = ref(false)
const renameFileModalOpen = ref(false)
const deleteFileModalOpen = ref(false)
const batchExportModalOpen = ref(false)
const pendingCreateEntryFileId = ref<string | null>(null)
const pendingCreateFileLocale = ref<string | null>(null)
const pendingFileActionId = ref<string | null>(null)
const findModalOpen = ref(false)
const replaceModalOpen = ref(false)
const bulkModalOpen = ref(false)
const importModalOpen = ref(false)
const exportModalOpen = ref(false)
const pendingExportFileId = ref<string | null>(null)
const preferencesModalOpen = ref(false)
const projectMetadataOpen = ref(false)
const aboutModalOpen = ref(false)
const licenseModalOpen = ref(false)
const importPreview = ref<ImportPreviewResponse | null>(null)

const findDraft = reactive({text: '', missingOnly: false, status: 'all' as TranslationStatus | 'all'})
const replaceDraft = reactive({search: '', replacement: '', useRegex: false})

useHead({title: 'glossa-morpho'})

onMounted(async () => {
  await workspace.bootstrap()
  window.addEventListener('gm:toggle-command-palette', onToggleCommandPalette)
})
onBeforeUnmount(() => window.removeEventListener('gm:toggle-command-palette', onToggleCommandPalette))
watch(activeProjectId, (next, prev) => {
  if (next !== prev) editor.resetForProject()
})

const filteredEntries = computed<EntrySummary[]>(() => {
  const project = activeProject.value
  if (!project || !selectedFileId.value) return []
  let entries = project.entries.filter((entry) => {
    const haystack = `${entry.key} ${entry.sourceValue} ${entry.targetValue}`.toLowerCase()
    return (entry.fileId === selectedFileId.value)
        && (!searchText.value.trim() || haystack.includes(searchText.value.toLowerCase()))
        && (!showOnlyMissing.value || !entry.targetValue)
        && (statusFilter.value === 'all' || entry.status === statusFilter.value)
  })
  if (sortBy.value === 'keyAsc') entries = [...entries].sort((a, b) => a.key.localeCompare(b.key))
  else if (sortBy.value === 'status') entries = [...entries].sort((a, b) => a.status.localeCompare(b.status) || b.updatedAt.localeCompare(a.updatedAt))
  else entries = [...entries].sort((a, b) => b.updatedAt.localeCompare(a.updatedAt))
  return entries
})

watch([activeProject, filteredEntries, selectedFileId], () => {
  editor.syncSelectedEntries(filteredEntries.value.map((entry) => entry.id))
  if (selectedFileId.value && activeProject.value && !activeProject.value.files.some((file) => file.id === selectedFileId.value)) {
    editor.selectFile(null)
    return
  }
  if (!selectedFileId.value) editor.selectEntry(null)
})

const selectedEntry = computed(() => {
  if (!activeProject.value || !selectedFileId.value || !selectedEntryId.value) return undefined
  if (!filteredEntries.value.some((entry) => entry.id === selectedEntryId.value)) return undefined
  return activeProject.value.details[selectedEntryId.value]
})
const currentFile = computed(() => activeProject.value?.files.find((file) => file.id === pendingFileActionId.value || file.id === selectedFileId.value) || null)
const statusOptions = computed(() => [
  {label: t('labels.allStatuses'), value: 'all'},
  {label: t('status.new'), value: 'new'},
  {label: t('status.translated'), value: 'translated'},
  {label: t('status.reviewed'), value: 'reviewed'},
  {label: t('status.approved'), value: 'approved'},
  {label: t('status.stale'), value: 'stale'}
])
const activeTabLabel = computed(() => activeTabId.value === HOME_TAB_ID ? t('home.title') : activeProject.value?.name ?? t('empty.noProjects'))
const selectedFileName = computed(() => {
  const project = activeProject.value
  if (!project || !selectedFileId.value) return t('labels.allFiles')
  return project.files.find((file) => file.id === selectedFileId.value)?.name || t('labels.allFiles')
})

async function onCreateProject(payload: Parameters<typeof workspace.createProject>[0]) {
  await workspace.createProject(payload)
  createModalOpen.value = false
  toast.add({title: t('feedback.projectCreated'), color: 'success'})
}

async function onOpenProject(path?: string) {
  const resolvedPath = path || await dialogs.pickProjectToOpen()
  if (!resolvedPath) return
  await workspace.openProject(resolvedPath)
  toast.add({title: t('feedback.projectOpened'), color: 'success'})
}

async function onSaveProject() {
  if (!activeProject.value) return
  await workspace.saveActiveProject()
  toast.add({title: t('feedback.projectSaved'), color: 'success'})
}

async function onSaveAsProject() {
  if (!activeProject.value) return
  const path = await dialogs.pickProjectSavePath({
    defaultPath: activeProject.value.path,
    projectName: activeProject.value.name
  })
  if (!path) return
  await workspace.saveActiveProjectAs(path)
  toast.add({title: t('feedback.projectSaved'), color: 'success'})
}

async function onSaveProjectMetadata(payload: UpdateProjectMetadataInput) {
  await workspace.updateProjectMetadata(payload)
  projectMetadataOpen.value = false
  toast.add({title: t('feedback.projectSaved'), color: 'success'})
}

function openNewEntry(fileId?: string) {
  pendingCreateEntryFileId.value = fileId || selectedFileId.value || activeProject.value?.files[0]?.id || null
  createEntryModalOpen.value = true
}

function openNewFile(localeCode?: string | null) {
  const selectedLocale = selectedFileId.value
      ? activeProject.value?.files.find((file) => file.id === selectedFileId.value)?.locale || null
      : null
  pendingCreateFileLocale.value = localeCode || selectedLocale || activeProject.value?.workingLocale || null
  createFileModalOpen.value = true
}

function openRenameFile(fileId?: string | null) {
  pendingFileActionId.value = fileId || selectedFileId.value || null
  renameFileModalOpen.value = Boolean(pendingFileActionId.value)
}

function openDeleteFile(fileId?: string | null) {
  pendingFileActionId.value = fileId || selectedFileId.value || null
  deleteFileModalOpen.value = Boolean(pendingFileActionId.value)
}

async function onCreateFile(payload: CreateResourceFileInput) {
  const project = await workspace.createResourceFile(payload)
  createFileModalOpen.value = false
  pendingCreateFileLocale.value = null
  const createdFile = project.files.find((file) => file.locale === payload.locale && file.logicalPath === payload.logicalPath)
  if (createdFile) editor.selectFile(createdFile.id)
  toast.add({title: t('feedback.fileCreated'), color: 'success'})
}

async function onRenameFile(payload: RenameResourceFileInput) {
  const project = await workspace.renameResourceFile(payload)
  renameFileModalOpen.value = false
  const file = project.files.find((item) => item.id === payload.fileId) || project.files[0]
  if (file) editor.selectFile(file.id)
  toast.add({title: t('feedback.fileRenamed'), color: 'success'})
}

async function onDeleteFile(payload: DeleteResourceFileInput) {
  await workspace.deleteResourceFile(payload)
  deleteFileModalOpen.value = false
  pendingFileActionId.value = null
  editor.selectFile(null)
  editor.syncSelectedEntries(filteredEntries.value.map((entry) => entry.id))
  toast.add({title: t('feedback.fileDeleted'), color: 'success'})
}

async function onCreateEntry(payload: CreateEntryInput) {
  const project = await workspace.createEntry(payload)
  createEntryModalOpen.value = false
  pendingCreateEntryFileId.value = null
  const created = project.entries.find((entry) => entry.fileId === payload.fileId && entry.key === payload.key)
  if (created) {
    editor.selectFile(payload.fileId)
    editor.selectEntry(created.id)
  }
  toast.add({title: t('feedback.entryCreated'), color: 'success'})
}

async function onDeleteEntry(entryId: string) {
  if (!activeProject.value) return
  await workspace.deleteEntry({projectId: activeProject.value.id, entryId})
  editor.syncSelectedEntries(filteredEntries.value.map((entry) => entry.id))
  toast.add({title: t('feedback.entryDeleted'), color: 'success'})
}

async function onDeleteEntries(entryIds: string[]) {
  if (!activeProject.value || !entryIds.length) return
  await workspace.deleteEntries({projectId: activeProject.value.id, entryIds})
  editor.syncSelectedEntries(filteredEntries.value.map((entry) => entry.id))
  toast.add({
    title: t('feedback.entryDeleted'),
    description: `${entryIds.length} ${t('labels.entries')}`,
    color: 'success'
  })
}

async function onSaveEntry(payload: { targetValue: string; note: string; status: TranslationStatus }) {
  if (!activeProject.value || !selectedEntry.value) return
  await workspace.updateEntry({projectId: activeProject.value.id, entryId: selectedEntry.value.id, ...payload})
  toast.add({title: t('feedback.entrySaved'), color: 'success'})
}

function updateStatusFilter(value: TranslationStatus | 'all') {
  editor.setStatusFilter(value)
}

function updateSortBy(value: EntrySortBy) {
  editor.setSortBy(value)
}

function changeView(value: 'list' | 'treemap') {
  editor.currentView = value
}

function changeShowOnlyMissing(value: boolean) {
  editor.showOnlyMissing = value
}

function openFindDialog() {
  findDraft.text = searchText.value
  findDraft.missingOnly = showOnlyMissing.value
  findDraft.status = statusFilter.value
  findModalOpen.value = true
}

function applyFindFilters() {
  editor.searchText = findDraft.text
  editor.showOnlyMissing = findDraft.missingOnly
  editor.setStatusFilter(findDraft.status)
  findModalOpen.value = false
}

function openReplaceDialog() {
  replaceDraft.search = bulkSearch.value
  replaceDraft.replacement = bulkReplacement.value
  replaceDraft.useRegex = bulkUseRegex.value
  replaceModalOpen.value = true
}

function openBulkDialog() {
  replaceDraft.search = bulkSearch.value
  replaceDraft.replacement = bulkReplacement.value
  replaceDraft.useRegex = bulkUseRegex.value
  bulkModalOpen.value = true
}

async function applyReplace(closeRef: Ref<boolean>, successTitle: string) {
  if (!activeProject.value || !replaceDraft.search) return
  bulkSearch.value = replaceDraft.search
  bulkReplacement.value = replaceDraft.replacement
  bulkUseRegex.value = replaceDraft.useRegex
  const result = await workspace.bulkReplace({
    projectId: activeProject.value.id,
    search: replaceDraft.search,
    replacement: replaceDraft.replacement,
    useRegex: replaceDraft.useRegex,
    targetScope: 'targetOnly'
  })
  if (result.changedEntryIds.length > 0 && !selectedEntryId.value) editor.selectEntry(result.changedEntryIds[0])
  closeRef.value = false
  toast.add({
    title: successTitle,
    description: `${result.changedEntryIds.length} ${t('labels.entries')}`,
    color: 'success'
  })
}

const onApplyReplace = async () => applyReplace(replaceModalOpen, t('feedback.replaceDone'))
const onApplyBulkReplace = async () => applyReplace(bulkModalOpen, t('feedback.bulkReplaceDone'))

function onToggleCommandPalette() {
  workspace.toggleCommandPalette()
}

function closeActiveProject() {
  if (activeProjectId.value) workspace.closeProject(activeProjectId.value)
}

function openImportDialog() {
  if (activeProject.value) {
    importPreview.value = null
    importModalOpen.value = true
  }
}

async function onPreviewImport(files: ImportFileInput[]) {
  if (!activeProjectId.value || files.length === 0) return
  importPreview.value = await workspace.previewImport(activeProjectId.value, files)
}

async function onCommitImport(previewId: string) {
  if (!activeProjectId.value) return
  const project = await workspace.commitImport({projectId: activeProjectId.value, previewId})
  importModalOpen.value = false
  importPreview.value = null
  if (project.entries.length > 0 && !selectedEntryId.value) editor.selectEntry(project.entries[0]?.id ?? null)
  toast.add({title: t('feedback.importCommitted'), color: 'success'})
}

function openExportDialog(fileId?: string | null) {
  if (!activeProject.value) return
  pendingExportFileId.value = fileId ?? selectedFileId.value ?? activeProject.value.files[0]?.id ?? null
  exportModalOpen.value = true
}

async function onExportProject(payload: { fileId: string; outputPath?: string }) {
  if (!activeProjectId.value) return
  const result = await workspace.exportProject({
    projectId: activeProjectId.value,
    fileId: payload.fileId,
    outputPath: payload.outputPath
  })
  exportModalOpen.value = false
  toast.add({title: t('feedback.exportComplete'), description: result.outputPath, color: 'success'})
}

async function onBatchExport(payload: BatchExportInput) {
  if (!activeProjectId.value) return
  const result = await workspace.exportProjectBatch({
    projectId: activeProjectId.value,
    fileIds: payload.fileIds,
    outputDirectory: payload.outputDirectory
  })
  batchExportModalOpen.value = false
  toast.add({
    title: t('feedback.exportComplete'),
    description: `${result.exportedFiles.length} ${t('labels.fileCount')}`,
    color: 'success'
  })
}

function handleEntrySelection(payload: { entryId: string; append?: boolean; range?: boolean; orderedIds?: string[] }) {
  editor.selectEntry(payload.entryId, payload)
}

function notifyNotWired(label: string) {
  toast.add({title: label, description: t('feedback.notWiredYet'), color: 'info'})
}

function notifySelectionAction(label: string) {
  toast.add({title: label, description: t('feedback.useTextSelection'), color: 'info'})
}
</script>

<template>
  <div class="flex h-screen flex-col overflow-hidden bg-default text-default">
    <AppHeader
        :has-active-project="Boolean(activeProject)"
        @copy="notifySelectionAction(t('menu.copy'))"
        @cut="notifySelectionAction(t('menu.cut'))"
        @paste="notifySelectionAction(t('menu.paste'))"
        @redo="notifyNotWired(t('menu.redo'))"
        @undo="notifyNotWired(t('menu.undo'))"
        @close-project="closeActiveProject"
        @create-from="notifyNotWired(t('menu.createFrom'))"
        @create-project="createModalOpen = true"
        @delete-selection="selectedEntryIds.length > 1 ? onDeleteEntries(selectedEntryIds) : (selectedEntryId ? onDeleteEntry(selectedEntryId) : notifySelectionAction(t('menu.delete')))"
        @export-batch-files="batchExportModalOpen = true"
        @export-files="openExportDialog"
        @import-files="openImportDialog"
        @open-about="aboutModalOpen = true"
        @open-bulk-edit="openBulkDialog"
        @open-command-palette="onToggleCommandPalette"
        @open-documentation="notifyNotWired(t('menu.documentation'))"
        @open-feedback="notifyNotWired(t('menu.feedback'))"
        @open-find="openFindDialog"
        @open-license="licenseModalOpen = true"
        @open-preferences="preferencesModalOpen = true"
        @open-project="onOpenProject()"
        @open-project-metadata="projectMetadataOpen = true"
        @open-replace="openReplaceDialog"
        @report-issue="notifyNotWired(t('menu.reportIssue'))"
        @save-as-project="onSaveAsProject"
        @save-project="onSaveProject"
        @select-all="editor.selectAllEntries(filteredEntries.map((entry) => entry.id))"
    />

    <ProjectTabs :active-tab-id="activeTabId" :home-tab-id="HOME_TAB_ID" :tabs="projects"
                 @close="workspace.closeProject" @reorder="workspace.reorderProjectTabs"
                 @select="workspace.setActiveTab"/>

    <main class="min-h-0 flex-1 overflow-hidden">
      <WorkspaceHome v-if="activeTabId === HOME_TAB_ID" :recent-projects="recentProjects" class="h-full"
                     @create-project="createModalOpen = true"
                     @open-project="(path) => path ? onOpenProject(path) : onOpenProject()"/>
      <EditorWorkspace
          v-else-if="activeProject"
          :active-project="activeProject"
          :current-view="currentView"
          :filtered-entries="filteredEntries"
          :search-text="searchText"
          :selected-entry="selectedEntry"
          :selected-entry-id="selectedEntryId"
          :selected-entry-ids="selectedEntryIds"
          :selected-file-id="selectedFileId"
          :show-only-missing="showOnlyMissing"
          :sort-by="sortBy"
          :status-filter="statusFilter"
          class="h-full"
          @change-show-only-missing="changeShowOnlyMissing"
          @change-view="changeView"
          @save-entry="onSaveEntry"
          @select-entry="handleEntrySelection"
          @select-all-entries="editor.selectAllEntries"
          @select-file="editor.selectFile"
          @update-sort-by="updateSortBy"
          @update-status-filter="updateStatusFilter"
          @create-entry="openNewEntry"
          @create-file="openNewFile"
          @rename-file="openRenameFile"
          @delete-file="openDeleteFile"
          @delete-entry="onDeleteEntry"
          @delete-entries="onDeleteEntries"
          @import-files="openImportDialog"
          @export-files="openExportDialog"
          @batch-export="batchExportModalOpen = true"
      />
      <div v-else class="flex h-full items-center justify-center text-sm text-muted">{{ $t('empty.noProjects') }}</div>
    </main>

    <WorkspaceStatusBar :active-project="activeProject" :active-tab-label="activeTabLabel"
                        :filtered-entries="filteredEntries" :is-busy="isBusy" :last-saved-at="lastSavedAt"
                        :selected-file-name="selectedFileName"/>

    <CreateProjectModal v-model:open="createModalOpen" @create="onCreateProject"/>
    <CreateEntryModal v-model:open="createEntryModalOpen" :file-id="pendingCreateEntryFileId" :project="activeProject"
                      @create="onCreateEntry"/>
    <CreateFileModal v-model:open="createFileModalOpen" :initial-locale="pendingCreateFileLocale"
                     :project="activeProject" @create="onCreateFile"/>
    <RenameFileModal v-model:open="renameFileModalOpen" :file="currentFile" :project="activeProject"
                     @save="onRenameFile"/>
    <DeleteFileModal v-model:open="deleteFileModalOpen" :file="currentFile" :project="activeProject"
                     @confirm="onDeleteFile"/>
    <BatchExportModal v-model:open="batchExportModalOpen" :project="activeProject" @export="onBatchExport"/>
    <PreferencesModal v-model:open="preferencesModalOpen"/>
    <ProjectMetadataModal v-model:open="projectMetadataOpen" :project="activeProject" @save="onSaveProjectMetadata"/>
    <ImportResourcesModal v-model:open="importModalOpen" :preview="importPreview" :project="activeProject"
                          @commit="onCommitImport" @preview="onPreviewImport"/>
    <ExportProjectModal v-model:open="exportModalOpen" :project="activeProject" :selected-file-id="pendingExportFileId"
                        @export="onExportProject"/>

    <UModal v-model:open="findModalOpen" :description="$t('descriptions.findReplace')" :title="$t('actions.findFilter')"
            :ui="{ content: 'sm:max-w-2xl' }">
      <template #body>
        <div class="space-y-4">
          <UFormField :label="$t('labels.find') as string">
            <UInput v-model="findDraft.text" icon="i-lucide-search"/>
          </UFormField>
          <UFormField :label="$t('labels.status') as string">
            <USelectMenu v-model="findDraft.status" :items="statusOptions" value-key="value"/>
          </UFormField>
          <UCheckbox v-model="findDraft.missingOnly" :label="$t('labels.missingOnly') as string"/>
        </div>
      </template>
      <template #footer>
        <div class="flex justify-end gap-2">
          <UButton color="neutral" variant="soft" @click="findModalOpen = false">{{ $t('actions.cancel') }}</UButton>
          <UButton icon="i-lucide-search" @click="applyFindFilters">{{ $t('actions.apply') }}</UButton>
        </div>
      </template>
    </UModal>

    <UModal v-model:open="replaceModalOpen" :description="$t('descriptions.replace')" :title="$t('actions.replace')"
            :ui="{ content: 'sm:max-w-2xl' }">
      <template #body>
        <div class="space-y-4">
          <UFormField :label="$t('labels.find') as string">
            <UInput v-model="replaceDraft.search" icon="i-lucide-search"/>
          </UFormField>
          <UFormField :label="$t('labels.replacement') as string">
            <UInput v-model="replaceDraft.replacement" icon="i-lucide-replace"/>
          </UFormField>
          <UCheckbox v-model="replaceDraft.useRegex" :label="$t('labels.regex') as string"/>
        </div>
      </template>
      <template #footer>
        <div class="flex justify-end gap-2">
          <UButton color="neutral" variant="soft" @click="replaceModalOpen = false">{{ $t('actions.cancel') }}</UButton>
          <UButton icon="i-lucide-replace" @click="onApplyReplace">{{ $t('actions.apply') }}</UButton>
        </div>
      </template>
    </UModal>

    <UModal v-model:open="bulkModalOpen" :description="$t('descriptions.bulkReplace')"
            :title="$t('actions.bulkReplace')" :ui="{ content: 'sm:max-w-3xl' }">
      <template #body>
        <div class="space-y-4">
          <UFormField :label="$t('labels.find') as string">
            <UInput v-model="replaceDraft.search" icon="i-lucide-search"/>
          </UFormField>
          <UFormField :label="$t('labels.replacement') as string">
            <UInput v-model="replaceDraft.replacement" icon="i-lucide-replace"/>
          </UFormField>
          <UCheckbox v-model="replaceDraft.useRegex" :label="$t('labels.regex') as string"/>
        </div>
      </template>
      <template #footer>
        <div class="flex justify-end gap-2">
          <UButton color="neutral" variant="soft" @click="bulkModalOpen = false">{{ $t('actions.cancel') }}</UButton>
          <UButton icon="i-lucide-wand-sparkles" @click="onApplyBulkReplace">{{ $t('actions.apply') }}</UButton>
        </div>
      </template>
    </UModal>

    <UModal v-model:open="aboutModalOpen" :title="$t('menu.about')" :ui="{ content: 'sm:max-w-xl' }">
      <template #body>
        <div class="space-y-3 text-sm"><p>{{ $t('app.name') }}</p>
          <p>{{ $t('app.subtitle') }}</p></div>
      </template>
    </UModal>
    <UModal v-model:open="licenseModalOpen" :title="$t('menu.license')" :ui="{ content: 'sm:max-w-xl' }">
      <template #body><p class="text-sm text-muted">{{ $t('help.licenseDescription') }}</p></template>
    </UModal>

    <CommandPalette :active-project-id="activeProjectId" :open="isCommandPaletteOpen" :projects="projects"
                    :recent-projects="recentProjects" @close="workspace.toggleCommandPalette(false)"
                    @create-project="createModalOpen = true" @go-home="workspace.openHome" @open-project="onOpenProject"
                    @save-project="onSaveProject" @select-project="workspace.setActiveTab"/>
  </div>
</template>
