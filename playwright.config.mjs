import { defineConfig, devices } from '@playwright/test';
export default defineConfig({
  testDir: './tests/browser',
  timeout: 90_000,
  workers: 1,
  use: {
    ...devices['Pixel 7'],
    baseURL: process.env.PWA_URL || 'http://127.0.0.1:4173/iroh-lab/',
    launchOptions: process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE ? { executablePath: process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE } : {},
    // Invitations are bearer credentials: never save automated traces or video.
    trace: 'off',
    screenshot: 'off',
  },
});
