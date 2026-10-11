import { describe, expect, it } from 'vitest';
import { countsFor, defaultProfile } from './types';
describe('count preview matches backend conversions', () => {
  it('rounds sensitivity, distance and raw counts', () => {
    expect(
      countsFor({
        ...defaultProfile,
        game: 'custom',
        customYaw: 0.022,
        sensitivity: 1.5,
      }),
    ).toBe(5455);
    expect(
      countsFor({ ...defaultProfile, mode: 'distance', cm360: 25.4, dpi: 800 }),
    ).toBe(4000);
    expect(
      countsFor({ ...defaultProfile, mode: 'raw', rawCounts: 1234.5 }),
    ).toBe(1235);
  });
  it('rejects non-finite and unsafe counts', () => {
    expect(countsFor({ ...defaultProfile, sensitivity: 0 })).toBe(0);
    expect(countsFor({ ...defaultProfile, sensitivity: 1e-12 })).toBe(0);
  });
});
