export type WorkspaceActionId =
    | 'file.new'
    | 'file.rename'
    | 'file.delete'
    | 'file.import'
    | 'file.export'
    | 'file.exportBatch'
    | 'entry.new'
    | 'entry.delete'
    | 'entry.deleteSelected'

export interface WorkspaceActionDescriptor {
    id: WorkspaceActionId
    labelKey: string
    icon: string
    color?: 'neutral' | 'primary' | 'error' | 'warning' | 'success' | 'info'
}

export interface WorkspaceContextMenuItem {
    label: string
    value: string
    icon?: string
    disabled?: boolean
    color?: 'neutral' | 'primary' | 'error' | 'warning' | 'success' | 'info'
}

const definitions: Record<WorkspaceActionId, WorkspaceActionDescriptor> = {
    'file.new': {id: 'file.new', labelKey: 'actions.newFile', icon: 'i-lucide-file-plus'},
    'file.rename': {id: 'file.rename', labelKey: 'actions.renameFile', icon: 'i-lucide-file-pen-line'},
    'file.delete': {id: 'file.delete', labelKey: 'actions.deleteFile', icon: 'i-lucide-trash-2', color: 'error'},
    'file.import': {id: 'file.import', labelKey: 'actions.importResources', icon: 'i-lucide-file-down'},
    'file.export': {id: 'file.export', labelKey: 'actions.exportResources', icon: 'i-lucide-file-up'},
    'file.exportBatch': {id: 'file.exportBatch', labelKey: 'actions.batchExport', icon: 'i-lucide-files'},
    'entry.new': {id: 'entry.new', labelKey: 'actions.newEntry', icon: 'i-lucide-file-plus-2'},
    'entry.delete': {id: 'entry.delete', labelKey: 'actions.deleteEntry', icon: 'i-lucide-trash-2', color: 'error'},
    'entry.deleteSelected': {
        id: 'entry.deleteSelected',
        labelKey: 'actions.deleteSelectedEntries',
        icon: 'i-lucide-trash-2',
        color: 'error'
    }
}

export function actionValue(actionId: WorkspaceActionId, payload?: string) {
    return payload ? `${actionId}::${payload}` : actionId
}

export function parseActionValue(value: string) {
    const [actionId, payload] = value.split('::')
    return {actionId: actionId as WorkspaceActionId, payload: payload ?? null}
}

export function buildContextMenuItems(
    t: (key: string) => any,
    items: Array<{ id: WorkspaceActionId; payload?: string; disabled?: boolean }>
): WorkspaceContextMenuItem[] {
    return items.map((item) => {
        const definition = definitions[item.id]
        return {
            label: t(definition.labelKey),
            value: actionValue(item.id, item.payload),
            icon: definition.icon,
            color: definition.color,
            disabled: item.disabled
        }
    })
}
