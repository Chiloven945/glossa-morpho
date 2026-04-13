<script lang="ts" setup>
import type {ProjectCompressionType, ProjectWorkspace, UpdateProjectMetadataInput, ViewMode} from '#shared/types/models'
import {APP_LOCALES} from '#shared/constants/locales'

const {t} = useI18n()

const props = defineProps<{ open: boolean; project?: ProjectWorkspace }>()
const emit = defineEmits<{ 'update:open': [boolean]; save: [UpdateProjectMetadataInput] }>()

const form = reactive<UpdateProjectMetadataInput>({
  projectId: '',
  name: '',
  primaryLocale: 'en-US',
  workingLocale: 'en-US',
  archiveFormat: 'lzma2',
  keySegmentationProfiles: ['dot', 'camel'],
  defaultView: 'list',
  defaultSort: 'updatedDesc'
})
const segmentationInput = ref('dot, camel')

const localeOptions = APP_LOCALES.map((item) => ({label: `${item.label} · ${item.code}`, value: item.code}))
const compressionOptions = [
  {label: 'LZMA2', value: 'lzma2'},
  {label: 'Deflate', value: 'deflate'}
]
const viewOptions = [
  {label: t('actions.list'), value: 'list'},
  {label: t('actions.treemap'), value: 'treemap'}
]
const sortOptions = [
  {label: t('labels.sortUpdatedDesc'), value: 'updatedDesc'},
  {label: t('labels.sortKeyAsc'), value: 'keyAsc'},
  {label: t('labels.sortStatus'), value: 'status'}
]

watch(() => props.project, (project) => {
  if (!project) return
  form.projectId = project.id
  form.name = project.name
  form.primaryLocale = project.primaryLocale
  form.workingLocale = project.workingLocale
  form.archiveFormat = (project.archiveFormat || 'lzma2') as ProjectCompressionType
  form.keySegmentationProfiles = [...(project.keySegmentationProfiles || ['dot', 'camel'])]
  form.defaultView = (project.defaultView || 'list') as ViewMode
  form.defaultSort = (project.defaultSort || 'updatedDesc') as 'updatedDesc' | 'keyAsc' | 'status'
  segmentationInput.value = form.keySegmentationProfiles.join(', ')
}, {immediate: true})

function submit() {
  emit('save', {
    ...form,
    keySegmentationProfiles: segmentationInput.value.split(',').map((item) => item.trim()).filter(Boolean)
  })
}
</script>

<template>
  <UModal :open="open" :title="$t('menu.projectMetadata')"
          :ui="{ content: 'sm:max-w-5xl', body: 'max-h-[80vh] overflow-y-auto' }"
          @update:open="emit('update:open', $event)">
    <template #body>
      <div class="space-y-6">
        <UCard>
          <div class="grid gap-4 lg:grid-cols-[minmax(0,1fr)_240px]">
            <UFormField :label="$t('labels.projectName') as string">
              <UInput v-model="form.name"/>
            </UFormField>
            <UFormField :label="$t('labels.compressionType') as string">
              <USelectMenu v-model="form.archiveFormat" :items="compressionOptions" value-key="value"/>
            </UFormField>
          </div>
        </UCard>
        <UCard>
          <div class="grid gap-4 lg:grid-cols-2">
            <UFormField :label="$t('labels.primaryLocale') as string">
              <USelectMenu v-model="form.primaryLocale" :items="localeOptions" value-key="value"/>
            </UFormField>
            <UFormField :label="$t('labels.workingLocale') as string">
              <USelectMenu v-model="form.workingLocale" :items="localeOptions" value-key="value"/>
            </UFormField>
          </div>
        </UCard>
        <UCard>
          <div class="grid gap-4 lg:grid-cols-2">
            <UFormField :label="$t('labels.defaultView') as string">
              <USelectMenu v-model="form.defaultView" :items="viewOptions" value-key="value"/>
            </UFormField>
            <UFormField :label="$t('labels.defaultSort') as string">
              <USelectMenu v-model="form.defaultSort" :items="sortOptions" value-key="value"/>
            </UFormField>
          </div>
          <div class="mt-4">
            <UFormField :description="$t('descriptions.keySegmentationProfiles') as string"
                        :label="$t('labels.keySegmentationProfiles') as string">
              <UInput v-model="segmentationInput"/>
            </UFormField>
          </div>
        </UCard>
      </div>
    </template>
    <template #footer>
      <div class="flex justify-end gap-2">
        <UButton color="neutral" variant="soft" @click="emit('update:open', false)">{{ $t('actions.cancel') }}</UButton>
        <UButton @click="submit">{{ $t('actions.save') }}</UButton>
      </div>
    </template>
  </UModal>
</template>
