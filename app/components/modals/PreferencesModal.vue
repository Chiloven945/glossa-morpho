<script lang="ts" setup>
import type {TreeItem} from '@nuxt/ui'
import {APP_LOCALES} from '#shared/constants/locales'

const props = defineProps<{
  open: boolean
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const {t, locale, setLocale} = useI18n()
const colorMode = useColorMode()

const settingsSection = ref<'application-language' | 'appearance-color-mode'>('application-language')

const localeOptions = APP_LOCALES.map((item) => ({
  label: `${item.code} · ${item.label}`,
  value: item.code
}))

const settingsTreeItems = computed<TreeItem[]>(() => [
  {
    label: t('settings.groupGeneral'),
    icon: 'i-lucide-app-window',
    defaultExpanded: true,
    children: [
      {
        label: t('settings.languageTitle'),
        icon: 'i-lucide-languages',
        class: settingsSection.value === 'application-language' ? 'text-primary' : undefined,
        onSelect: () => {
          settingsSection.value = 'application-language'
        }
      }
    ]
  },
  {
    label: t('settings.groupAppearance'),
    icon: 'i-lucide-palette',
    defaultExpanded: true,
    children: [
      {
        label: t('settings.colorMode'),
        icon: 'i-lucide-sun-moon',
        class: settingsSection.value === 'appearance-color-mode' ? 'text-primary' : undefined,
        onSelect: () => {
          settingsSection.value = 'appearance-color-mode'
        }
      }
    ]
  }
])

const colorModeOptions = computed(() => [
  {label: t('settings.followSystem'), value: 'system'},
  {label: t('settings.lightMode'), value: 'light'},
  {label: t('settings.darkMode'), value: 'dark'}
])
</script>

<template>
  <UModal
      :open="open"
      :title="$t('settings.title')"
      :ui="{ content: 'sm:max-w-5xl', body: 'min-h-0 max-h-[78vh] overflow-hidden' }"
      @update:open="emit('update:open', $event)"
  >
    <template #body>
      <div class="grid min-h-[34rem] grid-cols-[18rem_minmax(0,1fr)] gap-6">
        <div class="min-h-0 overflow-auto rounded-lg border border-default bg-elevated/40 p-3">
          <UTree :items="settingsTreeItems" color="neutral"/>
        </div>

        <div class="min-h-0 overflow-auto space-y-4 pr-1">
          <div>
            <h3 class="text-base font-semibold">
              {{ settingsSection === 'application-language' ? $t('settings.languageTitle') : $t('settings.colorMode') }}
            </h3>
            <p class="text-sm text-muted">
              {{
                settingsSection === 'application-language' ? $t('settings.languageDescription') : $t('settings.appearanceDescription')
              }}
            </p>
          </div>

          <UCard v-if="settingsSection === 'application-language'">
            <UForm :state="{ locale }" class="space-y-4">
              <UFormField :label="$t('labels.appLanguage') as string">
                <USelectMenu
                    :items="localeOptions"
                    :model-value="locale"
                    value-key="value"
                    @update:model-value="value => setLocale(String(value))"
                />
              </UFormField>
            </UForm>
          </UCard>

          <UCard v-else>
            <UForm :state="{ color: colorMode.preference }" class="space-y-4">
              <UFormField :label="$t('settings.colorMode') as string">
                <USelectMenu
                    :items="colorModeOptions"
                    :model-value="colorMode.preference"
                    value-key="value"
                    @update:model-value="value => colorMode.preference = String(value)"
                />
              </UFormField>
            </UForm>
          </UCard>
        </div>
      </div>
    </template>
  </UModal>
</template>
