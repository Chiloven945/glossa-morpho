<script lang="ts" setup>
import type {DeleteResourceFileInput, ProjectWorkspace, ResourceFileNode} from '#shared/types/models'

const props = defineProps<{
  open: boolean
  project?: ProjectWorkspace
  file?: ResourceFileNode | null
}>()

const emit = defineEmits<{
  'update:open': [boolean]
  confirm: [DeleteResourceFileInput]
}>()

const includeRelated = ref(true)

watch(() => props.open, (open) => {
  if (open) includeRelated.value = true
})

function submit() {
  if (!props.project || !props.file) return
  emit('confirm', {
    projectId: props.project.id,
    fileId: props.file.id,
    includeRelated: includeRelated.value
  })
}
</script>

<template>
  <UModal :description="$t('descriptions.deleteFile')" :open="open" :title="$t('actions.deleteFile')"
          :ui="{ content: 'sm:max-w-xl' }" @update:open="emit('update:open', $event)">
    <template #body>
      <div class="space-y-4">
        <UAlert :description="file?.logicalPath || ''" :title="file?.name || ''" color="error"
                icon="i-lucide-triangle-alert"/>
        <UCheckbox v-model="includeRelated" :label="$t('labels.deleteRelatedFiles') as string"/>
      </div>
    </template>
    <template #footer>
      <div class="flex justify-end gap-2">
        <UButton color="neutral" variant="soft" @click="emit('update:open', false)">{{ $t('actions.cancel') }}</UButton>
        <UButton color="error" icon="i-lucide-trash-2" @click="submit">{{ $t('actions.deleteFile') }}</UButton>
      </div>
    </template>
  </UModal>
</template>
