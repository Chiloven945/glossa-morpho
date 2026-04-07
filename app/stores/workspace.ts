import {defineStore} from 'pinia'
import type {BulkReplaceInput, CreateProjectInput, ProjectWorkspace, UpdateEntryInput} from '~/shared/types/models'
import {desktopApi} from '~/composables/useDesktopApi'

export const useWorkspaceStore = defineStore('workspace', {
    state: () => ({
        recentProjects: [] as string[],
        projects: [] as ProjectWorkspace[],
        activeProjectId: null as string | null,
        isCommandPaletteOpen: false,
        isBusy: false,
        lastSavedAt: ''
    }),

    getters: {
        activeProject(state): ProjectWorkspace | undefined {
            return state.projects.find((item) => item.id === state.activeProjectId)
        }
    },

    actions: {
        async bootstrap() {
            this.isBusy = true
            try {
                const data = await desktopApi.bootstrap()
                this.recentProjects = data.recentProjects
                this.projects = data.openedProjects
                this.activeProjectId = data.openedProjects[0]?.id ?? null
            } finally {
                this.isBusy = false
            }
        },

        async createProject(input: CreateProjectInput) {
            const project = await desktopApi.createProject(input)
            this.upsertProject(project)
            this.activeProjectId = project.id
        },

        async openProject(path = '/Users/you/projects/demo.gmproj') {
            const project = await desktopApi.openProject(path)
            this.upsertProject(project)
            this.activeProjectId = project.id
        },

        async saveActiveProject() {
            if (!this.activeProjectId) return
            const result = await desktopApi.saveProject(this.activeProjectId)
            this.lastSavedAt = result.savedAt
            const project = this.projects.find((item) => item.id === this.activeProjectId)
            if (project) project.dirty = false
        },

        setActiveProject(projectId: string) {
            this.activeProjectId = projectId
        },

        toggleCommandPalette(next?: boolean) {
            this.isCommandPaletteOpen = typeof next === 'boolean' ? next : !this.isCommandPaletteOpen
        },

        async updateEntry(input: UpdateEntryInput) {
            const project = await desktopApi.updateEntry(input)
            this.upsertProject(project)
        },

        async bulkReplace(input: BulkReplaceInput) {
            const result = await desktopApi.bulkReplace(input)
            this.upsertProject(result.project)
            return result
        },

        upsertProject(project: ProjectWorkspace) {
            this.projects = this.projects.filter((item) => item.id !== project.id).concat(project)
        }
    }
})
