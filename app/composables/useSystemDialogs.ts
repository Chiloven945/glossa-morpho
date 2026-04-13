function ensureGmprojExtension(path: string) {
    return path.toLowerCase().endsWith('.gmproj') ? path : `${path}.gmproj`
}

function fileNameFromPath(path: string) {
    return path.split(/[\\/]/).pop() || path
}

async function getDialogModule() {
    return import('@tauri-apps/plugin-dialog')
}

const exportExtensions: Record<string, string[]> = {
    json: ['json'],
    yaml: ['yaml', 'yml'],
    properties: ['properties'],
    resx: ['resx'],
    xaml: ['xaml'],
    xliff: ['xlf', 'xliff']
}

export function useSystemDialogs() {
    async function pickProjectToOpen() {
        try {
            const {open} = await getDialogModule()
            const selected = await open({
                multiple: false,
                directory: false,
                filters: [{name: 'glossa-morpho Project', extensions: ['gmproj']}]
            })
            return typeof selected === 'string' ? selected : null
        } catch {
            return null
        }
    }

    async function pickProjectSavePath(options?: { defaultPath?: string; projectName?: string }) {
        try {
            const {save} = await getDialogModule()
            const defaultPath = options?.defaultPath?.trim()
                ? options.defaultPath
                : `${(options?.projectName || 'project').trim() || 'project'}.gmproj`
            const selected = await save({
                defaultPath,
                filters: [{name: 'glossa-morpho Project', extensions: ['gmproj']}]
            })
            return typeof selected === 'string' && selected ? ensureGmprojExtension(selected) : null
        } catch {
            return null
        }
    }

    async function pickImportFiles() {
        try {
            const {open} = await getDialogModule()
            const selected = await open({
                multiple: true,
                directory: false,
                filters: [
                    {
                        name: 'Localization Resources',
                        extensions: ['json', 'yaml', 'yml', 'properties', 'resx', 'xaml', 'xlf', 'xliff']
                    }
                ]
            })

            if (typeof selected === 'string') return [selected]
            if (Array.isArray(selected)) return selected.filter((item): item is string => typeof item === 'string')
            return []
        } catch {
            return []
        }
    }


    async function pickDirectory(defaultPath?: string) {
        try {
            const {open} = await getDialogModule()
            const selected = await open({
                multiple: false,
                directory: true,
                defaultPath
            })
            return typeof selected === 'string' ? selected : null
        } catch {
            return null
        }
    }

    async function pickExportFile(options: { defaultPath?: string; format: string }) {
        try {
            const {save} = await getDialogModule()
            const selected = await save({
                defaultPath: options.defaultPath,
                filters: [{
                    name: `${options.format.toUpperCase()} Resource`,
                    extensions: exportExtensions[options.format] || [options.format]
                }]
            })
            return typeof selected === 'string' && selected ? selected : null
        } catch {
            return null
        }
    }

    return {
        pickProjectToOpen,
        pickProjectSavePath,
        pickImportFiles,
        pickDirectory,
        pickExportFile,
        fileNameFromPath
    }
}
