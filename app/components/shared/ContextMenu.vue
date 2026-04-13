<script lang="ts" setup>
export interface ContextMenuItem {
  label: string
  value: string
  icon?: string
  disabled?: boolean
  color?: 'neutral' | 'primary' | 'error' | 'warning' | 'success' | 'info'
}

const props = defineProps<{
  open: boolean
  x: number
  y: number
  items: ContextMenuItem[]
}>()

const emit = defineEmits<{
  'update:open': [boolean]
  select: [string]
}>()

const menuRef = ref<HTMLElement | null>(null)
const position = reactive({x: 8, y: 8})

function closeMenu() {
  emit('update:open', false)
}

function choose(item: ContextMenuItem) {
  if (item.disabled) return
  emit('select', item.value)
  closeMenu()
}

function updatePosition() {
  const menuWidth = menuRef.value?.offsetWidth ?? 200
  const menuHeight = menuRef.value?.offsetHeight ?? 0
  position.x = Math.max(8, Math.min(props.x, window.innerWidth - menuWidth - 8))
  position.y = Math.max(8, Math.min(props.y, window.innerHeight - menuHeight - 8))
}

watch(() => props.open, async (open) => {
  if (!open) return
  await nextTick()
  updatePosition()
})

watch(() => [props.x, props.y, props.items.length], async () => {
  if (!props.open) return
  await nextTick()
  updatePosition()
})

const onKeyDown = (event: KeyboardEvent) => {
  if (event.key === 'Escape' && props.open) closeMenu()
}

onMounted(() => window.addEventListener('keydown', onKeyDown))
onBeforeUnmount(() => window.removeEventListener('keydown', onKeyDown))
</script>

<template>
  <Teleport to="body">
    <div v-if="open" class="fixed inset-0 z-[1200]" @mousedown.left="closeMenu" @contextmenu.prevent>
      <div
          ref="menuRef"
          :style="{ left: `${position.x}px`, top: `${position.y}px` }"
          class="fixed z-[1201] min-w-44 overflow-hidden rounded-lg border border-default bg-default p-1 shadow-2xl ring-1 ring-default/60"
          @mousedown.stop
          @click.stop
          @contextmenu.prevent.stop
      >
        <button
            v-for="item in items"
            :key="item.value"
            :class="[
            'flex w-full items-center gap-2 rounded-md px-2.5 py-2 text-left text-sm transition-colors',
            item.disabled ? 'cursor-not-allowed opacity-40' : 'hover:bg-elevated',
            item.color === 'error' ? 'text-error' : 'text-default'
          ]"
            :disabled="item.disabled"
            type="button"
            @click="choose(item)"
        >
          <UIcon v-if="item.icon" :name="item.icon" class="h-4 w-4 shrink-0"/>
          <span class="truncate">{{ item.label }}</span>
        </button>
      </div>
    </div>
  </Teleport>
</template>
