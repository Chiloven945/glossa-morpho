<script lang="ts" setup>
import type {CreateEntryInput, ProjectWorkspace, TranslationStatus} from '#shared/types/models'

const {t} = useI18n()

const props = defineProps<{
  open: boolean
  project?: ProjectWorkspace
  fileId?: string | null
}>()

const emit = defineEmits<{
  'update:open': [boolean]
  create: [CreateEntryInput]
}>()

const form = reactive({
  fileId: '',
  key: '',
  sourceValue: '',
  targetValue: '',
  note: '',
  status: 'new' as TranslationStatus
})

const fileOptions = computed(() => (props.project?.files || []).map((file) => ({
  label: `${file.locale} · ${file.name}`,
  value: file.id
})))
const selectedFile = computed(() => (props.project?.files || []).find((file) => file.id === form.fileId) || null)
const hasUpstream = computed(() => Boolean(selectedFile.value?.basedOnLocale))

const statusOptions = computed(() => [
  {label: t('status.new'), value: 'new'},
  {label: t('status.translated'), value: 'translated'},
  {label: t('status.reviewed'), value: 'reviewed'},
  {label: t('status.approved'), value: 'approved'},
  {label: t('status.stale'), value: 'stale'}
])

watch(() => props.open, (open) => {
  if (!open) return
  form.fileId = props.fileId || props.project?.files[0]?.id || ''
  form.key = ''
  form.sourceValue = ''
  form.targetValue = ''
  form.note = ''
  form.status = 'new'
}, {immediate: true})

function submit() {
  if (!props.project || !form.fileId || !form.key.trim()) return
  emit('create', {
    projectId: props.project.id,
    fileId: form.fileId,
    key: form.key.trim(),
    sourceValue: hasUpstream.value ? form.sourceValue : '',
    targetValue: hasUpstream.value ? form.targetValue : (form.targetValue || form.sourceValue),
    note: form.note,
    status: form.status
  })
}
</script>

<template>
  <UModal :open="open" :title="$t('actions.newEntry')"
          :ui="{ content: 'sm:max-w-3xl', body: 'max-h-[78vh] overflow-y-auto' }"
          @update:open="emit('update:open', $event)">
    <template #body>
      <UForm :state="form" class="space-y-4">
        <UFormField :label="$t('labels.file') as string">
          <USelectMenu v-model="form.fileId" :items="fileOptions" value-key="value"/>
        </UFormField>
        <UFormField :label="$t('labels.key') as string">
          <UInput v-model="form.key"/>
        </UFormField>
        <UFormField v-if="hasUpstream" :label="$t('labels.sourceValue') as string">
          <UTextarea v-model="form.sourceValue" :rows="4"/>
        </UFormField>
        <UFormField v-if="hasUpstream" :label="$t('labels.targetValue') as string">
          <UTextarea v-model="form.targetValue" :rows="4"/>
        </UFormField>
        <UFormField v-else :label="$t('labels.sourceValue') as string">
          <UTextarea v-model="form.sourceValue" :rows="4"/>
        </UFormField>
        <UFormField :label="$t('labels.note') as string">
          <UTextarea v-model="form.note" :rows="3"/>
        </UFormField>
        <UFormField :label="$t('labels.status') as string">
          <USelectMenu v-model="form.status" :items="statusOptions" value-key="value"/>
        </UFormField>
      </UForm>
    </template>
    <template #footer>
      <div class="flex justify-end gap-2">
        <UButton color="neutral" variant="soft" @click="emit('update:open', false)">{{ $t('actions.cancel') }}</UButton>
        <UButton @click="submit">{{ $t('actions.create') }}</UButton>
      </div>
    </template>
  </UModal>
</template>
