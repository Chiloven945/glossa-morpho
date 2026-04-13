<script lang="ts" setup>
import type {NavigationMenuItem} from '@nuxt/ui'

const {t} = useI18n()

const props = defineProps<{
  hasActiveProject?: boolean
}>()

const emit = defineEmits<{
  createProject: []
  createFrom: []
  openProject: []
  saveProject: []
  saveAsProject: []
  closeProject: []
  importFiles: []
  exportFiles: []
  exportBatchFiles: []
  openCommandPalette: []
  openFind: []
  openReplace: []
  openProjectMetadata: []
  openPreferences: []
  openBulkEdit: []
  undo: []
  redo: []
  cut: []
  copy: []
  paste: []
  deleteSelection: []
  selectAll: []
  openAbout: []
  openLicense: []
  openDocumentation: []
  reportIssue: []
  openFeedback: []
}>()

const isMaximized = ref(false)

function menuClass(options?: { disabled?: boolean; divider?: boolean }) {
  const parts = [] as string[]
  if (options?.divider) parts.push('mt-1 border-t border-default pt-2')
  if (options?.disabled) parts.push('opacity-50')
  return parts.join(' ')
}

const menuItems = computed<NavigationMenuItem[]>(() => [
  {
    label: t('menu.file'),
    children: [
      {
        label: t('menu.openProject'),
        icon: 'i-lucide-folder-open',
        onSelect: () => emit('openProject')
      },
      {
        label: t('menu.save'),
        icon: 'i-lucide-save',
        class: menuClass({disabled: !props.hasActiveProject}),
        onSelect: () => props.hasActiveProject && emit('saveProject')
      },
      {
        label: t('menu.saveAs'),
        icon: 'i-lucide-save-all',
        class: menuClass({disabled: !props.hasActiveProject}),
        onSelect: () => props.hasActiveProject && emit('saveAsProject')
      },
      {
        label: t('menu.closeProject'),
        icon: 'i-lucide-folder-x',
        class: menuClass({divider: true, disabled: !props.hasActiveProject}),
        onSelect: () => props.hasActiveProject && emit('closeProject')
      },
      {
        label: t('menu.newProject'),
        icon: 'i-lucide-folder-plus',
        class: menuClass({divider: true}),
        onSelect: () => emit('createProject')
      },
      {
        label: t('menu.createFrom'),
        icon: 'i-lucide-copy-plus',
        onSelect: () => emit('createFrom')
      },
      {
        label: t('menu.import'),
        icon: 'i-lucide-download',
        class: menuClass({divider: true, disabled: !props.hasActiveProject}),
        onSelect: () => props.hasActiveProject && emit('importFiles')
      },
      {
        label: t('menu.export'),
        icon: 'i-lucide-upload',
        class: menuClass({disabled: !props.hasActiveProject}),
        onSelect: () => props.hasActiveProject && emit('exportFiles')
      },
      {
        label: t('menu.batchExport'),
        icon: 'i-lucide-files',
        class: menuClass({disabled: !props.hasActiveProject}),
        onSelect: () => props.hasActiveProject && emit('exportBatchFiles')
      },
      {
        label: t('menu.exit'),
        icon: 'i-lucide-power',
        class: menuClass({divider: true}),
        onSelect: () => closeWindow()
      }
    ]
  },
  {
    label: t('menu.edit'),
    children: [
      {
        label: t('menu.undo'),
        icon: 'i-lucide-undo-2',
        onSelect: () => emit('undo')
      },
      {
        label: t('menu.redo'),
        icon: 'i-lucide-redo-2',
        onSelect: () => emit('redo')
      },
      {
        label: t('menu.cut'),
        icon: 'i-lucide-scissors',
        class: menuClass({divider: true}),
        onSelect: () => emit('cut')
      },
      {
        label: t('menu.copy'),
        icon: 'i-lucide-copy',
        onSelect: () => emit('copy')
      },
      {
        label: t('menu.paste'),
        icon: 'i-lucide-clipboard-paste',
        onSelect: () => emit('paste')
      },
      {
        label: t('menu.delete'),
        icon: 'i-lucide-trash-2',
        onSelect: () => emit('deleteSelection')
      },
      {
        label: t('menu.selectAll'),
        icon: 'i-lucide-list-checks',
        onSelect: () => emit('selectAll')
      },
      {
        label: t('menu.find'),
        icon: 'i-lucide-search',
        class: menuClass({divider: true}),
        onSelect: () => emit('openFind')
      },
      {
        label: t('menu.replace'),
        icon: 'i-lucide-replace',
        onSelect: () => emit('openReplace')
      },
      {
        label: t('menu.projectMetadata'),
        icon: 'i-lucide-file-badge-2',
        class: menuClass({divider: true, disabled: !props.hasActiveProject}),
        onSelect: () => props.hasActiveProject && emit('openProjectMetadata')
      },
      {
        label: t('menu.preferences'),
        icon: 'i-lucide-settings-2',
        onSelect: () => emit('openPreferences')
      }
    ]
  },
  {
    label: t('menu.view'),
    children: [
      {
        label: t('menu.commandPalette'),
        icon: 'i-lucide-command',
        onSelect: () => emit('openCommandPalette')
      }
    ]
  },
  {
    label: t('menu.tools'),
    children: [
      {
        label: t('menu.bulkEdit'),
        icon: 'i-lucide-wand-sparkles',
        class: menuClass({disabled: !props.hasActiveProject}),
        onSelect: () => props.hasActiveProject && emit('openBulkEdit')
      }
    ]
  },
  {
    label: t('menu.help'),
    children: [
      {
        label: t('menu.about'),
        icon: 'i-lucide-info',
        onSelect: () => emit('openAbout')
      },
      {
        label: t('menu.license'),
        icon: 'i-lucide-scroll-text',
        onSelect: () => emit('openLicense')
      },
      {
        label: t('menu.documentation'),
        icon: 'i-lucide-book-open',
        class: menuClass({divider: true}),
        onSelect: () => emit('openDocumentation')
      },
      {
        label: t('menu.reportIssue'),
        icon: 'i-lucide-bug',
        onSelect: () => emit('reportIssue')
      },
      {
        label: t('menu.feedback'),
        icon: 'i-lucide-message-square-more',
        onSelect: () => emit('openFeedback')
      }
    ]
  }
])

async function getWindow() {
  try {
    const {getCurrentWindow} = await import('@tauri-apps/api/window')
    return getCurrentWindow()
  } catch {
    return null
  }
}

async function withCurrentWindow(action: (win: any) => Promise<void>) {
  const win = await getWindow()
  if (!win) return
  await action(win)
}

async function refreshWindowState() {
  const win = await getWindow()
  if (!win) {
    isMaximized.value = false
    return
  }

  isMaximized.value = await win.isMaximized()
}

async function startDragging() {
  await withCurrentWindow(async (win) => {
    if (typeof win.startDragging === 'function') {
      await win.startDragging()
    }
  })
}

async function minimizeWindow() {
  await withCurrentWindow(async (win) => {
    await win.minimize()
  })
}

async function toggleMaximize() {
  await withCurrentWindow(async (win) => {
    await win.toggleMaximize()
    isMaximized.value = await win.isMaximized()
  })
}

async function closeWindow() {
  await withCurrentWindow(async (win) => {
    await win.close()
  })
}

function onHeaderPointerDown(event: PointerEvent) {
  if (event.button !== 0) return
  const target = event.target as HTMLElement | null
  if (!target || target.closest('[data-no-drag]')) return
  startDragging()
}

function onHeaderDoubleClick(event: MouseEvent) {
  const target = event.target as HTMLElement | null
  if (!target || target.closest('[data-no-drag]')) return
  toggleMaximize()
}

const unlistenResize = ref<null | (() => void)>(null)

onMounted(async () => {
  await refreshWindowState()

  const win = await getWindow()
  if (!win) return

  unlistenResize.value = await win.listen('tauri://resize', async () => {
    isMaximized.value = await win.isMaximized()
  })
})

onBeforeUnmount(() => {
  unlistenResize.value?.()
})
</script>

<template>
  <header
      class="relative z-50 isolate flex h-10 shrink-0 items-center overflow-visible border-b border-default bg-elevated pl-3 pr-1"
      @dblclick="onHeaderDoubleClick"
      @pointerdown="onHeaderPointerDown"
  >
    <div class="mr-3 flex min-w-0 items-center gap-2" data-window-drag-handle>
      <div
          class="flex size-5 items-center justify-center rounded bg-primary/15 text-[10px] font-bold text-primary ring-1 ring-inset ring-primary/20">
        GM
      </div>
      <span class="truncate text-sm font-semibold text-highlighted">{{ t('app.name') }}</span>
    </div>

    <div
        class="relative z-[60] shrink-0 text-sm font-normal [&_button]:font-normal [&_[role=menuitem]]:font-normal [&_[role=menu]]:z-[70] [&_a]:font-normal [&_span]:font-normal"
        data-no-drag
    >
      <UNavigationMenu
          :items="menuItems"
          :ui="{
            root: 'font-normal',
            list: 'gap-1',
            item: 'font-normal',
            link: 'font-normal',
            linkLabel: 'font-normal',
            content: 'z-[80] overflow-visible border border-default bg-default shadow-2xl ring-1 ring-default/50',
            childList: 'z-[80] rounded-xl bg-default',
            viewport: 'z-[80]'
          }"
          arrow
          class="shrink-0"
          color="neutral"
          content-orientation="vertical"
          disable-hover-trigger
          variant="link"
      />
    </div>

    <div class="min-w-0 flex-1" data-window-drag-handle/>

    <div class="flex items-center gap-1" data-no-drag>
      <UButton
          :aria-label="t('menu.preferences')"
          color="neutral"
          icon="i-lucide-settings-2"
          size="sm"
          square
          variant="ghost"
          @click="emit('openPreferences')"
      />
      <USeparator class="mx-1 h-5" orientation="vertical"/>
      <UButton
          :aria-label="t('window.minimize')"
          color="neutral"
          icon="i-lucide-minus"
          size="sm"
          square
          variant="ghost"
          @click="minimizeWindow"
      />
      <UButton
          :aria-label="t('window.maximize')"
          :icon="isMaximized ? 'i-lucide-copy' : 'i-lucide-square'"
          color="neutral"
          size="sm"
          square
          variant="ghost"
          @click="toggleMaximize"
      />
      <UButton
          :aria-label="t('window.close')"
          color="error"
          icon="i-lucide-x"
          size="sm"
          square
          variant="ghost"
          @click="closeWindow"
      />
    </div>
  </header>
</template>

<style scoped>
:deep([data-radix-popper-content-wrapper]) {
  z-index: 90 !important;
}
</style>
