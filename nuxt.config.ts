import tailwindcss from '@tailwindcss/vite'
import pkg from './package.json'

const isDevMode = process.env.NODE_ENV !== 'production'

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
    build: {
      sourcemap: isDevMode,
      target: 'es2020',
      minify: 'terser',
      terserOptions: {
        compress: {
          drop_console: false,
          pure_funcs: ['console.log', 'console.debug', 'console.info', 'console.warn'],
        },
        format: {
          comments: false,
        },
      },
    },
  },

  css: ['~/assets/css/tailwind.css'],

  devtools: { enabled: false },
})
