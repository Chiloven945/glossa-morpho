<script lang="ts" setup>
import type {FileTreeNode} from '~/utils/file-tree'

const props = defineProps<{
  node: FileTreeNode
  level: number
  expandedIds: string[]
}>()

const emit = defineEmits<{
  select: [node: FileTreeNode]
  toggle: [nodeId: string]
  createFile: [localeCode?: string | null]
  openMenu: [payload: { event: MouseEvent; node: FileTreeNode }]
}>()

const isLocale = computed(() => props.node.kind === 'locale')
const hasChildren = computed(() => Boolean(props.node.children?.length))
const isExpanded = computed(() => props.expandedIds.includes(props.node.id))
const indentStyle = computed(() => ({paddingLeft: `${props.level * 14 + 8}px`}))

function handleSelect() {
  emit('select', props.node)
}

function handleMenu(event: MouseEvent) {
  emit('openMenu', {event, node: props.node})
}
</script>

<template>
  <div>
    <div
        :class="[
        'group flex items-start gap-1 rounded-md px-2 py-1.5 text-sm transition-colors',
        node.kind === 'file' && node.selected ? 'bg-elevated ring-1 ring-primary/30' : 'hover:bg-elevated/70'
      ]"
        :style="indentStyle"
        @contextmenu.prevent.stop="handleMenu"
        @mousedown.right.prevent.stop="handleMenu"
    >
      <button
          v-if="isLocale"
          class="mt-0.5 inline-flex h-5 w-5 shrink-0 items-center justify-center rounded text-muted transition hover:bg-accented hover:text-default"
          type="button"
          @click.stop="emit('toggle', node.id)"
      >
        <UIcon :name="isExpanded ? 'i-lucide-chevron-down' : 'i-lucide-chevron-right'" class="h-4 w-4"/>
      </button>
      <span v-else class="inline-block h-5 w-5 shrink-0"/>

      <button class="flex min-w-0 flex-1 items-start gap-2 text-left" type="button" @click="handleSelect">
        <UIcon :name="node.icon" class="mt-0.5 h-4 w-4 shrink-0 text-muted"/>
        <span class="min-w-0 flex-1">
          <span class="block truncate font-medium">{{ node.label }}</span>
          <span v-if="node.description" class="block truncate text-xs text-muted">{{ node.description }}</span>
          <span
              v-if="node.kind === 'file' && (node.relatedLocales?.length || node.basedOnLocale)"
              class="mt-1 flex flex-wrap items-center gap-1.5"
          >
            <UBadge
                v-if="(node.relatedLocales?.length || 0) > 1"
                color="neutral"
                size="sm"
                variant="soft"
            >
              {{ (node.relatedLocales || []).join(' → ') }}
            </UBadge>
            <UBadge v-if="node.basedOnLocale" color="primary" size="sm" variant="subtle">
              ← {{ node.basedOnLocale }}
            </UBadge>
          </span>
        </span>
      </button>

      <UButton
          v-if="node.kind === 'locale'"
          class="opacity-0 transition group-hover:opacity-100"
          color="neutral"
          icon="i-lucide-file-plus"
          size="xs"
          variant="ghost"
          @click.stop="emit('createFile', node.localeCode ?? null)"
      />
    </div>

    <div v-if="isLocale && hasChildren && isExpanded" class="space-y-0.5">
      <FileTreeBranch
          v-for="child in node.children"
          :key="child.id"
          :expanded-ids="expandedIds"
          :level="level + 1"
          :node="child"
          @select="emit('select', $event)"
          @toggle="emit('toggle', $event)"
          @create-file="emit('createFile', $event)"
          @open-menu="emit('openMenu', $event)"
      />
    </div>
  </div>
</template>
