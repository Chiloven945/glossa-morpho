export interface TreemapAggregationRequest {
    keys: string[]
}

export interface TreemapAggregationResult {
    groups: Record<string, number>
}

export function aggregateTreemap(request: TreemapAggregationRequest): TreemapAggregationResult {
    const groups = request.keys.reduce<Record<string, number>>((acc, key) => {
        const group = key.split('.')[0] || 'root'
        acc[group] = (acc[group] ?? 0) + 1
        return acc
    }, {})

    return {groups}
}
