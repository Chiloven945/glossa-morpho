import {defineStore} from 'pinia'
import type {TranslationStatus, ViewMode} from '#shared/types/models'

export type EntrySortBy = 'updatedDesc' | 'keyAsc' | 'status'
export type WorkspacePanel = 'files' | 'entries' | 'inspector'

interface SelectEntryOptions {
    append?: boolean
    range?: boolean
    orderedIds?: string[]
}

export const useEditorStore = defineStore('editor', {
    state: () => ({
        searchText: '',
        statusFilter: 'all' as TranslationStatus | 'all',
        sortBy: 'updatedDesc' as EntrySortBy,
        selectedFileId: null as string | null,
        selectedEntryId: null as string | null,
        selectedEntryIds: [] as string[],
        lastSelectionAnchorId: null as string | null,
        currentView: 'list' as ViewMode,
        showOnlyMissing: false,
        bulkSearch: '',
        bulkReplacement: '',
        bulkUseRegex: false,
        panelVisibility: {
            files: true,
            entries: true,
            inspector: true
        } as Record<WorkspacePanel, boolean>
    }),

    actions: {
        selectFile(fileId: string | null) {
            if (this.selectedFileId !== fileId) {
                this.selectedEntryId = null
                this.selectedEntryIds = []
                this.lastSelectionAnchorId = null
            }
            this.selectedFileId = fileId
        },

        selectEntry(entryId: string | null, options: SelectEntryOptions = {}) {
            if (!entryId) {
                this.selectedEntryId = null
                this.selectedEntryIds = []
                this.lastSelectionAnchorId = null
                return
            }

            if (options.range && this.lastSelectionAnchorId && options.orderedIds?.length) {
                const startIndex = options.orderedIds.indexOf(this.lastSelectionAnchorId)
                const endIndex = options.orderedIds.indexOf(entryId)
                if (startIndex >= 0 && endIndex >= 0) {
                    const [from, to] = startIndex <= endIndex ? [startIndex, endIndex] : [endIndex, startIndex]
                    const range = options.orderedIds.slice(from, to + 1)
                    this.selectedEntryIds = Array.from(new Set([...(options.append ? this.selectedEntryIds : []), ...range]))
                    this.selectedEntryId = entryId
                    return
                }
            }

            if (options.append) {
                this.selectedEntryIds = this.selectedEntryIds.includes(entryId)
                    ? this.selectedEntryIds.filter((item) => item !== entryId)
                    : [...this.selectedEntryIds, entryId]
                this.selectedEntryId = entryId
                this.lastSelectionAnchorId = entryId
                if (this.selectedEntryIds.length === 0) this.selectedEntryId = null
                return
            }

            this.selectedEntryId = entryId
            this.selectedEntryIds = [entryId]
            this.lastSelectionAnchorId = entryId
        },

        selectAllEntries(entryIds: string[]) {
            this.selectedEntryIds = [...entryIds]
            this.selectedEntryId = entryIds[0] ?? null
            this.lastSelectionAnchorId = entryIds[0] ?? null
        },

        syncSelectedEntries(validIds: string[]) {
            const valid = new Set(validIds)
            this.selectedEntryIds = this.selectedEntryIds.filter((id) => valid.has(id))
            if (this.selectedEntryId && !valid.has(this.selectedEntryId)) {
                this.selectedEntryId = this.selectedEntryIds[0] ?? null
            }
            if (this.lastSelectionAnchorId && !valid.has(this.lastSelectionAnchorId)) {
                this.lastSelectionAnchorId = this.selectedEntryId
            }
        },

        setStatusFilter(value: TranslationStatus | 'all') {
            this.statusFilter = value
        },

        setSortBy(value: EntrySortBy) {
            this.sortBy = value
        },

        togglePanel(panel: WorkspacePanel) {
            const next = !this.panelVisibility[panel]
            const visibleCount = Object.values(this.panelVisibility).filter(Boolean).length
            if (!next && visibleCount === 1) return
            this.panelVisibility = {...this.panelVisibility, [panel]: next}
        },

        resetForProject() {
            this.searchText = ''
            this.statusFilter = 'all'
            this.sortBy = 'updatedDesc'
            this.selectedFileId = null
            this.selectedEntryId = null
            this.selectedEntryIds = []
            this.lastSelectionAnchorId = null
            this.currentView = 'list'
            this.showOnlyMissing = false
            this.bulkSearch = ''
            this.bulkReplacement = ''
            this.bulkUseRegex = false
            this.panelVisibility = {
                files: true,
                entries: true,
                inspector: true
            }
        }
    }
})
