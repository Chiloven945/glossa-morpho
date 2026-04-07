<script setup lang="ts">
import {h, resolveComponent} from 'vue'
import type {TableColumn, TabsItem} from '@nuxt/ui'
import type {EntrySummary, ProjectWorkspace, ViewMode} from '~/shared/types/models'

const props = defineProps<{
  project: ProjectWorkspace
  entries: EntrySummary[]
  selectedEntryId: string | null
  searchText: string
  viewMode: ViewMode
  showOnlyMissing: boolean
  bulkSearch: string
  bulkReplacement: string
  bulkUseRegex: boolean
}>()

const emit = defineEmits<{
  updateSearch: [value: string]
  selectEntry: [entryId: string]
  changeView: [view: ViewMode]
  changeShowOnlyMissing: [value: boolean]
  updateBulkSearch: [value: string]
  updateBulkReplacement: [value: string]
  updateBulkUseRegex: [value: boolean]
  applyBulkReplace: []
}>()

const UBadge = resolveComponent('UBadge')
const UButton = resolveComponent('UButton')

const viewItems = computed<TabsItem[]>(() => [
  {label: 'List', value: 'list', icon: 'i-lucide-list'},
  {label: 'Treemap', value: 'treemap', icon: 'i-lucide-chart-no-axes-column'}
])

function statusColor(status: EntrySummary['status']) {
  if (status === 'approved') return 'success'
  if (status === 'reviewed') return 'info'
  if (status === 'translated') return 'primary'
  if (status === 'stale') return 'warning'
  return 'neutral'
}

const columns: TableColumn<EntrySummary>[] = [
  {
    accessorKey: 'key',
    header: 'Key',
    cell: ({row}) => {
      const entry = row.original
      return h('div', {class: 'space-y-1 min-w-0'}, [
        h(UButton, {
          color: entry.id === props.selectedEntryId ? 'primary' : 'neutral',
          variant: entry.id === props.selectedEntryId ? 'soft' : 'ghost',
          class: 'justify-start px-0 font-medium max-w-full',
          onClick: () => emit('selectEntry', entry.id)
        }, () => entry.key),
        h('p', {class: 'text-xs text-muted truncate'}, entry.fileId)
      ])
    },
    meta: {
      class: {
        th: 'w-[26rem]',
        td: 'align-top'
      }
    }
  },
  {
    accessorKey: 'sourceValue',
    header: 'Source',
    cell: ({row}) => h('p', {class: 'line-clamp-2 text-sm'}, row.original.sourceValue || '—')
  },
  {
    accessorKey: 'targetValue',
    header: 'Target',
    cell: ({row}) => h('p', {class: 'line-clamp-2 text-sm'}, row.original.targetValue || '—')
  },
  {
    accessorKey: 'status',
    header: 'Status',
    cell: ({row}) => h(UBadge, {
      color: statusColor(row.original.status),
      variant: 'subtle',
      class: 'capitalize'
    }, () => row.original.status)
  },
  {
    accessorKey: 'updatedAt',
    header: 'Updated',
    cell: ({row}) => h('span', {class: 'text-xs text-muted'}, new Date(row.original.updatedAt).toLocaleString())
  }
]
</script>

<template>
  <div class="space-y-4">
    <div class="grid gap-4 xl:grid-cols-[minmax(0,1fr)_20rem]">
      <UCard>
        <div class="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
          <div class="flex min-w-0 flex-1 flex-col gap-3 lg:flex-row lg:items-center">
            <UInput
                :model-value="searchText"
                icon="i-lucide-search"
                class="w-full lg:max-w-md"
                :placeholder="$t('labels.searchEntries') as string"
                @update:model-value="(value) => emit('updateSearch', String(value))"
            />
            <div class="flex flex-wrap items-center gap-3">
              <UCheckbox
                  :model-value="showOnlyMissing"
                  :label="$t('labels.missingOnly') as string"
                  @update:model-value="(value) => emit('changeShowOnlyMissing', Boolean(value))"
              />
              <UBadge color="neutral" variant="subtle">
                {{ entries.length }} {{ $t('labels.entries') }}
              </UBadge>
            </div>
          </div>

          <UTabs
              :model-value="viewMode"
              :items="viewItems"
              :content="false"
              class="w-full lg:w-auto"
              @update:model-value="(value) => emit('changeView', value as ViewMode)"
          />
        </div>
      </UCard>

      <UCard>
        <div class="grid grid-cols-2 gap-3">
          <div>
            <p class="text-xs text-muted">{{ $t('labels.total') }}</p>
            <p class="text-lg font-semibold">{{ project.stats.total }}</p>
          </div>
          <div>
            <p class="text-xs text-muted">{{ $t('labels.translated') }}</p>
            <p class="text-lg font-semibold">{{ project.stats.translated }}</p>
          </div>
          <div>
            <p class="text-xs text-muted">{{ $t('labels.missing') }}</p>
            <p class="text-lg font-semibold">{{ project.stats.missing }}</p>
          </div>
          <div>
            <p class="text-xs text-muted">{{ $t('labels.reviewed') }}</p>
            <p class="text-lg font-semibold">{{ project.stats.reviewed }}</p>
          </div>
        </div>
      </UCard>
    </div>

    <UCard>
      <template #header>
        <div class="flex items-center justify-between gap-3">
          <div>
            <h2 class="font-semibold">{{ $t('actions.bulkReplace') }}</h2>
            <p class="text-sm text-muted">{{ $t('descriptions.bulkReplace') }}</p>
          </div>
          <UButton icon="i-lucide-scan-search" @click="emit('applyBulkReplace')">
            {{ $t('actions.apply') }}
          </UButton>
        </div>
      </template>

      <div class="grid gap-4 md:grid-cols-2 xl:grid-cols-[minmax(0,1fr)_minmax(0,1fr)_auto]">
        <UFormField :label="$t('labels.search') as string" name="bulk-search">
          <UInput
              :model-value="bulkSearch"
              placeholder="welcome"
              @update:model-value="(value) => emit('updateBulkSearch', String(value))"
          />
        </UFormField>

        <UFormField :label="$t('labels.replacement') as string" name="bulk-replacement">
          <UInput
              :model-value="bulkReplacement"
              placeholder="欢迎"
              @update:model-value="(value) => emit('updateBulkReplacement', String(value))"
          />
        </UFormField>

        <div class="flex items-end">
          <UCheckbox
              :model-value="bulkUseRegex"
              :label="$t('labels.regex') as string"
              @update:model-value="(value) => emit('updateBulkUseRegex', Boolean(value))"
          />
        </div>
      </div>
    </UCard>

    <TreemapView v-if="viewMode === 'treemap'" :nodes="project.treemap"/>

    <UCard v-else>
      <UTable :data="entries" :columns="columns" class="min-h-[32rem]"/>
    </UCard>
  </div>
</template>
