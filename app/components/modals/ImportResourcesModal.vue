<script lang="ts" setup>
import type {ImportFileInput, ImportPreviewResponse, ProjectWorkspace} from '#shared/types/models'
import {APP_LOCALES} from '#shared/constants/locales'
import type {TableColumn} from '@nuxt/ui'

const {t} = useI18n()
const dialogs = useSystemDialogs()

const props = defineProps<{
  open: boolean
  project?: ProjectWorkspace
  preview: ImportPreviewResponse | null
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  preview: [files: ImportFileInput[]]
  commit: [previewId: string]
}>()

const rows = ref<ImportFileInput[]>([])

const localeOptions = computed(() => APP_LOCALES.map((item) => ({
  label: `${item.label} · ${item.code}`,
  value: item.code
})))
const fileColumns: TableColumn<ImportPreviewResponse['items'][number]>[] = [
  {accessorKey: 'name', header: 'File'},
  {accessorKey: 'locale', header: 'Locale'},
  {accessorKey: 'format', header: 'Format'},
  {accessorKey: 'entryCount', header: 'Entries'},
  {accessorKey: 'conflictCount', header: 'Conflicts'}
]
const entryColumns: TableColumn<ImportPreviewResponse['entries'][number]>[] = [
  {accessorKey: 'key', header: 'Key'},
  {accessorKey: 'sourceValue', header: 'Source'},
  {accessorKey: 'targetValue', header: 'Imported'}
]
const conflictColumns: TableColumn<ImportPreviewResponse['conflicts'][number]>[] = [
  {accessorKey: 'kind', header: 'Kind'},
  {accessorKey: 'logicalPath', header: 'File'},
  {accessorKey: 'locale', header: 'Locale'},
  {accessorKey: 'key', header: 'Key'},
  {accessorKey: 'message', header: 'Message'}
]

function inferLocaleFromPath(path: string) {
  const lower = path.toLowerCase()
  return APP_LOCALES.find((item) => lower.includes(item.code.toLowerCase()))?.code ?? props.project?.workingLocale ?? 'en-US'
}

function inferBasedOnLocale(locale: string) {
  return props.project?.localeGraph.find((node) => node.code === locale)?.parentCode ?? props.project?.primaryLocale ?? null
}

function resetRows() {
  rows.value = []
}

async function addFiles() {
  const selected = await dialogs.pickImportFiles()
  if (!selected.length) return

  const additions = selected.map<ImportFileInput>((path) => {
    const locale = inferLocaleFromPath(path)
    return {
      path,
      locale,
      basedOnLocale: inferBasedOnLocale(locale),
      logicalPath: dialogs.fileNameFromPath(path)
    }
  })

  const existing = new Set(rows.value.map((row) => row.path))
  rows.value = rows.value.concat(additions.filter((row) => !existing.has(row.path)))
}

function removeRow(index: number) {
  rows.value = rows.value.filter((_, currentIndex) => currentIndex !== index)
}

function basedOnOptions(row: ImportFileInput) {
  const graph = props.project?.localeGraph ?? []
  return [{label: t('labels.none'), value: null}].concat(
      graph.filter((node) => node.code !== row.locale).map((node) => ({
        label: `${node.label} · ${node.code}`,
        value: node.code
      }))
  )
}

function submitPreview() {
  emit('preview', rows.value.filter((row) => row.path.trim()))
}

function commit() {
  if (props.preview) emit('commit', props.preview.previewId)
}

watch(() => props.open, (open) => {
  if (open) resetRows()
})
</script>

<template>
  <UModal
      :description="$t('descriptions.importResources')"
      :open="open"
      :title="$t('actions.importResources')"
      :ui="{ content: 'sm:max-w-6xl', body: 'min-h-0 max-h-[80vh] overflow-y-auto' }"
      @update:open="emit('update:open', $event)"
  >
    <template #body>
      <div class="space-y-6">
        <div class="flex items-center justify-between gap-3">
          <div>
            <h3 class="font-medium">{{ $t('labels.importFiles') }}</h3>
            <p class="text-sm text-muted">{{ $t('descriptions.importFilesHint') }}</p>
          </div>
          <UButton color="neutral" icon="i-lucide-folder-plus" variant="soft" @click="addFiles">
            {{ $t('actions.addFiles') }}
          </UButton>
        </div>

        <UCard v-if="rows.length" :ui="{ body: 'p-0' }">
          <UTable :columns="[
            { accessorKey: 'path', header: t('labels.file') },
            { accessorKey: 'locale', header: t('labels.locale') },
            { accessorKey: 'basedOnLocale', header: t('labels.basedOnLocale') },
            { accessorKey: 'logicalPath', header: t('labels.logicalPath') },
            { id: 'actions', header: '' }
          ]" :data="rows">
            <template #path-cell="{ row }">
              <div class="min-w-0">
                <p class="truncate font-medium">{{ row.original.path.split(/[/\\]/).pop() }}</p>
                <p class="truncate text-xs text-muted">{{ row.original.path }}</p>
              </div>
            </template>
            <template #locale-cell="{ row }">
              <USelectMenu v-model="row.original.locale" :items="localeOptions" value-key="value"/>
            </template>
            <template #basedOnLocale-cell="{ row }">
              <USelectMenu v-model="row.original.basedOnLocale" :items="basedOnOptions(row.original)"
                           value-key="value"/>
            </template>
            <template #logicalPath-cell="{ row }">
              <UInput v-model="row.original.logicalPath" :placeholder="$t('placeholders.logicalPath') as string"/>
            </template>
            <template #actions-cell="{ row }">
              <div class="flex justify-end pr-2">
                <UButton color="error" icon="i-lucide-trash-2" variant="ghost" @click="removeRow(row.index)"/>
              </div>
            </template>
          </UTable>
        </UCard>

        <UAlert
            v-else
            :title="$t('descriptions.importEmpty')"
            color="neutral"
            icon="i-lucide-file-search"
            variant="subtle"
        />

        <div v-if="preview" class="space-y-4">
          <div class="grid gap-4 lg:grid-cols-3">
            <UCard variant="subtle">
              <p class="text-sm text-muted">{{ $t('labels.previewFiles') }}</p>
              <p class="text-2xl font-semibold">{{ preview.totals.files }}</p>
            </UCard>
            <UCard variant="subtle">
              <p class="text-sm text-muted">{{ $t('labels.previewEntries') }}</p>
              <p class="text-2xl font-semibold">{{ preview.totals.entries }}</p>
            </UCard>
            <UCard variant="subtle">
              <p class="text-sm text-muted">{{ $t('labels.conflicts') }}</p>
              <p class="text-2xl font-semibold">{{ preview.totals.conflicts }}</p>
            </UCard>
          </div>

          <UAlert
              :color="preview.conflicts.length ? 'warning' : 'success'"
              :title="preview.conflicts.length ? $t('descriptions.importHasConflicts') : $t('descriptions.importReady')"
              variant="subtle"
          />

          <UCard>
            <template #header><h3 class="font-medium">{{ $t('labels.previewFiles') }}</h3></template>
            <UTable :columns="fileColumns" :data="preview.items"/>
          </UCard>

          <UCard>
            <template #header><h3 class="font-medium">{{ $t('labels.previewEntries') }}</h3></template>
            <UTable :columns="entryColumns" :data="preview.entries"/>
          </UCard>

          <UCard v-if="preview.conflicts.length">
            <template #header><h3 class="font-medium">{{ $t('labels.conflictHints') }}</h3></template>
            <UTable :columns="conflictColumns" :data="preview.conflicts"/>
          </UCard>
        </div>
      </div>
    </template>

    <template #footer>
      <div class="flex items-center justify-between gap-3">
        <UButton color="neutral" variant="soft" @click="emit('update:open', false)">{{ $t('actions.cancel') }}</UButton>
        <div class="flex items-center gap-2">
          <UButton :disabled="!rows.length" color="neutral" icon="i-lucide-search-check" variant="soft"
                   @click="submitPreview">
            {{ $t('actions.previewImport') }}
          </UButton>
          <UButton :disabled="!preview" icon="i-lucide-download" @click="commit">
            {{ $t('actions.commitImport') }}
          </UButton>
        </div>
      </div>
    </template>
  </UModal>
</template>
