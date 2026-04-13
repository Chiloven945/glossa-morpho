<script lang="ts" setup>
import type {ProjectWorkspace} from '#shared/types/models'

const props = defineProps<{
  open: boolean
  project?: ProjectWorkspace
  selectedFileId?: string | null
}>()
const emit = defineEmits<{
  'update:open': [value: boolean]
  export: [{ fileId: string; outputPath?: string }]
}>()

const {t} = useI18n()
const dialogs = useSystemDialogs()
const selectedFileId = ref<string | null>(null)
const outputPath = ref('')

const fileOptions = computed(() => (props.project?.files || []).map(file => ({
  label: `${file.name} · ${file.locale} · ${file.format.toUpperCase()}`,
  value: file.id,
  description: file.logicalPath
})))

const selectedFile = computed(() => props.project?.files.find(file => file.id === selectedFileId.value) || null)

function resetState() {
  selectedFileId.value = props.selectedFileId || props.project?.files[0]?.id || null
  outputPath.value = ''
}

async function chooseFile() {
  if (!selectedFile.value) return
  const selected = await dialogs.pickExportFile({
    format: selectedFile.value.format,
    defaultPath: selectedFile.value.logicalPath || selectedFile.value.name
  })
  if (selected) outputPath.value = selected
}

async function submit() {
  if (!selectedFileId.value) return
  if (!outputPath.value) {
    await chooseFile()
    if (!outputPath.value) return
  }
  emit('export', {fileId: selectedFileId.value, outputPath: outputPath.value || undefined})
}

watch(() => props.open, open => {
  if (open) resetState()
})
</script>

<template>
  <UModal
      :description="$t('descriptions.exportResources')"
      :open="open"
      :title="$t('actions.exportResources')"
      :ui="{ content: 'sm:max-w-3xl', body: 'space-y-6' }"
      @update:open="emit('update:open', $event)"
  >
    <template #body>
      <UCard variant="subtle">
        <div class="grid gap-4 md:grid-cols-[minmax(0,1fr)_auto] md:items-end">
          <UFormField :description="$t('descriptions.exportSingleFile') as string" :label="$t('labels.file') as string">
            <USelectMenu v-model="selectedFileId" :items="fileOptions" value-key="value"/>
          </UFormField>
          <UButton color="neutral" icon="i-lucide-file-output" variant="soft" @click="chooseFile">
            {{ $t('actions.chooseFile') }}
          </UButton>
        </div>
      </UCard>

      <UCard v-if="selectedFile" :ui="{ body: 'space-y-3' }">
        <div class="flex flex-wrap items-center gap-2">
          <UBadge color="neutral" variant="subtle">{{ selectedFile.format.toUpperCase() }}</UBadge>
          <UBadge color="neutral" variant="subtle">{{ selectedFile.locale }}</UBadge>
          <span class="text-sm text-muted">{{ selectedFile.logicalPath }}</span>
        </div>

        <UFormField :label="$t('labels.outputFile') as string">
          <UInput :model-value="outputPath" class="min-w-0" readonly/>
        </UFormField>
      </UCard>
    </template>
    <template #footer>
      <div class="flex items-center justify-between gap-3">
        <UButton color="neutral" variant="soft" @click="emit('update:open', false)">{{ $t('actions.cancel') }}</UButton>
        <UButton icon="i-lucide-file-output" @click="submit">{{ $t('actions.exportResources') }}</UButton>
      </div>
    </template>
  </UModal>
</template>
