export interface RegexPreviewResult {
    original: string
    next: string
}

export function previewRegexReplace(values: string[], pattern: RegExp, replacement: string): RegexPreviewResult[] {
    return values.map((original) => ({
        original,
        next: original.replace(pattern, replacement)
    }))
}
