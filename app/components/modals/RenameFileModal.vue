<script lang="ts" setup>
import type {ProjectWorkspace, RenameResourceFileInput, ResourceFileNode} from '#shared/types/models'

const {t} = useI18n()

const props = defineProps<{
  open: boolean
  project?: ProjectWorkspace
  file?: ResourceFileNode | null
}>()

const emit = defineEmits<{
  'update:open': [boolean]
  save: [RenameResourceFileInput]
}>()

const form = reactive({
  name: '',
  logicalPath: '',
  includeRelated: true
})

function resetForm() {
  form.name = props.file?.name || ''
  form.logicalPath = props.file?.logicalPath || ''
  form.includeRelated = true
}

watch(() => props.open, (open) => {
  if (open) resetForm()
})

function submit() {
  if (!props.project || !props.file || !form.logicalPath.trim()) return
  emit('save', {
    projectId: props.project.id,
    fileId: props.file.id,
    name: form.name.trim() || props.file.name,
    logicalPath: form.logicalPath.trim(),
    includeRelated: form.includeRelated
  })
}
</script>

<template>
  <UModal :description="$t('descriptions.renameFile')" :open="open" :title="$t('actions.renameFile')"
          :ui="{ content: 'sm:max-w-2xl', body: 'space-y-5' }" @update:open="emit('update:open', $event)">
    <template #body>
      <UForm :state="form" class="space-y-4">
        <UFormField :label="$t('labels.fileName') as string">
          <UInput v-model="form.name"/>
        </UFormField>
        <UFormField :description="$t('descriptions.renameRelatedFiles') as string"
                    :label="$t('labels.logicalPath') as string">
          <UInput v-model="form.logicalPath"/>
        </UFormField>
        <UCheckbox v-model="form.includeRelated" :label="$t('labels.renameRelatedFiles') as string"/>
      </UForm>
    </template>
    <template #footer>
      <div class="flex justify-end gap-2">
        <UButton color="neutral" variant="soft" @click="emit('update:open', false)">{{ $t('actions.cancel') }}</UButton>
        <UButton icon="i-lucide-file-pen-line" @click="submit">{{ $t('actions.renameFile') }}</UButton>
      </div>
    </template>
  </UModal>
</template>
