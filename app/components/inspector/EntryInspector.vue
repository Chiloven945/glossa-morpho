<script setup lang="ts">
import type {TabsItem} from '@nuxt/ui'
import type {EntryDetail, TranslationStatus} from '~/shared/types/models'

const props = defineProps<{
  entry?: EntryDetail
}>()

const emit = defineEmits<{
  save: [payload: { targetValue: string; note: string; status: TranslationStatus }]
}>()

const targetValue = ref('')
const note = ref('')
const status = ref<TranslationStatus>('new')
const activeTab = ref('edit')

const statusOptions = [
  {label: 'new', value: 'new'},
  {label: 'translated', value: 'translated'},
  {label: 'reviewed', value: 'reviewed'},
  {label: 'approved', value: 'approved'},
  {label: 'stale', value: 'stale'}
]

const tabs = computed<TabsItem[]>(() => [
  {label: 'Edit', value: 'edit', icon: 'i-lucide-pencil-line'},
  {label: 'History', value: 'history', icon: 'i-lucide-history'},
  {label: 'Candidates', value: 'candidates', icon: 'i-lucide-sparkles'},
  {label: 'Validation', value: 'validation', icon: 'i-lucide-shield-check'}
])

watch(
    () => props.entry,
    (entry) => {
      targetValue.value = entry?.targetValue ?? ''
      note.value = entry?.note ?? ''
      status.value = entry?.status ?? 'new'
      activeTab.value = 'edit'
    },
    {immediate: true}
)

function save() {
  emit('save', {
    targetValue: targetValue.value,
    note: note.value,
    status: status.value
  })
}

function applyCandidate(value: string) {
  targetValue.value = value
  activeTab.value = 'edit'
}
</script>

<template>
  <UCard class="h-full">
    <template #header>
      <div class="space-y-3">
        <div class="flex items-start justify-between gap-3">
          <div>
            <h2 class="font-semibold">{{ $t('labels.inspector') }}</h2>
            <p class="text-sm text-muted">
              {{ entry?.key || $t('empty.noSelectionTitle') }}
            </p>
          </div>
          <UBadge v-if="entry" color="primary" variant="soft">
            {{ entry.status }}
          </UBadge>
        </div>

        <div v-if="entry" class="grid grid-cols-2 gap-3">
          <div>
            <p class="text-xs text-muted">{{ $t('labels.sourceLocale') }}</p>
            <p class="font-medium">{{ entry.sourceLocale }}</p>
          </div>
          <div>
            <p class="text-xs text-muted">{{ $t('labels.targetLocale') }}</p>
            <p class="font-medium">{{ entry.targetLocale }}</p>
          </div>
        </div>
      </div>
    </template>

    <div v-if="entry" class="space-y-4">
      <UTabs v-model="activeTab" :items="tabs" :content="false" variant="link"/>

      <div v-if="activeTab === 'edit'" class="space-y-4">
        <UForm :state="{ targetValue, note, status }" class="space-y-4">
          <UFormField :label="$t('labels.source') as string" name="sourceValue">
            <UTextarea :model-value="entry.sourceValue" :rows="4" disabled class="w-full"/>
          </UFormField>

          <UFormField :label="$t('labels.target') as string" name="targetValue">
            <UTextarea v-model="targetValue" :rows="6" class="w-full"/>
          </UFormField>

          <UFormField :label="$t('labels.note') as string" name="note">
            <UTextarea v-model="note" :rows="4" class="w-full"/>
          </UFormField>

          <div class="grid gap-4 md:grid-cols-[minmax(0,1fr)_auto]">
            <UFormField :label="$t('labels.status') as string" name="status">
              <USelectMenu v-model="status" :items="statusOptions" value-key="value" class="w-full"/>
            </UFormField>

            <div class="flex items-end justify-end">
              <UButton icon="i-lucide-save" @click="save">
                {{ $t('actions.saveEntry') }}
              </UButton>
            </div>
          </div>
        </UForm>
      </div>

      <div v-else-if="activeTab === 'history'" class="space-y-3">
        <UCard v-for="item in entry.history" :key="item.id">
          <div class="space-y-3">
            <div class="flex items-center justify-between gap-3">
              <UBadge color="neutral" variant="subtle">{{ item.action }}</UBadge>
              <span class="text-xs text-muted">{{ new Date(item.createdAt).toLocaleString() }}</span>
            </div>
            <p class="text-sm text-muted">{{ item.operator }}</p>
            <div class="grid gap-3">
              <div>
                <p class="text-xs text-muted">Before</p>
                <p class="text-sm">{{ item.beforeValue || '∅' }}</p>
              </div>
              <div>
                <p class="text-xs text-muted">After</p>
                <p class="text-sm">{{ item.afterValue || '∅' }}</p>
              </div>
            </div>
          </div>
        </UCard>
        <p v-if="entry.history.length === 0" class="text-sm text-muted">{{ $t('empty.noHistory') }}</p>
      </div>

      <div v-else-if="activeTab === 'candidates'" class="space-y-3">
        <UCard v-for="candidate in entry.candidates" :key="candidate.id">
          <div class="space-y-3">
            <div class="flex items-center justify-between gap-3">
              <UBadge color="primary" variant="subtle">{{ candidate.source }}</UBadge>
              <span class="text-xs text-muted">{{ Math.round(candidate.score * 100) }}%</span>
            </div>
            <p class="text-sm">{{ candidate.value }}</p>
            <div class="flex justify-end">
              <UButton color="neutral" variant="outline" size="sm" @click="applyCandidate(candidate.value)">
                {{ $t('actions.useCandidate') }}
              </UButton>
            </div>
          </div>
        </UCard>
        <p v-if="entry.candidates.length === 0" class="text-sm text-muted">{{ $t('empty.noCandidates') }}</p>
      </div>

      <div v-else class="space-y-3">
        <UCard v-for="issue in entry.issues" :key="issue.id">
          <div class="flex items-start gap-3">
            <UBadge :color="issue.level === 'error' ? 'error' : issue.level === 'warning' ? 'warning' : 'info'"
                    variant="subtle">
              {{ issue.level }}
            </UBadge>
            <p class="text-sm">{{ issue.message }}</p>
          </div>
        </UCard>
        <p v-if="entry.issues.length === 0" class="text-sm text-muted">{{ $t('empty.noIssues') }}</p>
      </div>
    </div>

    <div v-else class="flex min-h-[24rem] items-center justify-center">
      <div class="space-y-2 text-center">
        <p class="text-base font-medium">{{ $t('empty.noSelectionTitle') }}</p>
        <p class="text-sm text-muted">{{ $t('empty.noSelectionDescription') }}</p>
      </div>
    </div>
  </UCard>
</template>
