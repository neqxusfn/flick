export type Mode = 'game' | 'distance' | 'raw';
export type Direction = 'left' | 'right';
export type Theme = 'system' | 'light' | 'dark';
export interface Profile {
  id: string;
  name: string;
  game: string;
  mode: Mode;
  sensitivity: number;
  customYaw: number;
  cm360: number;
  dpi: number;
  rawCounts: number;
  trigger: number;
  triggerLabel: string;
  duration: number;
  direction: Direction;
  flickBack: boolean;
  process: string;
}
export interface Settings {
  version: number;
  riskAcknowledged: boolean;
  profiles: Profile[];
  activeId: string;
  autoSwitch: boolean;
  startWithWindows: boolean;
  startMinimized: boolean;
  theme: Theme;
}
export interface Snapshot {
  settings: Settings;
  enabled: boolean;
  recording: boolean;
  calibration: 'idle' | 'armed' | 'measuring';
  error: string | null;
  healthy: boolean;
  environment: {
    acceleration: boolean;
    elevated: boolean;
    foregroundElevated: boolean;
  };
}
export const defaultProfile: Profile = {
  id: 'default',
  name: 'My profile',
  game: 'valorant',
  mode: 'game',
  sensitivity: 0.5,
  customYaw: 0.022,
  cm360: 32.66,
  dpi: 800,
  rawCounts: 5143,
  trigger: 5,
  triggerLabel: 'Mouse 4',
  duration: 12,
  direction: 'right',
  flickBack: false,
  process: '',
};
export const defaultSettings: Settings = {
  version: 1,
  riskAcknowledged: false,
  profiles: [defaultProfile],
  activeId: 'default',
  autoSwitch: false,
  startWithWindows: false,
  startMinimized: false,
  theme: 'system',
};
export const initialState: Snapshot = {
  settings: defaultSettings,
  enabled: false,
  recording: false,
  calibration: 'idle',
  error: null,
  healthy: true,
  environment: {
    acceleration: false,
    elevated: false,
    foregroundElevated: false,
  },
};
// Primary source: KovaaK's 2.0.1.0 developer release notes; see docs/yaw-sources.md.
// https://store.steampowered.com/news/posts/?appids=824270&enddate=1595643090&feed=steam_community_announcements
export const presets = [
  { id: 'valorant', name: 'Valorant', yaw: 0.07 },
  { id: 'fortnite', name: 'Fortnite', yaw: 0.005555 },
  { id: 'custom', name: 'Custom', yaw: null },
] as const;
export function countsFor(profile: Profile): number {
  const yaw =
    presets.find((p) => p.id === profile.game)?.yaw ?? profile.customYaw;
  const value =
    profile.mode === 'game'
      ? 180 / (profile.sensitivity * yaw)
      : profile.mode === 'distance'
        ? (profile.cm360 / 2 / 2.54) * profile.dpi
        : profile.rawCounts;
  return Number.isFinite(value) && value >= 1 && value <= 1_000_000
    ? Math.round(value)
    : 0;
}
