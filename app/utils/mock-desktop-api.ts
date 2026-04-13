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
    EntryDetail,
    EntrySummary,
    ExportProjectInput,
    ExportProjectResult,
    ImportPreviewResponse,
    LocaleDependencyNode,
    PreviewImportInput,
    ProjectWorkspace,
    RenameResourceFileInput,
    ResourceFileNode,
    TranslationStatus,
    TreemapNode,
    UpdateEntryInput,
    UpdateProjectMetadataInput,
    ValidationIssue
} from '#shared/types/models'

const now = () => new Date().toISOString()

const localeLabels = new Map([
    ['en-US', 'English'],
    ['zh-CN', '简体中文'],
    ['ja-JP', '日本語'],
    ['fr-FR', 'Français']
])

const importPreviews = new Map<string, ImportPreviewResponse>()
const importPreviewFiles = new Map<string, PreviewImportInput['files']>()

function defaultProjectPath(name = 'Demo Localization Project') {
    const slug = name.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-+|-+$/g, '') || 'glossa-project'
    return `/Users/you/projects/${slug}.gmproj`
}

function pushRecentProject(path: string) {
    mockState.recentProjects = [path, ...mockState.recentProjects.filter((item) => item !== path)].slice(0, 12)
}

function makeFile(id: string, name: string, logicalPath: string, format: ResourceFileNode['format'], locale: string, basedOnLocale: string | null): ResourceFileNode {
    return {id, name, logicalPath, format, locale, basedOnLocale, rawRelativePath: null}
}

function buildIssues(sourceValue: string, targetValue: string): ValidationIssue[] {
    const issues: ValidationIssue[] = []
    const placeholderRegex = /\{[A-Za-z0-9_.-]+\}|%(?:\d+\$)?[sdfoxeguc]/g
    const sourcePlaceholders = Array.from(sourceValue.matchAll(placeholderRegex)).map((item) => item[0]).sort()
    const targetPlaceholders = Array.from(targetValue.matchAll(placeholderRegex)).map((item) => item[0]).sort()
    if (!targetValue) {
        issues.push({id: crypto.randomUUID(), level: 'info', message: 'Translation is empty and still needs a value.'})
        return issues
    }
    if (JSON.stringify(sourcePlaceholders) !== JSON.stringify(targetPlaceholders)) {
        issues.push({id: crypto.randomUUID(), level: 'error', message: 'Placeholder mismatch detected.'})
    }
    return issues
}

function makeEntry(
    id: string,
    fileId: string,
    key: string,
    sourceValue: string,
    targetValue: string,
    status: TranslationStatus,
    sourceLocale: string,
    targetLocale: string,
    filePath: string
): EntryDetail {
    const issues = buildIssues(sourceValue, targetValue)
    return {
        id,
        fileId,
        key,
        filePath,
        sourceLocale,
        targetLocale,
        sourceValue,
        targetValue,
        status,
        noteCount: targetValue ? 1 : 0,
        candidateCount: 2,
        updatedAt: now(),
        note: status === 'new' ? '' : 'Check punctuation and placeholders before review.',
        issues,
        candidates: [
            {id: `${id}-c1`, source: 'history', value: targetValue || '欢迎回来，{name}', score: 0.92},
            {id: `${id}-c2`, source: 'manual', value: targetValue || '你好，{name}', score: 0.61}
        ],
        history: [{
            id: `${id}-h1`,
            action: 'edit',
            beforeValue: '',
            afterValue: targetValue,
            operator: 'system',
            createdAt: now()
        }]
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

function defaultLocaleGraph(): LocaleDependencyNode[] {
    return [
        {code: 'en-US', label: 'English', parentCode: null},
        {code: 'zh-CN', label: '简体中文', parentCode: 'en-US'},
        {code: 'ja-JP', label: '日本語', parentCode: 'zh-CN'}
    ]
}

function buildFiles(localeGraph: LocaleDependencyNode[]): ResourceFileNode[] {
    const files: ResourceFileNode[] = []
    for (const node of localeGraph) {
        files.push(
            makeFile(`file-json-${node.code}`, `app.${node.code}.json`, `locale/${node.code}/app.${node.code}.json`, 'json', node.code, node.parentCode),
            makeFile(`file-yaml-${node.code}`, `marketing.${node.code}.yaml`, `locale/${node.code}/marketing.${node.code}.yaml`, 'yaml', node.code, node.parentCode)
        )
    }
    return files
}

function descendantLocales(localeGraph: LocaleDependencyNode[], localeCode: string): string[] {
    const children = localeGraph.filter((item) => item.parentCode === localeCode)
    return children.flatMap((child) => [child.code, ...descendantLocales(localeGraph, child.code)])
}

function replaceLocaleInLogicalPath(logicalPath: string, fromLocale: string, toLocale: string) {
    return logicalPath
        .replace(new RegExp(`/${fromLocale}(?=/|$)`, 'g'), `/${toLocale}`)
        .replace(new RegExp(`\\.${fromLocale}(?=\\.[^.]+$)`, 'g'), `.${toLocale}`)
}

function normalizeResourceGroupKey(file: Pick<ResourceFileNode, 'logicalPath' | 'locale'>) {
    return file.logicalPath
        .replaceAll('\\', '/')
        .replace(new RegExp(`/${file.locale}(?=/|$)`, 'g'), '/{locale}')
        .replace(new RegExp(`\\.${file.locale}(?=\\.[^.]+$)`, 'g'), '.{locale}')
        .toLowerCase()
}

function relatedFileIds(project: ProjectWorkspace, fileId: string) {
    const source = project.files.find((file) => file.id === fileId)
    if (!source) return [] as string[]
    const key = normalizeResourceGroupKey(source)
    return project.files.filter((file) => normalizeResourceGroupKey(file) === key).map((file) => file.id)
}

function directChildFiles(project: ProjectWorkspace, parentFile: ResourceFileNode) {
    const ids = relatedFileIds(project, parentFile.id)
    return ids
        .map((id) => project.files.find((file) => file.id === id))
        .filter((file): file is ResourceFileNode => Boolean(file && file.basedOnLocale === parentFile.locale))
}

function isRootFile(file: ResourceFileNode) {
    return !file.basedOnLocale
}

function findEntryDetail(project: ProjectWorkspace, fileId: string, key: string) {
    return Object.values(project.details).find((detail) => detail.fileId === fileId && detail.key === key)
}

function syncDescendantEntries(project: ProjectWorkspace, parentFile: ResourceFileNode, key: string, parentTargetValue: string, updatedAt = now()) {
    for (const childFile of directChildFiles(project, parentFile)) {
        let childDetail = findEntryDetail(project, childFile.id, key)
        if (!childDetail) {
            const id = crypto.randomUUID()
            childDetail = {
                id,
                fileId: childFile.id,
                key,
                sourceValue: parentTargetValue,
                targetValue: '',
                status: 'new',
                noteCount: 0,
                candidateCount: 0,
                updatedAt,
                note: '',
                issues: buildIssues(parentTargetValue, ''),
                candidates: [],
                history: [{
                    id: crypto.randomUUID(),
                    action: 'create',
                    beforeValue: '',
                    afterValue: '',
                    operator: 'mock-user',
                    createdAt: updatedAt
                }],
                filePath: childFile.logicalPath,
                sourceLocale: parentFile.locale,
                targetLocale: childFile.locale
            }
            project.details[id] = childDetail
        } else if (childDetail.sourceValue !== parentTargetValue) {
            childDetail.sourceValue = parentTargetValue
            childDetail.sourceLocale = parentFile.locale
            childDetail.updatedAt = updatedAt
            childDetail.status = childDetail.targetValue ? 'stale' : 'new'
            childDetail.issues = buildIssues(childDetail.sourceValue, childDetail.targetValue)
        }

        syncDescendantEntries(project, childFile, key, childDetail.targetValue, updatedAt)
    }
}

function createLinkedFiles(project: ProjectWorkspace, input: CreateResourceFileInput) {
    const locales = [input.locale]
    if (input.includeDescendants !== false) {
        locales.push(...descendantLocales(project.localeGraph, input.locale))
    }

    const created: ResourceFileNode[] = []
    for (const locale of locales) {
        const logicalPath = locale === input.locale ? input.logicalPath : replaceLocaleInLogicalPath(input.logicalPath, input.locale, locale)
        if (project.files.some((file) => file.locale === locale && file.logicalPath === logicalPath)) continue
        const parentCode = project.localeGraph.find((node) => node.code === locale)?.parentCode ?? null
        const name = logicalPath.split('/').at(-1) || input.name
        created.push({
            id: crypto.randomUUID(),
            name,
            logicalPath,
            format: input.format,
            locale,
            basedOnLocale: locale === input.locale ? (input.basedOnLocale ?? parentCode) : parentCode,
            rawRelativePath: null
        })
    }
    return created
}

function rebuildProject(project: ProjectWorkspace) {
    project.entries = Object.values(project.details)
        .map((detail) => ({
            id: detail.id,
            fileId: detail.fileId,
            key: detail.key,
            sourceValue: detail.sourceValue,
            targetValue: detail.targetValue,
            status: detail.status,
            noteCount: detail.note ? 1 : 0,
            candidateCount: detail.candidates.length,
            updatedAt: detail.updatedAt
        }))
        .sort((a, b) => a.key.localeCompare(b.key))
    project.treemap = buildTreemap(project.entries)
    project.stats = {
        total: project.entries.length,
        translated: project.entries.filter((entry) => Boolean(entry.targetValue)).length,
        missing: project.entries.filter((entry) => !entry.targetValue).length,
        reviewed: project.entries.filter((entry) => entry.status === 'reviewed' || entry.status === 'approved').length
    }
}

function makeProject(input?: Partial<CreateProjectInput> & { name?: string; path?: string }): ProjectWorkspace {
    const localeGraph = (input?.localeGraph?.length ? input.localeGraph : defaultLocaleGraph()).map((item) => ({
        ...item,
        label: item.label || localeLabels.get(item.code) || item.code
    }))
    const primaryLocale = input?.primaryLocale ?? localeGraph.find((item) => !item.parentCode)?.code ?? localeGraph[0]?.code ?? 'en-US'
    const workingLocale = input?.workingLocale ?? localeGraph.at(-1)?.code ?? primaryLocale
    const upstreamLocale = localeGraph.find((item) => item.code === workingLocale)?.parentCode ?? primaryLocale
    const files = buildFiles(localeGraph)
    const projectName = input?.name ?? 'Demo Localization Project'
    const details = [
        makeEntry('entry-1', `file-json-${workingLocale}`, 'dashboard.welcomeBack', 'Welcome back, {name}', '欢迎回来，{name}', 'translated', upstreamLocale, workingLocale, `locale/${workingLocale}/app.${workingLocale}.json`),
        makeEntry('entry-2', `file-json-${workingLocale}`, 'dashboard.emptyState.title', 'No records found', '', 'new', upstreamLocale, workingLocale, `locale/${workingLocale}/app.${workingLocale}.json`),
        makeEntry('entry-3', `file-yaml-${workingLocale}`, 'marketing.heroSubtitle', 'Ship localization projects faster', '更快交付本地化项目', 'reviewed', upstreamLocale, workingLocale, `locale/${workingLocale}/marketing.${workingLocale}.yaml`)
    ]
    const entryMap = Object.fromEntries(details.map((item) => [item.id, item]))
    const project: ProjectWorkspace = {
        id: `project-${Math.random().toString(36).slice(2, 10)}`,
        name: projectName,
        path: input?.path || defaultProjectPath(projectName),
        localeGraph,
        primaryLocale,
        workingLocale,
        archiveFormat: input?.archiveFormat || 'lzma2',
        keySegmentationProfiles: input?.keySegmentationProfiles || ['dot', 'camel'],
        defaultView: 'list',
        defaultSort: 'updatedDesc',
        dirty: false,
        files,
        entries: [],
        details: entryMap,
        treemap: [],
        stats: {total: 0, translated: 0, missing: 0, reviewed: 0}
    }
    rebuildProject(project)
    return project
}

const mockState: BootstrapResponse = {recentProjects: [defaultProjectPath('demo')], openedProjects: []}

function replaceProject(project: ProjectWorkspace) {
    mockState.openedProjects = mockState.openedProjects.filter((item) => item.id !== project.id).concat(project)
}

export const mockDesktopApi = {
    async bootstrap(): Promise<BootstrapResponse> {
        return structuredClone(mockState)
    },
    async createProject(input: CreateProjectInput): Promise<ProjectWorkspace> {
        const project = makeProject(input)
        replaceProject(project)
        pushRecentProject(project.path)
        return structuredClone(project)
    },
    async openProject(path: string): Promise<ProjectWorkspace> {
        const project = makeProject({path, name: 'Opened Demo Project'})
        replaceProject(project)
        pushRecentProject(project.path)
        return structuredClone(project)
    },
    async saveProject(projectId: string) {
        const project = mockState.openedProjects.find((item) => item.id === projectId)
        if (project) {
            project.dirty = false
            pushRecentProject(project.path)
        }
        return {projectId, path: project?.path || '', savedAt: now()}
    },
    async saveProjectAs(projectId: string, path: string) {
        const project = mockState.openedProjects.find((item) => item.id === projectId)
        if (!project) throw new Error('Project not found in mock state')
        project.path = path.endsWith('.gmproj') ? path : `${path}.gmproj`
        project.dirty = false
        pushRecentProject(project.path)
        replaceProject(project)
        return {project: structuredClone(project), savedAt: now()}
    },
    async updateProjectMetadata(input: UpdateProjectMetadataInput): Promise<ProjectWorkspace> {
        const project = mockState.openedProjects.find((item) => item.id === input.projectId)
        if (!project) throw new Error('Project not found in mock state')
        project.name = input.name
        project.primaryLocale = input.primaryLocale
        project.workingLocale = input.workingLocale
        project.archiveFormat = input.archiveFormat
        project.keySegmentationProfiles = input.keySegmentationProfiles
        project.defaultView = input.defaultView
        project.defaultSort = input.defaultSort
        project.dirty = true
        replaceProject(project)
        return structuredClone(project)
    },
    async createResourceFile(input: CreateResourceFileInput): Promise<ProjectWorkspace> {
        const project = mockState.openedProjects.find((item) => item.id === input.projectId)
        if (!project) throw new Error('Project not found in mock state')
        if (project.files.some((file) => file.locale === input.locale && file.logicalPath === input.logicalPath)) {
            throw new Error('A file with the same locale and logical path already exists')
        }
        project.files.push(...createLinkedFiles(project, input))
        project.files.sort((a, b) => a.logicalPath.localeCompare(b.logicalPath) || a.locale.localeCompare(b.locale))
        project.dirty = true
        replaceProject(project)
        return structuredClone(project)
    },
    async renameResourceFile(input: RenameResourceFileInput): Promise<ProjectWorkspace> {
        const project = mockState.openedProjects.find((item) => item.id === input.projectId)
        if (!project) throw new Error('Project not found in mock state')
        const source = project.files.find((file) => file.id === input.fileId)
        if (!source) throw new Error('File not found in mock state')
        const targets = (input.includeRelated ?? true) ? relatedFileIds(project, input.fileId) : [input.fileId]
        for (const targetId of targets) {
            const file = project.files.find((item) => item.id === targetId)
            if (!file) continue
            const nextLogicalPath = file.locale === source.locale ? input.logicalPath : replaceLocaleInLogicalPath(input.logicalPath, source.locale, file.locale)
            const duplicate = project.files.some((candidate) => candidate.id !== file.id && candidate.locale === file.locale && candidate.logicalPath === nextLogicalPath)
            if (duplicate) throw new Error(`A file already exists at ${nextLogicalPath}`)
            file.logicalPath = nextLogicalPath
            file.name = nextLogicalPath.split('/').at(-1) || input.name
            for (const detail of Object.values(project.details)) {
                if (detail.fileId === file.id) detail.filePath = nextLogicalPath
            }
        }
        project.files.sort((a, b) => a.logicalPath.localeCompare(b.logicalPath) || a.locale.localeCompare(b.locale))
        project.dirty = true
        rebuildProject(project)
        replaceProject(project)
        return structuredClone(project)
    },
    async deleteResourceFile(input: DeleteResourceFileInput): Promise<ProjectWorkspace> {
        const project = mockState.openedProjects.find((item) => item.id === input.projectId)
        if (!project) throw new Error('Project not found in mock state')
        const targets = (input.includeRelated ?? true) ? relatedFileIds(project, input.fileId) : [input.fileId]
        const entryIds = Object.values(project.details).filter((detail) => targets.includes(detail.fileId)).map((detail) => detail.id)
        project.files = project.files.filter((file) => !targets.includes(file.id))
        for (const entryId of entryIds) delete project.details[entryId]
        project.dirty = true
        rebuildProject(project)
        replaceProject(project)
        return structuredClone(project)
    },
    async createEntry(input: CreateEntryInput): Promise<ProjectWorkspace> {
        const project = mockState.openedProjects.find((item) => item.id === input.projectId)
        if (!project) throw new Error('Project not found in mock state')
        const file = project.files.find((item) => item.id === input.fileId)
        if (!file) throw new Error('File not found in mock state')
        if (Object.values(project.details).some((detail) => detail.fileId === input.fileId && detail.key === input.key)) {
            throw new Error(`Entry key already exists in file: ${input.key}`)
        }

        const updatedAt = now()
        const targetValue = isRootFile(file)
            ? (input.targetValue || input.sourceValue || '')
            : (input.targetValue || '')
        const sourceValue = isRootFile(file) ? '' : (input.sourceValue || '')

        const id = crypto.randomUUID()
        project.details[id] = {
            id,
            fileId: file.id,
            key: input.key,
            sourceValue,
            targetValue,
            status: input.status || (targetValue ? 'translated' : 'new'),
            noteCount: input.note ? 1 : 0,
            candidateCount: 0,
            updatedAt,
            note: input.note || '',
            issues: buildIssues(sourceValue, targetValue),
            candidates: [],
            history: [{
                id: crypto.randomUUID(),
                action: 'create',
                beforeValue: '',
                afterValue: targetValue,
                operator: 'mock-user',
                createdAt: updatedAt
            }],
            filePath: file.logicalPath,
            sourceLocale: isRootFile(file) ? '' : (file.basedOnLocale || ''),
            targetLocale: file.locale
        }

        syncDescendantEntries(project, file, input.key, targetValue, updatedAt)
        project.dirty = true
        rebuildProject(project)
        replaceProject(project)
        return structuredClone(project)
    },
    async deleteEntry(input: DeleteEntryInput): Promise<ProjectWorkspace> {
        const project = mockState.openedProjects.find((item) => item.id === input.projectId)
        if (!project) throw new Error('Project not found in mock state')
        delete project.details[input.entryId]
        project.dirty = true
        rebuildProject(project)
        replaceProject(project)
        return structuredClone(project)
    },
    async deleteEntries(input: DeleteEntriesInput): Promise<ProjectWorkspace> {
        const project = mockState.openedProjects.find((item) => item.id === input.projectId)
        if (!project) throw new Error('Project not found in mock state')
        for (const entryId of input.entryIds) delete project.details[entryId]
        project.dirty = true
        rebuildProject(project)
        replaceProject(project)
        return structuredClone(project)
    },
    async updateEntry(input: UpdateEntryInput): Promise<ProjectWorkspace> {
        const project = mockState.openedProjects.find((item) => item.id === input.projectId)
        if (!project) throw new Error('Project not found in mock state')
        const detail = project.details[input.entryId]
        if (!detail) throw new Error('Entry not found in mock state')
        const file = project.files.find((item) => item.id === detail.fileId)
        if (!file) throw new Error('File not found in mock state')
        const previousValue = detail.targetValue
        if (typeof input.targetValue === 'string') detail.targetValue = input.targetValue
        if (typeof input.note === 'string') detail.note = input.note
        if (input.status) detail.status = input.status
        else detail.status = detail.targetValue ? 'translated' : 'new'
        if (isRootFile(file)) {
            detail.sourceValue = ''
            detail.sourceLocale = ''
        }
        detail.updatedAt = now()
        detail.issues = buildIssues(detail.sourceValue, detail.targetValue)
        if (previousValue && previousValue !== detail.targetValue && !detail.candidates.some((candidate) => candidate.value === previousValue)) {
            detail.candidates.unshift({id: crypto.randomUUID(), source: 'history', value: previousValue, score: 0.81})
        }
        detail.history.unshift({
            id: crypto.randomUUID(),
            action: 'edit',
            beforeValue: previousValue,
            afterValue: detail.targetValue,
            operator: 'mock-user',
            createdAt: detail.updatedAt
        })
        syncDescendantEntries(project, file, detail.key, detail.targetValue, detail.updatedAt)
        project.dirty = true
        rebuildProject(project)
        replaceProject(project)
        return structuredClone(project)
    },
    async bulkReplace(input: BulkReplaceInput): Promise<BulkReplaceResult> {
        const project = mockState.openedProjects.find((item) => item.id === input.projectId)
        if (!project) throw new Error('Project not found in mock state')
        const matcher = input.useRegex ? new RegExp(input.search, 'g') : null
        const changedEntryIds: string[] = []
        for (const entry of Object.values(project.details)) {
            const original = entry.targetValue
            const next = input.useRegex ? original.replace(matcher!, input.replacement) : original.split(input.search).join(input.replacement)
            if (next !== original) {
                entry.targetValue = next
                entry.updatedAt = now()
                entry.status = next ? 'translated' : 'new'
                entry.issues = buildIssues(entry.sourceValue, entry.targetValue)
                entry.history.unshift({
                    id: crypto.randomUUID(),
                    action: input.useRegex ? 'bulk_edit_regex' : 'bulk_edit',
                    beforeValue: original,
                    afterValue: next,
                    operator: 'bulk-replace',
                    createdAt: entry.updatedAt
                })
                changedEntryIds.push(entry.id)
            }
        }
        project.dirty = true
        rebuildProject(project)
        replaceProject(project)
        return {changedEntryIds, project: structuredClone(project)}
    },
    async previewImport(input: PreviewImportInput): Promise<ImportPreviewResponse> {
        const previewId = crypto.randomUUID()
        const items = input.files.map((file, index) => ({
            previewFileId: `${previewId}-${index}`,
            path: file.path,
            logicalPath: file.logicalPath || file.path.split(/[\\/]/).pop() || file.path,
            name: file.path.split(/[\\/]/).pop() || file.path,
            format: (file.path.split('.').pop() || 'json') as ResourceFileNode['format'],
            locale: file.locale,
            basedOnLocale: file.basedOnLocale ?? null,
            entryCount: 2,
            conflictCount: 0
        }))
        const entries: EntrySummary[] = items.flatMap((item, index) => ([{
            id: `${item.previewFileId}-1`,
            fileId: item.previewFileId,
            key: `imported.key.${index}.title`,
            sourceValue: 'Imported source',
            targetValue: 'Imported target',
            status: 'translated',
            noteCount: 0,
            candidateCount: 0,
            updatedAt: now()
        }]))
        const preview: ImportPreviewResponse = {
            previewId,
            items,
            entries,
            conflicts: [],
            totals: {files: items.length, entries: entries.length, conflicts: 0}
        }
        importPreviews.set(previewId, preview)
        importPreviewFiles.set(previewId, input.files)
        return structuredClone(preview)
    },
    async commitImport(input: CommitImportInput): Promise<ProjectWorkspace> {
        const project = mockState.openedProjects.find((item) => item.id === input.projectId)
        const preview = importPreviews.get(input.previewId)
        if (!project || !preview) throw new Error('Import preview missing in mock state')
        for (const [index, item] of preview.items.entries()) {
            const fileId = crypto.randomUUID()
            project.files.push({
                id: fileId,
                name: item.name,
                logicalPath: item.logicalPath,
                format: item.format,
                locale: item.locale,
                basedOnLocale: item.basedOnLocale ?? null,
                rawRelativePath: null
            })
            const detail: EntryDetail = {
                id: crypto.randomUUID(),
                fileId,
                key: `imported.${index}.title`,
                sourceValue: 'Imported source',
                targetValue: 'Imported target',
                status: 'translated',
                noteCount: 0,
                candidateCount: 0,
                updatedAt: now(),
                note: '',
                issues: [],
                candidates: [],
                history: [],
                filePath: item.logicalPath,
                sourceLocale: item.basedOnLocale || project.primaryLocale,
                targetLocale: item.locale
            }
            project.details[detail.id] = detail
        }
        project.dirty = true
        rebuildProject(project)
        replaceProject(project)
        return structuredClone(project)
    },
    async exportProject(input: ExportProjectInput): Promise<ExportProjectResult> {
        const project = mockState.openedProjects.find((item) => item.id === input.projectId)
        const file = project?.files.find((item) => item.id === input.fileId) || project?.files[0]
        const outputPath = input.outputPath || (file ? `/tmp/${file.name}` : '/tmp/export.out')
        return {projectId: input.projectId, outputPath, exportedFiles: file ? [outputPath] : []}
    },
    async exportProjectBatch(input: BatchExportInput): Promise<ExportProjectResult> {
        const project = mockState.openedProjects.find((item) => item.id === input.projectId)
        const files = project?.files.filter((file) => input.fileIds.includes(file.id)) || []
        const outputPath = input.outputDirectory || '/tmp/export-batch'
        return {
            projectId: input.projectId,
            outputPath,
            exportedFiles: files.map((file) => `${outputPath}/${file.logicalPath}`)
        }
    },
    async buildTreemap(projectId: string): Promise<TreemapNode[]> {
        return structuredClone(mockState.openedProjects.find((item) => item.id === projectId)?.treemap || [])
    }
}
