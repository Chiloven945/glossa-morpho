export type TranslationStatus = 'new' | 'translated' | 'reviewed' | 'approved' | 'stale'
export type ViewMode = 'list' | 'treemap'

export interface ResourceFileNode {
    id: string
    name: string
    logicalPath: string
    format: 'json' | 'yaml' | 'properties' | 'resx' | 'xaml' | 'xliff'
    locale: string
    role: 'source' | 'target'
}

export interface CandidateItem {
    id: string
    source: 'history' | 'tm' | 'mt' | 'manual'
    value: string
    score: number
}

export interface HistoryEvent {
    id: string
    action: 'edit' | 'bulk_edit' | 'import_override' | 'revert'
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
    sourceLocale: string
    targetLocale: string
    targetLocales: string[]
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
    sourceLocale: string
    targetLocale: string
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
