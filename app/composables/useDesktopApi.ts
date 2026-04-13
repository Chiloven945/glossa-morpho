import type {
    BatchExportInput,
    BootstrapResponse,
    BulkReplaceInput,
    BulkReplaceResult,
    CommitImportInput,
    CreateEntryInput,
    CreateProjectInput,
    CreateResourceFileInput,
    DeleteEntriesInput,
    DeleteEntryInput,
    DeleteResourceFileInput,
    ExportProjectInput,
    ExportProjectResult,
    ImportPreviewResponse,
    PreviewImportInput,
    ProjectWorkspace,
    RenameResourceFileInput,
    TreemapNode,
    UpdateEntryInput,
    UpdateProjectMetadataInput
} from '#shared/types/models'
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

    saveProject(projectId: string): Promise<{ projectId: string; path: string; savedAt: string }> {
        return tryInvoke('save_project', {projectId}, () => mockDesktopApi.saveProject(projectId))
    },

    saveProjectAs(projectId: string, path: string): Promise<{ project: ProjectWorkspace; savedAt: string }> {
        return tryInvoke('save_project_as', {projectId, path}, () => mockDesktopApi.saveProjectAs(projectId, path))
    },

    updateProjectMetadata(input: UpdateProjectMetadataInput): Promise<ProjectWorkspace> {
        return tryInvoke('update_project_metadata', {input}, () => mockDesktopApi.updateProjectMetadata(input))
    },

    createResourceFile(input: CreateResourceFileInput): Promise<ProjectWorkspace> {
        return tryInvoke('create_resource_file', {input}, () => mockDesktopApi.createResourceFile(input))
    },

    renameResourceFile(input: RenameResourceFileInput): Promise<ProjectWorkspace> {
        return tryInvoke('rename_resource_file', {input}, () => mockDesktopApi.renameResourceFile(input))
    },

    deleteResourceFile(input: DeleteResourceFileInput): Promise<ProjectWorkspace> {
        return tryInvoke('delete_resource_file', {input}, () => mockDesktopApi.deleteResourceFile(input))
    },

    createEntry(input: CreateEntryInput): Promise<ProjectWorkspace> {
        return tryInvoke('create_entry', {input}, () => mockDesktopApi.createEntry(input))
    },

    deleteEntry(input: DeleteEntryInput): Promise<ProjectWorkspace> {
        return tryInvoke('delete_entry', {input}, () => mockDesktopApi.deleteEntry(input))
    },

    deleteEntries(input: DeleteEntriesInput): Promise<ProjectWorkspace> {
        return tryInvoke('delete_entries', {input}, () => mockDesktopApi.deleteEntries(input))
    },

    updateEntry(input: UpdateEntryInput): Promise<ProjectWorkspace> {
        return tryInvoke('update_entry', {input}, () => mockDesktopApi.updateEntry(input))
    },

    bulkReplace(input: BulkReplaceInput): Promise<BulkReplaceResult> {
        return tryInvoke('bulk_replace', {input}, () => mockDesktopApi.bulkReplace(input))
    },

    previewImport(input: PreviewImportInput): Promise<ImportPreviewResponse> {
        return tryInvoke('preview_import', {input}, () => mockDesktopApi.previewImport(input))
    },

    commitImport(input: CommitImportInput): Promise<ProjectWorkspace> {
        return tryInvoke('commit_import', {input}, () => mockDesktopApi.commitImport(input))
    },

    exportProject(input: ExportProjectInput): Promise<ExportProjectResult> {
        return tryInvoke('export_project', {input}, () => mockDesktopApi.exportProject(input))
    },

    exportProjectBatch(input: BatchExportInput): Promise<ExportProjectResult> {
        return tryInvoke('export_project_batch', {input}, () => mockDesktopApi.exportProjectBatch(input))
    },

    buildTreemap(projectId: string): Promise<TreemapNode[]> {
        return tryInvoke('build_treemap', {projectId}, () => mockDesktopApi.buildTreemap(projectId))
    }
}
