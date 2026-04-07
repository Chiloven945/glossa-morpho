export default defineNuxtConfig({
  compatibilityDate: '2026-04-07',
  devtools: { enabled: true },
  ssr: false,
  modules: ['@nuxt/ui', '@pinia/nuxt', '@nuxtjs/i18n'],
  css: ['~/assets/css/main.css'],
  imports: {
    dirs: ['app/stores', 'app/composables', 'app/utils']
  },
  ui: {
    fonts: false
  },
  i18n: {
    defaultLocale: 'zh-CN',
    lazy: true,
    langDir: 'locales',
    locales: [
      { code: 'zh-CN', language: 'zh-CN', file: 'zh-CN.json', name: '简体中文' },
      { code: 'en-US', language: 'en-US', file: 'en-US.json', name: 'English' },
      { code: 'ja-JP', language: 'ja-JP', file: 'ja-JP.json', name: '日本語' }
    ],
    vueI18n: './i18n/i18n.config.ts'
  }
})
