import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import fs from 'fs';

// https://vitejs.dev/config/
export default defineConfig({
  server: { https: {
    key: fs.readFileSync('../key.pem'),
    cert: fs.readFileSync('../cert.pem'),
  } },
  plugins: [vue()],
})
