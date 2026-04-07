export default defineNuxtPlugin(() => {
    window.addEventListener('keydown', (event) => {
        const isCommandK = (event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'k'
        if (isCommandK) {
            event.preventDefault()
            window.dispatchEvent(new CustomEvent('gm:toggle-command-palette'))
        }
    })
})
