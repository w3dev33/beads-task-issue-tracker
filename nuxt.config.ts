import tailwindcss from '@tailwindcss/vite'
import pkg from './package.json'

export default defineNuxtConfig({
  runtimeConfig: {
    public: {
      appVersion: pkg.version,
    },
  },
  compatibilityDate: '2025-01-01',
  future: {
    compatibilityVersion: 4,
  },

  ssr: false,

  router: {
    options: {
      hashMode: true,
    },
  },

  app: {
    cdnURL: './',
  },

  devServer: {
    port: 3133,
  },

  modules: ['shadcn-nuxt'],

  shadcn: {
    prefix: '',
    componentDir: './app/components/ui',
  },

  vite: {
    plugins: [tailwindcss()],
  },

  css: ['~/assets/css/tailwind.css'],

  devtools: { enabled: false },
})
