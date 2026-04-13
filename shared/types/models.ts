export type TranslationStatus = 'new' | 'translated' | 'reviewed' | 'approved' | 'stale'
export type ViewMode = 'list' | 'treemap'
export type ResourceFormat = 'json' | 'yaml' | 'properties' | 'resx' | 'xaml' | 'xliff'
export type ProjectCompressionType = 'lzma2' | 'deflate'

export interface LocaleDependencyNode {
    code: string
    label: string
    parentCode: string | null
}

export interface ResourceFileNode {
    id: string
    name: string
    logicalPath: string
    format: ResourceFormat
    locale: string
    basedOnLocale: string | null
    rawRelativePath?: string | null
}

export interface CandidateItem {
    id: string
    source: 'history' | 'tm' | 'mt' | 'manual'
    value: string
    score: number
}

export interface HistoryEvent {
    id: string
    action: 'edit' | 'bulk_edit' | 'import_override' | 'revert' | 'bulk_edit_regex' | 'create' | 'delete'
    beforeValue: string
    afterValue: string
    operator: string
    createdAt: string
}

export interface ValidationIssue {
    id: string
    level: 'info' | 'warning' | 'error'
    message: string
}

export interface EntrySummary {
    id: string
    fileId: string
    key: string
    sourceValue: string
    targetValue: string
    status: TranslationStatus
    noteCount: number
    candidateCount: number
    updatedAt: string
}

export interface EntryDetail extends EntrySummary {
    filePath: string
    sourceLocale: string
    targetLocale: string
    note: string
    issues: ValidationIssue[]
    candidates: CandidateItem[]
    history: HistoryEvent[]
}

export interface ProjectStats {
    total: number
    translated: number
    missing: number
    reviewed: number
}

export interface TreemapNode {
    id: string
    label: string
    path: string
    count: number
    translatedCount: number
    missingCount: number
    charCount: number
}

export interface ProjectWorkspace {
    id: string
    name: string
    path: string
    workspaceDir?: string
    localeGraph: LocaleDependencyNode[]
    primaryLocale: string
    workingLocale: string
    archiveFormat: ProjectCompressionType
    keySegmentationProfiles?: string[]
    defaultView?: ViewMode
    defaultSort?: 'updatedDesc' | 'keyAsc' | 'status'
    dirty: boolean
    files: ResourceFileNode[]
    entries: EntrySummary[]
    details: Record<string, EntryDetail>
    treemap: TreemapNode[]
    stats: ProjectStats
}

export interface BootstrapResponse {
    recentProjects: string[]
    openedProjects: ProjectWorkspace[]
}

export interface CreateProjectInput {
    name: string
    path?: string
    localeGraph: LocaleDependencyNode[]
    primaryLocale: string
    workingLocale: string
    archiveFormat?: ProjectCompressionType
    keySegmentationProfiles?: string[]
}

export interface UpdateProjectMetadataInput {
    projectId: string
    name: string
    primaryLocale: string
    workingLocale: string
    archiveFormat: ProjectCompressionType
    keySegmentationProfiles: string[]
    defaultView: ViewMode
    defaultSort: 'updatedDesc' | 'keyAsc' | 'status'
}


export interface CreateResourceFileInput {
    projectId: string
    name: string
    logicalPath: string
    format: ResourceFormat
    locale: string
    basedOnLocale?: string | null
    includeDescendants?: boolean
}

export interface RenameResourceFileInput {
    projectId: string
    fileId: string
    name: string
    logicalPath: string
    includeRelated?: boolean
}

export interface DeleteResourceFileInput {
    projectId: string
    fileId: string
    includeRelated?: boolean
}

export interface CreateEntryInput {
    projectId: string
    fileId: string
    key: string
    sourceValue?: string
    targetValue?: string
    note?: string
    status?: TranslationStatus
}

export interface DeleteEntryInput {
    projectId: string
    entryId: string
}

export interface DeleteEntriesInput {
    projectId: string
    entryIds: string[]
}

export interface UpdateEntryInput {
    projectId: string
    entryId: string
    targetValue?: string
    note?: string
    status?: TranslationStatus
}

export interface BulkReplaceInput {
    projectId: string
    search: string
    replacement: string
    useRegex: boolean
    targetScope: 'targetOnly' | 'sourceAndTarget'
}

export interface BulkReplaceResult {
    changedEntryIds: string[]
    project: ProjectWorkspace
}

export interface ImportFileInput {
    path: string
    locale: string
    basedOnLocale?: string | null
    logicalPath?: string
}

export interface ImportPreviewItem {
    previewFileId: string
    path: string
    logicalPath: string
    name: string
    format: ResourceFormat
    locale: string
    basedOnLocale: string | null
    entryCount: number
    conflictCount: number
}

export interface ImportConflict {
    kind: 'file' | 'entry'
    logicalPath: string
    locale: string
    key?: string
    existingValue?: string
    incomingValue?: string
    message: string
}

export interface ImportPreviewResponse {
    previewId: string
    items: ImportPreviewItem[]
    entries: EntrySummary[]
    conflicts: ImportConflict[]
    totals: {
        files: number
        entries: number
        conflicts: number
    }
}

export interface PreviewImportInput {
    projectId: string
    files: ImportFileInput[]
}

export interface CommitImportInput {
    projectId: string
    previewId: string
}

export interface ExportProjectInput {
    projectId: string
    fileId?: string
    outputPath?: string
}

export interface BatchExportInput {
    projectId: string
    fileIds: string[]
    outputDirectory?: string
}

export interface ExportProjectResult {
    projectId: string
    outputPath: string
    exportedFiles: string[]
}
