export default defineNuxtPlugin(() => {
    if (!import.meta.client) return

    const allowNative = (event: MouseEvent) => {
        const target = event.target as HTMLElement | null
        return Boolean(target?.closest('[data-allow-native-context-menu="true"]'))
    }

    document.addEventListener(
        'contextmenu',
        (event: MouseEvent) => {
            if (allowNative(event)) return
            event.preventDefault()
        },
        {capture: true}
    )
})
