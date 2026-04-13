import {defineStore} from 'pinia'
import type {
    BatchExportInput,
    BulkReplaceInput,
    CommitImportInput,
    CreateEntryInput,
    CreateProjectInput,
    CreateResourceFileInput,
    DeleteEntriesInput,
    DeleteEntryInput,
    DeleteResourceFileInput,
    ExportProjectInput,
    ExportProjectResult,
    ImportFileInput,
    ProjectWorkspace,
    RenameResourceFileInput,
    UpdateEntryInput,
    UpdateProjectMetadataInput
} from '#shared/types/models'
import {desktopApi} from '~/composables/useDesktopApi'

export const HOME_TAB_ID = 'home'

export const useWorkspaceStore = defineStore('workspace', {
    state: () => ({
        recentProjects: [] as string[],
        projects: [] as ProjectWorkspace[],
        activeTabId: HOME_TAB_ID as string,
        isCommandPaletteOpen: false,
        isBusy: false,
        lastSavedAt: ''
    }),

    getters: {
        activeProject(state): ProjectWorkspace | undefined {
            if (state.activeTabId === HOME_TAB_ID) return undefined
            return state.projects.find((item) => item.id === state.activeTabId)
        },
        activeProjectId(): string | null {
            return this.activeProject?.id ?? null
        }
    },

    actions: {
        async bootstrap() {
            this.isBusy = true
            try {
                const data = await desktopApi.bootstrap()
                this.recentProjects = data.recentProjects
                this.projects = data.openedProjects
                this.activeTabId = HOME_TAB_ID
            } finally {
                this.isBusy = false
            }
        },

        async createProject(input: CreateProjectInput) {
            const project = await desktopApi.createProject(input)
            this.upsertProject(project)
            this.pushRecentProject(project.path)
            this.activeTabId = project.id
            return project
        },

        async openProject(path: string) {
            const project = await desktopApi.openProject(path)
            this.upsertProject(project)
            this.pushRecentProject(project.path)
            this.activeTabId = project.id
            return project
        },

        async saveActiveProject() {
            if (!this.activeProjectId) return null
            const result = await desktopApi.saveProject(this.activeProjectId)
            this.lastSavedAt = result.savedAt
            const project = this.projects.find((item) => item.id === this.activeProjectId)
            if (project) {
                project.dirty = false
                project.path = result.path
                this.pushRecentProject(project.path)
            }
            return result
        },

        async saveActiveProjectAs(path: string) {
            if (!this.activeProjectId) return null
            const result = await desktopApi.saveProjectAs(this.activeProjectId, path)
            this.lastSavedAt = result.savedAt
            this.upsertProject(result.project)
            this.pushRecentProject(result.project.path)
            this.activeTabId = result.project.id
            return result
        },

        async updateProjectMetadata(input: UpdateProjectMetadataInput) {
            const project = await desktopApi.updateProjectMetadata(input)
            this.upsertProject(project)
            return project
        },

        async createResourceFile(input: CreateResourceFileInput) {
            const project = await desktopApi.createResourceFile(input)
            this.upsertProject(project)
            return project
        },

        async renameResourceFile(input: RenameResourceFileInput) {
            const project = await desktopApi.renameResourceFile(input)
            this.upsertProject(project)
            return project
        },

        async deleteResourceFile(input: DeleteResourceFileInput) {
            const project = await desktopApi.deleteResourceFile(input)
            this.upsertProject(project)
            return project
        },

        async createEntry(input: CreateEntryInput) {
            const project = await desktopApi.createEntry(input)
            this.upsertProject(project)
            return project
        },

        async deleteEntry(input: DeleteEntryInput) {
            const project = await desktopApi.deleteEntry(input)
            this.upsertProject(project)
            return project
        },

        async deleteEntries(input: DeleteEntriesInput) {
            const project = await desktopApi.deleteEntries(input)
            this.upsertProject(project)
            return project
        },

        async previewImport(projectId: string, files: ImportFileInput[]) {
            return desktopApi.previewImport({projectId, files})
        },

        async commitImport(input: CommitImportInput) {
            const project = await desktopApi.commitImport(input)
            this.upsertProject(project)
            this.activeTabId = project.id
            return project
        },

        async exportProject(input: ExportProjectInput): Promise<ExportProjectResult> {
            return desktopApi.exportProject(input)
        },

        async exportProjectBatch(input: BatchExportInput): Promise<ExportProjectResult> {
            return desktopApi.exportProjectBatch(input)
        },

        setActiveTab(tabId: string) {
            this.activeTabId = tabId
        },

        openHome() {
            this.activeTabId = HOME_TAB_ID
        },

        reorderProjectTabs(projectId: string, targetProjectId: string) {
            if (projectId === targetProjectId) return

            const ordered = [...this.projects]
            const fromIndex = ordered.findIndex((item) => item.id === projectId)
            const targetIndex = ordered.findIndex((item) => item.id === targetProjectId)
            if (fromIndex < 0 || targetIndex < 0) return

            const [moved] = ordered.splice(fromIndex, 1)
            ordered.splice(targetIndex, 0, moved)
            this.projects = ordered
        },

        closeProject(projectId: string) {
            const currentIndex = this.projects.findIndex((item) => item.id === projectId)
            const nextProjects = this.projects.filter((item) => item.id !== projectId)

            if (this.activeTabId === projectId) {
                const fallbackIndex = Math.max(0, currentIndex - 1)
                this.activeTabId = nextProjects[fallbackIndex]?.id ?? HOME_TAB_ID
            }

            this.projects = nextProjects
        },

        toggleCommandPalette(next?: boolean) {
            this.isCommandPaletteOpen = typeof next === 'boolean' ? next : !this.isCommandPaletteOpen
        },

        async updateEntry(input: UpdateEntryInput) {
            const project = await desktopApi.updateEntry(input)
            this.upsertProject(project)
            return project
        },

        async bulkReplace(input: BulkReplaceInput) {
            const result = await desktopApi.bulkReplace(input)
            this.upsertProject(result.project)
            return result
        },

        upsertProject(project: ProjectWorkspace) {
            const index = this.projects.findIndex((item) => item.id === project.id)
            if (index === -1) {
                this.projects = [...this.projects, project]
                return
            }

            const next = [...this.projects]
            next.splice(index, 1, project)
            this.projects = next
        },

        pushRecentProject(path: string) {
            this.recentProjects = [path, ...this.recentProjects.filter((item) => item !== path)].slice(0, 12)
        }
    }
})
