import {coverageConfigDefaults, defineConfig} from 'vitest/config'
import tsconfigPaths from 'vite-tsconfig-paths'

const exclude = [
    '**/node_modules/**',
    '**/dist/**',
    '**/cypress/**',
    '**/.{idea,git,cache,output,temp}/**',
    '**/{karma,rollup,webpack,vite,vitest,jest,ava,babel,nyc,cypress,tsup,build}.config.*',

]

export default defineConfig({
    plugins: [tsconfigPaths()],
    test: {
        coverage: {
            provider: 'v8',
            reporter: ['text', 'clover', 'html'],
            exclude: [
                '**/encore.gen/**',
                '**/coverage/**',
                '**/.encore/**',
                ...coverageConfigDefaults.exclude,
            ],
        },
        exclude,
        watch: false,
    }
})