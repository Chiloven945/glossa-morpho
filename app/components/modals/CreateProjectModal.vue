<script lang="ts" setup>
import type {CreateProjectInput, LocaleDependencyNode, ProjectCompressionType} from '#shared/types/models'
import {APP_LOCALES} from '#shared/constants/locales'

const {t} = useI18n()
const dialogs = useSystemDialogs()

const props = defineProps<{ open: boolean }>()
const emit = defineEmits<{ 'update:open': [value: boolean]; create: [payload: CreateProjectInput] }>()

const form = reactive({
  name: 'Starter Project',
  path: '',
  primaryLocale: 'en-US',
  workingLocale: 'ja-JP',
  localeGraph: [
    {code: 'en-US', label: 'English', parentCode: null},
    {code: 'zh-CN', label: '简体中文', parentCode: 'en-US'},
    {code: 'ja-JP', label: '日本語', parentCode: 'zh-CN'}
  ] as LocaleDependencyNode[],
  archiveFormat: 'lzma2' as ProjectCompressionType,
  keySegmentationProfiles: 'dot, camel'
})

const localeOptions = APP_LOCALES.map(item => ({label: `${item.code} · ${item.label}`, value: item.code}))
const localeLabelMap = new Map(APP_LOCALES.map(item => [item.code, item.label]))
const localeGraphOptions = computed(() => form.localeGraph.map(node => ({
  label: `${node.code} · ${node.label}`,
  value: node.code
})))
const compressionOptions = [
  {label: 'LZMA2', value: 'lzma2'},
  {label: 'Deflate', value: 'deflate'}
]

function localeLabel(code: string) {
  return localeLabelMap.get(code) ?? code
}

function resetForm() {
  form.name = 'Starter Project'
  form.path = ''
  form.primaryLocale = 'en-US'
  form.workingLocale = 'ja-JP'
  form.localeGraph = [
    {code: 'en-US', label: 'English', parentCode: null},
    {code: 'zh-CN', label: '简体中文', parentCode: 'en-US'},
    {code: 'ja-JP', label: '日本語', parentCode: 'zh-CN'}
  ]
  form.archiveFormat = 'lzma2'
  form.keySegmentationProfiles = 'dot, camel'
}

function addLocaleNode() {
  const fallbackParent = form.localeGraph.at(-1)?.code ?? null
  form.localeGraph.push({code: 'fr-FR', label: localeLabel('fr-FR'), parentCode: fallbackParent})
}

function removeLocaleNode(index: number) {
  const node = form.localeGraph[index]
  if (!node || form.localeGraph.length === 1) return
  form.localeGraph = form.localeGraph.filter((_, currentIndex) => currentIndex !== index).map((item) => item.parentCode === node.code ? {
    ...item,
    parentCode: null
  } : item)
  if (!form.localeGraph.some((item) => item.code === form.primaryLocale)) form.primaryLocale = form.localeGraph[0]?.code ?? 'en-US'
  if (!form.localeGraph.some((item) => item.code === form.workingLocale)) form.workingLocale = form.localeGraph.at(-1)?.code ?? form.primaryLocale
}

function updateLocaleNodeCode(index: number, code: string) {
  const option = APP_LOCALES.find((item) => item.code === code)
  const previousCode = form.localeGraph[index]?.code
  const currentNode = form.localeGraph[index]
  if (!currentNode) return
  form.localeGraph[index] = {...currentNode, code, label: option?.label ?? code}
  form.localeGraph = form.localeGraph.map((item, currentIndex) => {
    if (currentIndex === index) return item
    if (item.parentCode === previousCode) return {...item, parentCode: code}
    return item
  })
  if (form.primaryLocale === previousCode) form.primaryLocale = code
  if (form.workingLocale === previousCode) form.workingLocale = code
}

function updateLocaleNodeParent(index: number, value: unknown) {
  const node = form.localeGraph[index]
  if (!node) return
  node.parentCode = typeof value === 'string' ? value : null
}

function parentOptionsFor(index: number) {
  const selfCode = form.localeGraph[index]?.code
  return [
    {label: t('labels.rootLocale'), value: null},
    ...form.localeGraph.filter((item) => item.code !== selfCode).map((item) => ({
      label: `${item.code} · ${item.label}`,
      value: item.code
    }))
  ]
}

function normalizedLocaleGraph() {
  const knownCodes = new Set(form.localeGraph.map((node) => node.code).filter(Boolean))
  const seen = new Set<string>()
  const nodes: LocaleDependencyNode[] = []
  for (const node of form.localeGraph) {
    if (!node.code || seen.has(node.code)) continue
    seen.add(node.code)
    nodes.push({
      code: node.code,
      label: localeLabel(node.code),
      parentCode: node.parentCode && knownCodes.has(node.parentCode) ? node.parentCode : null
    })
  }
  if (nodes.length && !nodes.some((item) => item.parentCode === null)) nodes[0] = {...nodes[0], parentCode: null}
  return nodes.map((node) => ({...node, parentCode: node.code === form.primaryLocale ? null : node.parentCode}))
}

async function chooseLocation() {
  const selected = await dialogs.pickProjectSavePath({
    defaultPath: form.path || `${form.name || 'project'}.gmproj`,
    projectName: form.name
  })
  if (selected) form.path = selected
}

async function submit() {
  if (!form.path) {
    await chooseLocation()
    if (!form.path) return
  }
  emit('create', {
    name: form.name,
    path: form.path || undefined,
    primaryLocale: form.primaryLocale,
    workingLocale: form.workingLocale,
    localeGraph: normalizedLocaleGraph(),
    archiveFormat: form.archiveFormat,
    keySegmentationProfiles: form.keySegmentationProfiles.split(',').map(item => item.trim()).filter(Boolean)
  })
}

watch(() => props.open, (open) => {
  if (open) resetForm()
})
</script>

<template>
  <UModal :description="$t('descriptions.createProject')" :open="open" :title="$t('actions.newProject')"
          :ui="{ content: 'sm:max-w-6xl', body: 'min-h-0 max-h-[80vh] overflow-y-auto' }"
          @update:open="emit('update:open', $event)">
    <template #body>
      <div class="space-y-6">
        <UCard>
          <div class="grid gap-4 lg:grid-cols-[minmax(0,1fr)_260px]">
            <UFormField :label="$t('labels.projectName') as string">
              <UInput v-model="form.name"/>
            </UFormField>
            <UFormField :label="$t('labels.compressionType') as string">
              <USelectMenu v-model="form.archiveFormat" :items="compressionOptions" value-key="value"/>
            </UFormField>
          </div>
          <div class="mt-4 grid gap-3 lg:grid-cols-[minmax(0,1fr)_auto] lg:items-end">
            <UFormField :label="$t('labels.location') as string">
              <UInput :model-value="form.path" readonly/>
            </UFormField>
            <UButton color="neutral" icon="i-lucide-folder-open" variant="soft" @click="chooseLocation">
              {{ $t('actions.chooseLocation') }}
            </UButton>
          </div>
        </UCard>

        <UCard :ui="{ body: 'space-y-4' }">
          <template #header>
            <div class="flex items-center justify-between gap-3">
              <div>
                <h3 class="font-medium">{{ $t('labels.localeChain') }}</h3>
                <p class="text-sm text-muted">{{ $t('descriptions.localeChain') }}</p>
              </div>
              <UButton color="neutral" icon="i-lucide-plus" variant="outline" @click="addLocaleNode">
                {{ $t('actions.addLocale') }}
              </UButton>
            </div>
          </template>

          <UTable :columns="[
            { accessorKey: 'code', header: t('labels.locale') },
            { accessorKey: 'parentCode', header: t('labels.dependsOn') },
            { id: 'actions', header: '' }
          ]" :data="form.localeGraph">
            <template #code-cell="{ row }">
              <USelectMenu :items="localeOptions" :model-value="row.original.code" value-key="value"
                           @update:model-value="value => updateLocaleNodeCode(row.index, String(value))"/>
            </template>
            <template #parentCode-cell="{ row }">
              <USelectMenu :items="parentOptionsFor(row.index)" :model-value="row.original.parentCode" value-key="value"
                           @update:model-value="value => updateLocaleNodeParent(row.index, value)"/>
            </template>
            <template #actions-cell="{ row }">
              <div class="flex justify-end pr-2">
                <UButton color="error" icon="i-lucide-trash-2" variant="ghost" @click="removeLocaleNode(row.index)"/>
              </div>
            </template>
          </UTable>
        </UCard>

        <UCard>
          <div class="grid gap-4 lg:grid-cols-3">
            <UFormField :label="$t('labels.primaryLocale') as string">
              <USelectMenu v-model="form.primaryLocale" :items="localeGraphOptions" value-key="value"/>
            </UFormField>
            <UFormField :label="$t('labels.workingLocale') as string">
              <USelectMenu v-model="form.workingLocale" :items="localeGraphOptions" value-key="value"/>
            </UFormField>
            <UFormField :label="$t('labels.keySegmentationProfiles') as string">
              <UInput v-model="form.keySegmentationProfiles"/>
            </UFormField>
          </div>
        </UCard>
      </div>
    </template>
    <template #footer>
      <div class="flex justify-end gap-2">
        <UButton color="neutral" variant="soft" @click="emit('update:open', false)">{{ $t('actions.cancel') }}</UButton>
        <UButton @click="submit">{{ $t('actions.create') }}</UButton>
      </div>
    </template>
  </UModal>
</template>
