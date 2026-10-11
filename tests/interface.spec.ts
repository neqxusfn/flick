import { test, expect } from '@playwright/test';
test('main screen fits, counts update, theme and settings stay accessible', async ({
  page,
}, testInfo) => {
  await page.goto('/');
  await expect(
    page.getByRole('dialog', { name: 'Before you enable Flick' }),
  ).toBeVisible();
  await page.getByRole('button', { name: 'Keep paused' }).click();
  await page.getByRole('switch', { name: 'Enable Flick' }).click();
  await expect(
    page.getByRole('dialog', { name: 'Before you enable Flick' }),
  ).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Paused' })).toBeVisible();
  await page.getByRole('button', { name: 'I understand the risk' }).click();
  await expect(page.getByRole('heading', { name: 'Paused' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Test flick' })).toBeDisabled();
  await page.getByRole('switch', { name: 'Enable Flick' }).click();
  await expect(page.getByRole('heading', { name: 'Enabled' })).toBeVisible();
  const footer = await page.locator('footer').boundingBox();
  expect(footer).not.toBeNull();
  expect(footer!.y + footer!.height).toBeLessThanOrEqual(635);
  expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBe(
    520,
  );
  await page.screenshot({
    path: `test-results/flick-${testInfo.project.name}.png`,
  });
  await page.getByLabel('Sensitivity', { exact: true }).fill('1');
  await page.getByLabel('Sensitivity', { exact: true }).press('Tab');
  await expect(page.locator('.count')).toContainText('2,571');
  await page.getByRole('button', { name: 'Settings', exact: true }).click();
  await expect(page.getByRole('dialog')).toBeVisible();
  await page.getByLabel('Calibration mode').selectOption('distance');
  await page.getByLabel('Centimeters per 360 degrees').last().fill('25.4');
  await page.getByLabel('Centimeters per 360 degrees').last().press('Tab');
  await page.screenshot({
    path: `test-results/settings-${testInfo.project.name}.png`,
  });
  await page.getByRole('button', { name: 'Back to Flick' }).click();
  await expect(page.locator('.count')).toContainText('4,000');
  await page.getByRole('button', { name: 'Settings', exact: true }).click();
  await page.getByRole('button', { name: 'New profile' }).click();
  await expect(page.getByLabel('Profile name')).toHaveValue('Profile 2');
  await page.getByLabel('Calibration mode').selectOption('raw');
  await page
    .getByRole('dialog')
    .getByLabel('Raw counts', { exact: true })
    .fill('6000');
  await page
    .getByRole('dialog')
    .getByLabel('Raw counts', { exact: true })
    .press('Tab');
  await page.keyboard.press('Escape');
  await expect(page.getByRole('dialog')).not.toBeVisible();
  await expect(page.locator('.count')).toContainText('6,000');
  await page.getByRole('button', { name: 'Settings', exact: true }).click();
  await page.getByRole('button', { name: 'Delete', exact: true }).click();
  await page
    .getByRole('button', { name: 'Delete profile', exact: true })
    .click();
  await expect(page.getByLabel('Active profile')).toHaveValue('default');
});
test('invalid input keeps valid counts and preview reports native-only actions', async ({
  page,
}) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Keep paused' }).click();
  await page.getByLabel('Sensitivity', { exact: true }).fill('0');
  await page.getByLabel('Sensitivity', { exact: true }).press('Tab');
  await expect(page.getByLabel('Sensitivity', { exact: true })).toHaveAttribute(
    'aria-invalid',
    'true',
  );
  await expect(page.locator('.count')).toContainText('5,143');
  await page.getByRole('button', { name: 'Click to record trigger' }).click();
  await expect(page.getByRole('alert')).toContainText('Open the Windows app');
});
