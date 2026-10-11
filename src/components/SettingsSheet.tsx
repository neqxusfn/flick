import { useEffect, useRef, useState } from 'react';
import {
  ArrowLeft,
  Plus,
  Trash2,
  SlidersHorizontal,
  ShieldCheck,
  Info,
  X,
} from 'lucide-react';
import type { Mode, Profile, Settings, Snapshot, Theme } from '../types';
import { NumberField, Row, Toggle } from './Controls';
import { native } from '../hooks/useFlick';
interface Props {
  error: string | null;
  dismissError: () => void;
  state: Snapshot;
  close: () => void;
  updateSettings: (s: Partial<Settings>) => void;
  updateProfile: (p: Partial<Profile>) => void;
  command: (name: string, args?: Record<string, unknown>) => Promise<void>;
}
export function SettingsSheet({
  error,
  dismissError,
  state,
  close,
  updateSettings,
  updateProfile,
  command,
}: Props) {
  const dialog = useRef<HTMLDialogElement>(null);
  const [remove, setRemove] = useState(false);
  const { settings } = state;
  const profile = settings.profiles.find((p) => p.id === settings.activeId)!;
  useEffect(() => {
    const element = dialog.current;
    element?.showModal();
    return () => element?.close();
  }, []);
  const addProfile = () => {
    const id = crypto.randomUUID();
    updateSettings({
      profiles: [
        ...settings.profiles,
        {
          ...profile,
          id,
          name: `Profile ${settings.profiles.length + 1}`,
          process: '',
        },
      ],
      activeId: id,
    });
    setRemove(false);
  };
  const deleteProfile = () => {
    const profiles = settings.profiles.filter((p) => p.id !== profile.id);
    updateSettings({ profiles, activeId: profiles[0].id });
    setRemove(false);
  };
  return (
    <dialog
      ref={dialog}
      className="settings-sheet"
      aria-labelledby="settings-title"
      onCancel={(e) => {
        e.preventDefault();
        close();
      }}
    >
      {error && (
        <div className="toast" role="alert">
          <Info size={17} />
          <span>{error}</span>
          <button
            className="icon-button"
            aria-label="Dismiss error"
            onClick={dismissError}
          >
            <X size={15} />
          </button>
        </div>
      )}
      <header className="sheet-header">
        <button
          className="icon-button"
          aria-label="Back to Flick"
          onClick={close}
        >
          <ArrowLeft size={20} />
        </button>
        <h2 id="settings-title">Settings</h2>
        <span>Make it yours.</span>
      </header>
      <div className="sheet-body">
        <section>
          <h3>Profiles</h3>
          <div className="card">
            <Row label="Active profile">
              <select
                aria-label="Active profile"
                value={profile.id}
                onChange={(e) => {
                  updateSettings({ activeId: e.target.value });
                  setRemove(false);
                }}
              >
                {settings.profiles.map((p) => (
                  <option key={p.id} value={p.id}>
                    {p.name}
                  </option>
                ))}
              </select>
            </Row>
            <Row label="Name">
              <input
                className="text-field"
                aria-label="Profile name"
                key={profile.id}
                defaultValue={profile.name}
                maxLength={80}
                onBlur={(e) => {
                  const name = e.target.value.trim();
                  if (name) updateProfile({ name });
                  else e.target.value = profile.name;
                }}
              />
            </Row>
            <div className="profile-actions">
              <button
                className="text-button"
                onClick={addProfile}
                disabled={settings.profiles.length >= 50}
              >
                <Plus size={15} />
                New profile
              </button>
              <button
                className="text-button muted"
                disabled={settings.profiles.length === 1}
                onClick={() => setRemove(!remove)}
              >
                <Trash2 size={14} />
                Delete
              </button>
            </div>
            {remove && (
              <div className="confirm-delete">
                <span>Delete “{profile.name}”?</span>
                <button className="text-button" onClick={deleteProfile}>
                  Delete profile
                </button>
                <button
                  className="text-button muted"
                  onClick={() => setRemove(false)}
                >
                  Keep
                </button>
              </div>
            )}
          </div>
        </section>
        <section>
          <h3>Calibration</h3>
          <div className="card">
            <Row label="Measure by">
              <select
                aria-label="Calibration mode"
                value={profile.mode}
                onChange={(e) =>
                  updateProfile({ mode: e.target.value as Mode })
                }
              >
                <option value="game">Game sensitivity</option>
                <option value="distance">Distance + DPI</option>
                <option value="raw">Raw counts</option>
              </select>
            </Row>
            {profile.mode === 'game' && profile.game === 'custom' && (
              <Row label="Custom yaw">
                <NumberField
                  label="Custom yaw"
                  value={profile.customYaw}
                  min={0.000001}
                  step={0.001}
                  onChange={(customYaw) => updateProfile({ customYaw })}
                />
              </Row>
            )}
            {profile.mode === 'distance' && (
              <>
                <Row label="cm per 360°">
                  <NumberField
                    label="Centimeters per 360 degrees"
                    value={profile.cm360}
                    onChange={(cm360) => updateProfile({ cm360 })}
                  />
                </Row>
                <Row label="Mouse DPI">
                  <NumberField
                    label="Mouse DPI"
                    value={profile.dpi}
                    min={1}
                    step={100}
                    onChange={(dpi) => updateProfile({ dpi })}
                  />
                </Row>
              </>
            )}
            {profile.mode === 'raw' && (
              <Row label="Counts per 180°">
                <NumberField
                  label="Raw counts"
                  value={profile.rawCounts}
                  min={1}
                  step={1}
                  onChange={(rawCounts) => updateProfile({ rawCounts })}
                />
              </Row>
            )}
            <Row label="Measure a manual turn">
              <button
                className="text-button"
                disabled={!native}
                onClick={() =>
                  void command('calibrate', {
                    cancel: state.calibration !== 'idle',
                  })
                }
              >
                <SlidersHorizontal size={15} />
                {state.calibration === 'idle' ? 'Calibrate' : 'Cancel'}
              </button>
            </Row>
          </div>
          <p
            className={`help ${state.calibration !== 'idle' ? 'accent-text' : ''}`}
          >
            {state.calibration === 'measuring'
              ? 'Turn exactly 180° with your mouse. Press F9 again to save.'
              : state.calibration === 'armed'
                ? 'Switch to your game, aim at a landmark, then press F9 to begin.'
                : 'Calibrate in game with F9. Flick pauses while you measure.'}
          </p>
          {profile.mode === 'game' && (
            <p className="help">
              Only sourced presets are included. For other games, use a measured
              distance or raw counts.
            </p>
          )}
        </section>
        <section>
          <h3>Behavior</h3>
          <div className="card">
            <Row label="Flick back on release">
              <Toggle
                label="Flick back on release"
                checked={profile.flickBack}
                onChange={(flickBack) => updateProfile({ flickBack })}
              />
            </Row>
            <Row label="Switch profiles with game">
              <Toggle
                label="Auto-switch profiles"
                checked={settings.autoSwitch}
                onChange={(autoSwitch) => updateSettings({ autoSwitch })}
              />
            </Row>
            <Row label="Game process">
              <input
                className="text-field"
                aria-label="Game process"
                key={`${profile.id}-process`}
                defaultValue={profile.process}
                placeholder="e.g. cs2.exe"
                maxLength={100}
                onBlur={(e) =>
                  updateProfile({
                    process: e.target.value.trim().toLowerCase(),
                  })
                }
              />
            </Row>
          </div>
          <p className="help">
            F8 enables or pauses Flick. F9 is reserved for calibration. Remove
            your trigger’s existing game binding to avoid conflicts.
          </p>
        </section>
        <section>
          <h3>App</h3>
          <div className="card">
            <Row label="Appearance">
              <select
                aria-label="Appearance"
                value={settings.theme}
                onChange={(e) =>
                  updateSettings({ theme: e.target.value as Theme })
                }
              >
                <option value="system">System</option>
                <option value="light">Light</option>
                <option value="dark">Dark</option>
              </select>
            </Row>
            <Row label="Start with Windows">
              <Toggle
                label="Start with Windows"
                disabled={!native}
                checked={settings.startWithWindows}
                onChange={(startWithWindows) =>
                  updateSettings({ startWithWindows })
                }
              />
            </Row>
            <Row label="Open in the tray">
              <Toggle
                label="Start minimized"
                checked={settings.startMinimized}
                onChange={(startMinimized) =>
                  updateSettings({ startMinimized })
                }
              />
            </Row>
          </div>
          <p className="help">
            Flick starts paused. Your trigger works normally until you enable
            it.
          </p>
        </section>
        <section className="compatibility">
          <ShieldCheck size={20} />
          <div>
            <strong>Account risk in online games.</strong>
            <p>
              Flick automates a turn. Standard Windows input does not make it
              ban-safe. Check each game's current rules and use Flick only where
              input automation is allowed. Turn off Windows “Enhance pointer
              precision” for consistent movement.
            </p>
            {native && !state.environment.elevated && (
              <button
                className="text-button"
                onClick={() => void command('run_as_admin')}
              >
                Run as administrator
              </button>
            )}
          </div>
        </section>
        <p className="version">Flick 0.1.0 · Made for a little less effort.</p>
      </div>
    </dialog>
  );
}
