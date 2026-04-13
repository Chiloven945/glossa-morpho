<script lang="ts" setup>
import type {BatchExportInput, ProjectWorkspace} from '#shared/types/models'

const {t} = useI18n()
const dialogs = useSystemDialogs()

const props = defineProps<{
  open: boolean
  project?: ProjectWorkspace
}>()

const emit = defineEmits<{
  'update:open': [boolean]
  export: [BatchExportInput]
}>()

const selectedIds = ref<string[]>([])
const outputDirectory = ref('')

const fileOptions = computed(() => props.project?.files.map((file) => ({
  id: file.id,
  label: `${file.name} · ${file.locale}`,
  description: file.logicalPath
})) || [])

watch(() => props.open, (open) => {
  if (!open) return
  selectedIds.value = props.project?.files.map((file) => file.id) || []
  outputDirectory.value = ''
})

async function chooseDirectory() {
  const selected = await dialogs.pickDirectory()
  if (selected) outputDirectory.value = selected
}

function toggleFile(fileId: string, checked: boolean | 'indeterminate') {
  if (checked) selectedIds.value = Array.from(new Set([...selectedIds.value, fileId]))
  else selectedIds.value = selectedIds.value.filter((item) => item !== fileId)
}


function selectAllFiles() {
  selectedIds.value = fileOptions.value.map((item) => item.id)
}

function clearSelection() {
  selectedIds.value = []
}

function submit() {
  if (!props.project || !selectedIds.value.length || !outputDirectory.value) return
  emit('export', {
    projectId: props.project.id,
    fileIds: selectedIds.value,
    outputDirectory: outputDirectory.value
  })
}
</script>

<template>
  <UModal :description="$t('descriptions.batchExport')" :open="open" :title="$t('actions.batchExport')"
          :ui="{ content: 'sm:max-w-4xl', body: 'space-y-5 max-h-[78vh] overflow-y-auto' }"
          @update:open="emit('update:open', $event)">
    <template #body>
      <div class="space-y-4">
        <div class="flex items-center justify-between gap-3">
          <p class="text-sm text-muted">{{ $t('labels.fileCount') }}: {{ fileOptions.length }}</p>
          <div class="flex gap-2">
            <UButton color="neutral" size="sm" variant="soft" @click="selectAllFiles">{{
                $t('actions.selectAll')
              }}
            </UButton>
            <UButton color="neutral" size="sm" variant="soft" @click="clearSelection">{{
                $t('actions.clearSelection')
              }}
            </UButton>
          </div>
        </div>

        <UCard :ui="{ body: 'divide-y divide-default p-0' }">
          <div v-for="file in fileOptions" :key="file.id" class="flex items-start gap-3 px-4 py-3">
            <UCheckbox :model-value="selectedIds.includes(file.id)" @update:model-value="toggleFile(file.id, $event)"/>
            <div class="min-w-0 flex-1">
              <p class="truncate text-sm font-medium">{{ file.label }}</p>
              <p class="truncate text-xs text-muted">{{ file.description }}</p>
            </div>
          </div>
        </UCard>

        <div class="grid gap-3 md:grid-cols-[minmax(0,1fr)_auto] md:items-end">
          <UFormField :label="$t('labels.outputDirectory') as string">
            <UInput :model-value="outputDirectory" readonly/>
          </UFormField>
          <UButton color="neutral" icon="i-lucide-folder-open" variant="soft" @click="chooseDirectory">
            {{ $t('actions.chooseFolder') }}
          </UButton>
        </div>
      </div>
    </template>
    <template #footer>
      <div class="flex justify-end gap-2">
        <UButton color="neutral" variant="soft" @click="emit('update:open', false)">{{ $t('actions.cancel') }}</UButton>
        <UButton icon="i-lucide-files" @click="submit">{{ $t('actions.batchExport') }}</UButton>
      </div>
    </template>
  </UModal>
</template>
