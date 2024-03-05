import { test, expect } from '@playwright/test';
import { signInWithNewUser } from './utils';

test("use registers with Internet Identity and is redirected to a success page", async ({ page, context }) => {
  await page.goto('/');
  await expect(page).toHaveTitle(/Early Adopter Campaign/);

  await signInWithNewUser({ page, context });
  
  expect(await page.getByTestId("success-page-title").textContent()).toBe("Congratulations!");
});