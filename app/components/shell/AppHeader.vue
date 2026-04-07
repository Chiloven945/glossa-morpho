<script setup lang="ts">
const {t, locale, locales, setLocale} = useI18n()

const props = defineProps<{
  projectName?: string
  isBusy?: boolean
}>()

const emit = defineEmits<{
  createProject: []
  openProject: []
  saveProject: []
  openCommands: []
}>()

const localeOptions = computed(() =>
    locales.value.map((item) => ({
      label: 'name' in item ? item.name : item.code,
      value: item.code
    }))
)

function switchLocale(value: string) {
  setLocale(value)
}
</script>

<template>
  <UCard>
    <div class="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
      <div class="space-y-2">
        <div class="flex flex-wrap items-center gap-2">
          <h1 class="text-xl font-semibold tracking-tight">
            {{ t('app.name') }}
          </h1>
          <UBadge color="neutral" variant="subtle">
            Tauri + Nuxt UI 4
          </UBadge>
          <UBadge v-if="projectName" color="primary" variant="soft">
            {{ projectName }}
          </UBadge>
          <UBadge v-if="isBusy" color="warning" variant="soft">
            {{ t('app.busy') }}
          </UBadge>
        </div>
        <p class="text-sm text-muted">
          {{ t('app.subtitle') }}
        </p>
      </div>

      <div class="flex flex-col gap-3 lg:items-end">
        <div class="flex flex-wrap items-center gap-2">
          <UButton icon="i-lucide-folder-plus" @click="emit('createProject')">
            {{ t('actions.newProject') }}
          </UButton>
          <UButton color="neutral" variant="outline" icon="i-lucide-folder-open" @click="emit('openProject')">
            {{ t('actions.openProject') }}
          </UButton>
          <UButton color="neutral" variant="outline" icon="i-lucide-save" @click="emit('saveProject')">
            {{ t('actions.save') }}
          </UButton>
          <UButton color="neutral" variant="soft" icon="i-lucide-command" @click="emit('openCommands')">
            {{ t('actions.commandPalette') }}
            <template #trailing>
              <UKbd value="Ctrl"/>
              <UKbd value="K"/>
            </template>
          </UButton>
        </div>

        <div class="flex items-center gap-3">
          <span class="text-sm text-muted">{{ t('labels.appLanguage') }}</span>
          <USelectMenu
              :model-value="locale"
              :items="localeOptions"
              value-key="value"
              class="w-48"
              @update:model-value="switchLocale"
          />
        </div>
      </div>
    </div>
  </UCard>
</template>
