import type {
    BootstrapResponse,
    BulkReplaceInput,
    BulkReplaceResult,
    CreateProjectInput,
    ProjectWorkspace,
    TreemapNode,
    UpdateEntryInput
} from '~/shared/types/models'
import {mockDesktopApi} from '~/utils/mock-desktop-api'

async function tryInvoke<T>(command: string, payload: Record<string, unknown>, fallback: () => Promise<T>): Promise<T> {
    try {
        const api = await import('@tauri-apps/api/core')
        return await api.invoke<T>(command, payload)
    } catch {
        return fallback()
    }
}

export const desktopApi = {
    bootstrap(): Promise<BootstrapResponse> {
        return tryInvoke('bootstrap_workspace', {}, () => mockDesktopApi.bootstrap())
    },

    createProject(input: CreateProjectInput): Promise<ProjectWorkspace> {
        return tryInvoke('create_project', {input}, () => mockDesktopApi.createProject(input))
    },

    openProject(path: string): Promise<ProjectWorkspace> {
        return tryInvoke('open_project', {path}, () => mockDesktopApi.openProject(path))
    },

    saveProject(projectId: string): Promise<{ projectId: string; savedAt: string }> {
        return tryInvoke('save_project', {projectId}, () => mockDesktopApi.saveProject(projectId))
    },

    updateEntry(input: UpdateEntryInput): Promise<ProjectWorkspace> {
        return tryInvoke('update_entry', {input}, () => mockDesktopApi.updateEntry(input))
    },

    bulkReplace(input: BulkReplaceInput): Promise<BulkReplaceResult> {
        return tryInvoke('bulk_replace', {input}, () => mockDesktopApi.bulkReplace(input))
    },

    buildTreemap(projectId: string): Promise<TreemapNode[]> {
        return tryInvoke('build_treemap', {projectId}, () => mockDesktopApi.buildTreemap(projectId))
    }
}
