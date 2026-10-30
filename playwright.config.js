import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './tests/browser',
  fullyParallel: true,
  forbidOnly: Boolean(process.env.CI),
  retries: process.env.CI ? 1 : 0,
  // WebGPU is CPU-rendered in Linux CI. Concurrent renderers and continuous
  // screen recording starve animation/input updates on the hosted runner.
  workers: process.env.CI ? 1 : undefined,
  reporter: process.env.CI ? 'github' : 'list',
  timeout: 45_000,
  expect: { timeout: 8_000 },
  use: {
    ...devices['Desktop Chrome'],
    baseURL: 'http://127.0.0.1:18435',
    browserName: 'chromium',
    launchOptions: {
      args: [
        '--enable-unsafe-webgpu',
        ...(process.platform === 'linux' ? [
          '--enable-features=Vulkan',
          '--use-angle=vulkan',
          '--use-vulkan=swiftshader',
          '--use-webgpu-adapter=swiftshader',
          '--disable-vulkan-surface',
        ] : []),
      ],
    },
    screenshot: 'only-on-failure',
    trace: { mode: 'retain-on-failure', screenshots: false, snapshots: true, sources: true },
    video: 'off',
  },
  webServer: {
    command: 'python3 -m http.server 18435 --bind 127.0.0.1 --directory web',
    url: 'http://127.0.0.1:18435',
    reuseExistingServer: !process.env.CI,
    timeout: 10_000,
  },
  projects: [{ name: 'chromium', use: { ...devices['Desktop Chrome'] } }],
  outputDir: 'test-results',
});
