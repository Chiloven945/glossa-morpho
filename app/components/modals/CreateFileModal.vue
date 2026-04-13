<script lang="ts" setup>
import type {CreateResourceFileInput, ProjectWorkspace, ResourceFormat} from '#shared/types/models'

const {t} = useI18n()

const props = defineProps<{
  open: boolean
  project?: ProjectWorkspace
  initialLocale?: string | null
}>()

const emit = defineEmits<{
  'update:open': [boolean]
  create: [CreateResourceFileInput]
}>()

const form = reactive({
  locale: '',
  basedOnLocale: '__auto__',
  format: 'json' as ResourceFormat,
  name: '',
  logicalPath: '',
  includeDescendants: true
})

const localeOptions = computed(() => (props.project?.localeGraph || []).map(node => ({
  label: `${node.label} · ${node.code}`,
  value: node.code
})))

const basedOnOptions = computed(() => [
  {label: t('labels.autoFromLocaleChain'), value: '__auto__'},
  {label: t('labels.none'), value: '__none__'},
  ...(props.project?.localeGraph || []).map(node => ({label: `${node.label} · ${node.code}`, value: node.code}))
])

const formatOptions = computed(() => [
  {label: 'JSON', value: 'json'},
  {label: 'YAML', value: 'yaml'},
  {label: 'Properties', value: 'properties'},
  {label: 'RESX', value: 'resx'},
  {label: 'XAML', value: 'xaml'},
  {label: 'XLIFF', value: 'xliff'}
])

const extensionByFormat: Record<ResourceFormat, string> = {
  json: 'json',
  yaml: 'yaml',
  properties: 'properties',
  resx: 'resx',
  xaml: 'xaml',
  xliff: 'xlf'
}

function slugify(value: string) {
  const slug = value
      .trim()
      .replace(/\s+/g, '-')
      .replace(/[^a-zA-Z0-9._-]/g, '-')
      .replace(/-+/g, '-')
      .replace(/^[-.]+|[-.]+$/g, '')
      .toLowerCase()
  return slug || 'strings'
}

function deriveDefaults() {
  const locale = form.locale || props.project?.workingLocale || props.project?.localeGraph[0]?.code || 'en-US'
  const base = slugify(form.name || 'strings')
  const ext = extensionByFormat[form.format]
  form.logicalPath = `locale/${locale}/${base}.${locale}.${ext}`
}

function applyDefaults() {
  form.locale = props.initialLocale || props.project?.workingLocale || props.project?.localeGraph[0]?.code || ''
  form.basedOnLocale = '__auto__'
  form.format = 'json'
  form.name = 'strings'
  form.includeDescendants = true
  deriveDefaults()
}

watch(() => props.open, open => {
  if (open) applyDefaults()
}, {immediate: true})

watch(() => [form.locale, form.format], () => {
  if (!props.open) return
  deriveDefaults()
})

function submit() {
  if (!props.project || !form.locale || !form.name.trim() || !form.logicalPath.trim()) return

  emit('create', {
    projectId: props.project.id,
    name: form.name.trim(),
    logicalPath: form.logicalPath.trim(),
    format: form.format,
    locale: form.locale,
    includeDescendants: form.includeDescendants,
    basedOnLocale: form.basedOnLocale === '__auto__'
        ? props.project.localeGraph.find(node => node.code === form.locale)?.parentCode || null
        : form.basedOnLocale === '__none__'
            ? null
            : form.basedOnLocale
  })
}
</script>

<template>
  <UModal
      :description="$t('descriptions.createFile')"
      :open="open"
      :title="$t('actions.newFile')"
      :ui="{ content: 'sm:max-w-3xl', body: 'max-h-[78vh] overflow-y-auto' }"
      @update:open="emit('update:open', $event)"
  >
    <template #body>
      <UForm :state="form" class="space-y-5">
        <div class="grid gap-4 md:grid-cols-2">
          <UFormField :label="$t('labels.locale') as string">
            <USelectMenu v-model="form.locale" :items="localeOptions" value-key="value"/>
          </UFormField>
          <UFormField :label="$t('labels.basedOnLocale') as string">
            <USelectMenu v-model="form.basedOnLocale" :items="basedOnOptions" value-key="value"/>
          </UFormField>
        </div>

        <div class="grid gap-4 md:grid-cols-[minmax(0,1fr)_180px]">
          <UFormField :label="$t('labels.fileName') as string">
            <UInput v-model="form.name" @blur="deriveDefaults"/>
          </UFormField>
          <UFormField :label="$t('labels.format') as string">
            <USelectMenu v-model="form.format" :items="formatOptions" value-key="value"/>
          </UFormField>
        </div>

        <UFormField :description="$t('descriptions.logicalPath') as string" :label="$t('labels.logicalPath') as string">
          <UInput v-model="form.logicalPath"/>
        </UFormField>

        <UCheckbox v-model="form.includeDescendants" :label="$t('labels.createLinkedDescendants') as string"/>
      </UForm>
    </template>

    <template #footer>
      <div class="flex justify-end gap-2">
        <UButton color="neutral" variant="soft" @click="emit('update:open', false)">{{ $t('actions.cancel') }}</UButton>
        <UButton icon="i-lucide-file-plus-2" @click="submit">{{ $t('actions.create') }}</UButton>
      </div>
    </template>
  </UModal>
</template>
