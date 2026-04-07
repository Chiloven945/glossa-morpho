import {defineStore} from 'pinia'
import type {TranslationStatus, ViewMode} from '~/shared/types/models'

export const useEditorStore = defineStore('editor', {
    state: () => ({
        searchText: '',
        statusFilter: 'all' as TranslationStatus | 'all',
        selectedFileId: null as string | null,
        selectedEntryId: null as string | null,
        currentView: 'list' as ViewMode,
        showOnlyMissing: false,
        bulkSearch: '',
        bulkReplacement: '',
        bulkUseRegex: false
    }),

    actions: {
        selectFile(fileId: string | null) {
            this.selectedFileId = fileId
        },

        selectEntry(entryId: string | null) {
            this.selectedEntryId = entryId
        },

        resetForProject() {
            this.searchText = ''
            this.statusFilter = 'all'
            this.selectedFileId = null
            this.selectedEntryId = null
            this.currentView = 'list'
            this.showOnlyMissing = false
        }
    }
})
