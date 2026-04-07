import type {
    BootstrapResponse,
    BulkReplaceInput,
    BulkReplaceResult,
    CreateProjectInput,
    EntryDetail,
    EntrySummary,
    ProjectWorkspace,
    ResourceFileNode,
    TranslationStatus,
    TreemapNode,
    UpdateEntryInput
} from '~/shared/types/models'

const now = () => new Date().toISOString()

function makeFile(id: string, name: string, logicalPath: string, format: ResourceFileNode['format'], locale: string, role: ResourceFileNode['role']): ResourceFileNode {
    return {id, name, logicalPath, format, locale, role}
}

function makeEntry(id: string, fileId: string, key: string, sourceValue: string, targetValue: string, status: TranslationStatus): EntryDetail {
    return {
        id,
        fileId,
        key,
        filePath: fileId.includes('yaml') ? 'locale/app.yml' : 'locale/app.json',
        sourceLocale: 'en-US',
        targetLocale: 'zh-CN',
        sourceValue,
        targetValue,
        status,
        noteCount: targetValue ? 1 : 0,
        candidateCount: 2,
        updatedAt: now(),
        note: status === 'new' ? '' : 'Check punctuation and placeholders before review.',
        issues: targetValue.includes('{name}')
            ? []
            : [{id: `${id}-issue`, level: 'warning', message: 'Placeholder consistency not checked yet.'}],
        candidates: [
            {id: `${id}-c1`, source: 'history', value: targetValue || '欢迎回来，{name}', score: 0.92},
            {id: `${id}-c2`, source: 'manual', value: targetValue || '你好，{name}', score: 0.61}
        ],
        history: [
            {
                id: `${id}-h1`,
                action: 'edit',
                beforeValue: '',
                afterValue: targetValue,
                operator: 'system',
                createdAt: now()
            }
        ]
    }
}

function buildTreemap(entries: EntrySummary[]): TreemapNode[] {
    const map = new Map<string, TreemapNode>()

    for (const entry of entries) {
        const [group = 'root'] = entry.key.split('.')
        const current = map.get(group) ?? {
            id: group,
            label: group,
            path: group,
            count: 0,
            translatedCount: 0,
            missingCount: 0,
            charCount: 0
        }

        current.count += 1
        current.charCount += entry.sourceValue.length + entry.targetValue.length
        if (entry.targetValue) current.translatedCount += 1
        if (!entry.targetValue) current.missingCount += 1

        map.set(group, current)
    }

    return [...map.values()].sort((a, b) => b.count - a.count)
}

function makeProject(input?: Partial<CreateProjectInput> & { name?: string; path?: string }): ProjectWorkspace {
    const files = [
        makeFile('file-json-en', 'app.en-US.json', 'locale/source/app.en-US.json', 'json', 'en-US', 'source'),
        makeFile('file-json-zh', 'app.zh-CN.json', 'locale/target/app.zh-CN.json', 'json', 'zh-CN', 'target'),
        makeFile('file-yaml-en', 'marketing.en-US.yaml', 'locale/source/marketing.en-US.yaml', 'yaml', 'en-US', 'source'),
        makeFile('file-yaml-zh', 'marketing.zh-CN.yaml', 'locale/target/marketing.zh-CN.yaml', 'yaml', 'zh-CN', 'target')
    ]

    const details = [
        makeEntry('entry-1', 'file-json-zh', 'dashboard.welcomeBack', 'Welcome back, {name}', '欢迎回来，{name}', 'translated'),
        makeEntry('entry-2', 'file-json-zh', 'dashboard.emptyState.title', 'No records found', '', 'new'),
        makeEntry('entry-3', 'file-yaml-zh', 'marketing.heroSubtitle', 'Ship localization projects faster', '更快交付本地化项目', 'reviewed'),
        makeEntry('entry-4', 'file-yaml-zh', 'settings.autoSaveInterval', 'Auto save interval', '自动保存间隔', 'translated'),
        makeEntry('entry-5', 'file-json-zh', 'errors.networkTimeout', 'Network request timed out', '', 'new'),
        makeEntry('entry-6', 'file-json-zh', 'common.ok', 'OK', '确定', 'approved')
    ]

    const entryMap = Object.fromEntries(details.map((item) => [item.id, item]))
    const entries: EntrySummary[] = details.map(({
                                                     history,
                                                     candidates,
                                                     issues,
                                                     note,
                                                     filePath,
                                                     sourceLocale,
                                                     targetLocale,
                                                     ...rest
                                                 }) => rest)
    const treemap = buildTreemap(entries)
    const total = entries.length
    const translated = entries.filter((entry) => Boolean(entry.targetValue)).length
    const reviewed = entries.filter((entry) => entry.status === 'reviewed' || entry.status === 'approved').length

    return {
        id: `project-${Math.random().toString(36).slice(2, 10)}`,
        name: input?.name ?? 'Demo Localization Project',
        path: input?.path ?? '/Users/you/projects/demo.gmproj',
        sourceLocale: input?.sourceLocale ?? 'en-US',
        targetLocale: input?.targetLocale ?? 'zh-CN',
        targetLocales: [input?.targetLocale ?? 'zh-CN', 'ja-JP'],
        dirty: false,
        files,
        entries,
        details: entryMap,
        treemap,
        stats: {
            total,
            translated,
            missing: total - translated,
            reviewed
        }
    }
}

const mockState: BootstrapResponse = {
    recentProjects: ['/Users/you/projects/demo.gmproj'],
    openedProjects: []
}

function replaceProject(project: ProjectWorkspace) {
    mockState.openedProjects = mockState.openedProjects
        .filter((item) => item.id !== project.id)
        .concat(project)
}

export const mockDesktopApi = {
    async bootstrap(): Promise<BootstrapResponse> {
        return structuredClone(mockState)
    },

    async createProject(input: CreateProjectInput): Promise<ProjectWorkspace> {
        const project = makeProject(input)
        replaceProject(project)
        return structuredClone(project)
    },

    async openProject(path: string): Promise<ProjectWorkspace> {
        const project = makeProject({path, name: 'Opened Demo Project'})
        replaceProject(project)
        return structuredClone(project)
    },

    async saveProject(projectId: string): Promise<{ projectId: string; savedAt: string }> {
        return {projectId, savedAt: now()}
    },

    async updateEntry(input: UpdateEntryInput): Promise<ProjectWorkspace> {
        const project = mockState.openedProjects.find((item) => item.id === input.projectId)
        if (!project) {
            throw new Error('Project not found in mock state')
        }

        const detail = project.details[input.entryId]
        if (!detail) {
            throw new Error('Entry not found in mock state')
        }

        if (typeof input.targetValue === 'string') detail.targetValue = input.targetValue
        if (typeof input.note === 'string') detail.note = input.note
        if (input.status) detail.status = input.status
        detail.updatedAt = now()
        detail.history.unshift({
            id: `${detail.id}-${detail.history.length + 1}`,
            action: 'edit',
            beforeValue: detail.history[0]?.afterValue ?? '',
            afterValue: detail.targetValue,
            operator: 'mock-user',
            createdAt: detail.updatedAt
        })

        project.entries = project.entries.map((entry) =>
            entry.id === detail.id
                ? {
                    ...entry,
                    targetValue: detail.targetValue,
                    status: detail.status,
                    updatedAt: detail.updatedAt,
                    noteCount: detail.note ? 1 : 0
                }
                : entry
        )

        project.dirty = true
        project.treemap = buildTreemap(project.entries)
        project.stats = {
            total: project.entries.length,
            translated: project.entries.filter((entry) => Boolean(entry.targetValue)).length,
            missing: project.entries.filter((entry) => !entry.targetValue).length,
            reviewed: project.entries.filter((entry) => entry.status === 'reviewed' || entry.status === 'approved').length
        }

        replaceProject(project)
        return structuredClone(project)
    },

    async bulkReplace(input: BulkReplaceInput): Promise<BulkReplaceResult> {
        const project = mockState.openedProjects.find((item) => item.id === input.projectId)
        if (!project) {
            throw new Error('Project not found in mock state')
        }

        const matcher = input.useRegex ? new RegExp(input.search, 'g') : null
        const changedEntryIds: string[] = []

        for (const entry of Object.values(project.details)) {
            const original = entry.targetValue
            const next = input.useRegex
                ? original.replace(matcher!, input.replacement)
                : original.split(input.search).join(input.replacement)

            if (next !== original) {
                entry.targetValue = next
                entry.updatedAt = now()
                entry.status = next ? 'translated' : 'new'
                entry.history.unshift({
                    id: `${entry.id}-${entry.history.length + 1}`,
                    action: 'bulk_edit',
                    beforeValue: original,
                    afterValue: next,
                    operator: 'bulk-replace',
                    createdAt: entry.updatedAt
                })
                changedEntryIds.push(entry.id)
            }
        }

        project.entries = project.entries.map((entry) => {
            const detail = project.details[entry.id]
            return {
                ...entry,
                targetValue: detail.targetValue,
                status: detail.status,
                updatedAt: detail.updatedAt
            }
        })
        project.dirty = true
        project.treemap = buildTreemap(project.entries)
        project.stats = {
            total: project.entries.length,
            translated: project.entries.filter((entry) => Boolean(entry.targetValue)).length,
            missing: project.entries.filter((entry) => !entry.targetValue).length,
            reviewed: project.entries.filter((entry) => entry.status === 'reviewed' || entry.status === 'approved').length
        }

        replaceProject(project)
        return {changedEntryIds, project: structuredClone(project)}
    },

    async buildTreemap(projectId: string): Promise<TreemapNode[]> {
        const project = mockState.openedProjects.find((item) => item.id === projectId)
        return structuredClone(project?.treemap ?? [])
    }
}
