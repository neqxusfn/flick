import { defineConfig } from '@playwright/test';
export default defineConfig({
  testDir: './tests',
  fullyParallel: true,
  retries: 0,
  use: {
    browserName: 'chromium',
    channel: 'msedge',
    baseURL: 'http://127.0.0.1:1420',
    viewport: { width: 520, height: 640 },
    reducedMotion: 'reduce',
  },
  projects: [1, 1.25, 1.5].flatMap((scale) =>
    (['light', 'dark'] as const).map((theme) => ({
      name: `${theme}-${scale}`,
      use: { deviceScaleFactor: scale, colorScheme: theme },
    })),
  ),
  webServer: {
    command: 'npm run dev',
    url: 'http://127.0.0.1:1420',
    reuseExistingServer: !process.env.CI,
  },
});
