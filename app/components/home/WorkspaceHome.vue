<script lang="ts" setup>
const props = defineProps<{
  recentProjects: string[]
}>()

const emit = defineEmits<{
  createProject: []
  openProject: [path?: string]
}>()
</script>

<template>
  <div class="flex h-full min-h-0 flex-col overflow-auto bg-default">
    <div class="grid gap-6 p-6 xl:grid-cols-[minmax(0,0.9fr)_minmax(0,1.1fr)]">
      <UCard>
        <template #header>
          <div class="space-y-1">
            <p class="text-xs uppercase tracking-[0.2em] text-muted">Workspace</p>
            <h2 class="text-xl font-semibold">{{ $t('home.title') }}</h2>
            <p class="text-sm text-muted">{{ $t('home.description') }}</p>
          </div>
        </template>

        <div class="grid gap-4 md:grid-cols-2">
          <UCard variant="subtle">
            <div class="space-y-4">
              <div>
                <p class="font-medium">{{ $t('actions.newProject') }}</p>
                <p class="text-sm text-muted">{{ $t('home.newProjectDescription') }}</p>
              </div>
              <UButton icon="i-lucide-folder-plus" @click="emit('createProject')">
                {{ $t('actions.newProject') }}
              </UButton>
            </div>
          </UCard>

          <UCard variant="subtle">
            <div class="space-y-4">
              <div>
                <p class="font-medium">{{ $t('actions.openProject') }}</p>
                <p class="text-sm text-muted">{{ $t('home.openProjectDescription') }}</p>
              </div>
              <UButton color="neutral" icon="i-lucide-folder-open" variant="outline" @click="emit('openProject')">
                {{ $t('actions.openProject') }}
              </UButton>
            </div>
          </UCard>
        </div>
      </UCard>

      <UCard>
        <template #header>
          <div class="space-y-1">
            <h2 class="font-semibold">{{ $t('labels.recentProjects') }}</h2>
            <p class="text-sm text-muted">{{ $t('home.recentDescription') }}</p>
          </div>
        </template>

        <div v-if="recentProjects.length" class="space-y-2">
          <UButton
              v-for="path in recentProjects"
              :key="path"
              class="w-full justify-start"
              color="neutral"
              variant="soft"
              @click="emit('openProject', path)"
          >
            <span class="truncate">{{ path }}</span>
          </UButton>
        </div>
        <div v-else class="rounded-xl border border-dashed border-default px-4 py-8 text-sm text-muted">
          {{ $t('home.noRecentProjects') }}
        </div>
      </UCard>
    </div>
  </div>
</template>
