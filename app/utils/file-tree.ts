import type {LocaleDependencyNode, ProjectWorkspace, ResourceFileNode} from '#shared/types/models'

export type FileTreeNodeKind = 'root' | 'locale' | 'file'

export interface FileTreeNode {
    id: string
    kind: FileTreeNodeKind
    label: string
    description?: string
    icon: string
    localeCode?: string | null
    fileId?: string
    selected?: boolean
    relatedLocales?: string[]
    basedOnLocale?: string | null
    children?: FileTreeNode[]
}

function escapeRegExp(value: string) {
    return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
}

export function normalizeResourceGroupKey(file: Pick<ResourceFileNode, 'logicalPath' | 'locale' | 'name'>) {
    const locale = escapeRegExp(file.locale)
    let normalized = (file.logicalPath || file.name || '').replaceAll('\\', '/')
    normalized = normalized.replace(new RegExp(`/${locale}(?=/|$)`, 'g'), '/{locale}')
    normalized = normalized.replace(new RegExp(`(^|[^A-Za-z0-9])${locale}(?=\.[^.]+$)`, 'g'), `$1{locale}`)
    normalized = normalized.replace(new RegExp(`\.${locale}(?=\.[^.]+$)`, 'g'), '.{locale}')
    return normalized.toLowerCase()
}

export function buildFileAssociationMap(files: ResourceFileNode[]) {
    const byGroup = new Map<string, ResourceFileNode[]>()

    for (const file of files) {
        const key = normalizeResourceGroupKey(file)
        const list = byGroup.get(key) || []
        list.push(file)
        byGroup.set(key, list)
    }

    return byGroup
}

function localeMatches(node: LocaleDependencyNode, query: string) {
    const haystack = `${node.label} ${node.code}`.toLowerCase()
    return !query || haystack.includes(query)
}

function fileMatches(file: ResourceFileNode, query: string) {
    const haystack = `${file.name} ${file.logicalPath} ${file.locale} ${file.format}`.toLowerCase()
    return !query || haystack.includes(query)
}

export function buildFileTree(project: ProjectWorkspace, selectedFileId: string | null, filterText: string) {
    const query = filterText.trim().toLowerCase()
    const roots = project.localeGraph.filter((item) => !item.parentCode)
    const localeChildren = new Map<string | null, LocaleDependencyNode[]>()
    const byGroup = buildFileAssociationMap(project.files)

    for (const locale of project.localeGraph) {
        const list = localeChildren.get(locale.parentCode || null) || []
        list.push(locale)
        localeChildren.set(locale.parentCode || null, list)
    }

    const buildLocaleNode = (locale: LocaleDependencyNode): FileTreeNode | null => {
        const files = project.files
            .filter((file) => file.locale === locale.code && fileMatches(file, query))
            .sort((a, b) => a.logicalPath.localeCompare(b.logicalPath))
            .map((file) => {
                const siblings = byGroup.get(normalizeResourceGroupKey(file)) || [file]
                const relatedLocales = siblings.map((item) => item.locale)
                return {
                    id: file.id,
                    kind: 'file' as const,
                    label: file.name,
                    description: file.logicalPath,
                    icon: 'i-lucide-file-text',
                    fileId: file.id,
                    localeCode: file.locale,
                    selected: file.id === selectedFileId,
                    relatedLocales,
                    basedOnLocale: file.basedOnLocale ?? null
                }
            })

        const childLocales = (localeChildren.get(locale.code) || [])
            .map((item) => buildLocaleNode(item))
            .filter(Boolean) as FileTreeNode[]

        const visible = localeMatches(locale, query) || files.length > 0 || childLocales.length > 0
        if (!visible) return null

        return {
            id: `locale:${locale.code}`,
            kind: 'locale',
            label: `${locale.label} · ${locale.code}`,
            icon: locale.parentCode ? 'i-lucide-git-branch-plus' : 'i-lucide-languages',
            localeCode: locale.code,
            children: [...files, ...childLocales]
        }
    }

    return [
        {
            id: 'all-files',
            kind: 'root' as const,
            label: 'All Files',
            icon: 'i-lucide-files'
        },
        ...(roots.map((root) => buildLocaleNode(root)).filter(Boolean) as FileTreeNode[])
    ]
}
