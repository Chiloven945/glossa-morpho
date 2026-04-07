<script setup lang="ts">
import type {TreeItem} from '@nuxt/ui'
import type {ProjectWorkspace} from '~/shared/types/models'

const props = defineProps<{
  project: ProjectWorkspace
  selectedFileId: string | null
}>()

const emit = defineEmits<{
  select: [fileId: string | null]
}>()

const filter = ref('')

const items = computed<TreeItem[]>(() => {
  const q = filter.value.trim().toLowerCase()
  const groups = new Map<string, ProjectWorkspace['files']>()

  for (const file of props.project.files) {
    const matches = !q || `${file.name} ${file.logicalPath} ${file.locale} ${file.format}`.toLowerCase().includes(q)
    if (!matches) continue

    const key = `${file.role}:${file.locale}`
    const group = groups.get(key) ?? []
    group.push(file)
    groups.set(key, group)
  }

  return [
    {
      label: 'All files',
      icon: 'i-lucide-files',
      onSelect: () => emit('select', null),
      class: !props.selectedFileId ? 'text-primary' : undefined
    },
    ...Array.from(groups.entries()).map(([groupKey, files]) => ({
      label: groupKey,
      icon: groupKey.startsWith('source') ? 'i-lucide-languages' : 'i-lucide-file-output',
      defaultExpanded: true,
      children: files.map((file) => ({
        label: file.name,
        icon: 'i-lucide-file-text',
        trailingIcon: file.id === props.selectedFileId ? 'i-lucide-check' : undefined,
        onSelect: () => emit('select', file.id),
        children: [
          {
            label: file.logicalPath,
            icon: 'i-lucide-folder-tree',
            disabled: true
          }
        ]
      }))
    }))
  ]
})
</script>

<template>
  <UCard class="h-full">
    <template #header>
      <div class="space-y-3">
        <div class="flex items-center justify-between gap-3">
          <div>
            <h2 class="font-semibold">{{ $t('labels.files') }}</h2>
            <p class="text-sm text-muted">
              {{ project.files.length }} {{ $t('labels.fileCount') }}
            </p>
          </div>
          <div class="flex flex-wrap gap-2">
            <UBadge color="neutral" variant="subtle">{{ project.sourceLocale }}</UBadge>
            <UBadge color="primary" variant="soft">{{ project.targetLocale }}</UBadge>
          </div>
        </div>

        <UInput
            v-model="filter"
            icon="i-lucide-search"
            :placeholder="$t('labels.searchFiles') as string"
        />
      </div>
    </template>

    <UTree :items="items" color="neutral" class="min-h-[24rem]"/>
  </UCard>
</template>
