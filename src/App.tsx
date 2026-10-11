import { useEffect, useRef, useState } from 'react';
import {
  ArrowLeft,
  ArrowRight,
  ArrowUpRight,
  CornerUpRight,
  Gamepad2,
  Mouse,
  Settings2,
  X,
  Info,
  ChevronRight,
} from 'lucide-react';
import { useFlick, native } from './hooks/useFlick';
import { useTheme } from './hooks/useTheme';
import { countsFor, presets } from './types';
import { NumberField, Row, Toggle } from './components/Controls';
import { Titlebar } from './components/Titlebar';
import { SettingsSheet } from './components/SettingsSheet';
export default function App() {
  const flick = useFlick();
  const {
    state,
    ready,
    error,
    updateProfile,
    updateSettings,
    command,
    dismissError,
  } = flick;
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [testing, setTesting] = useState(false);
  const [tip, setTip] = useState(false);
  const [noticeOpen, setNoticeOpen] = useState(false);
  const noticeDialog = useRef<HTMLDialogElement>(null);
  useEffect(() => {
    if (ready && !state.settings.riskAcknowledged) setNoticeOpen(true);
  }, [ready, state.settings.riskAcknowledged]);
  useEffect(() => {
    const dialog = noticeDialog.current;
    if (!dialog) return;
    if (noticeOpen && !dialog.open) dialog.showModal();
    if (!noticeOpen && dialog.open) dialog.close();
  }, [noticeOpen]);
  useTheme(state.settings.theme);
  const profile =
    state.settings.profiles.find((p) => p.id === state.settings.activeId) ??
    state.settings.profiles[0];
  const counts = countsFor(profile);
  const test = async () => {
    setTesting(true);
    await command('test_flick');
    window.setTimeout(() => setTesting(false), 450);
  };
  return (
    <div className="app-shell">
      <Titlebar />
      <main aria-busy={!ready}>
        <section className="status-header" aria-label="Flick status">
          <div className={`app-symbol ${state.enabled ? 'active' : ''}`}>
            <CornerUpRight size={27} strokeWidth={1.7} />
          </div>
          <div className="status-copy">
            <h1>
              {state.enabled ? 'Enabled' : 'Paused'}
              <span className={`status-dot ${state.enabled ? 'active' : ''}`} />
            </h1>
            <button
              className="profile-link"
              onClick={() => setSettingsOpen(true)}
            >
              {profile.name}
              <ChevronRight size={12} />
            </button>
          </div>
          <Toggle
            label="Enable Flick"
            disabled={!ready || !state.healthy}
            checked={state.enabled}
            onChange={(enabled) => {
              if (enabled && !state.settings.riskAcknowledged) {
                setNoticeOpen(true);
              } else {
                void command('set_enabled', { enabled });
              }
            }}
          />
        </section>
        <p className="intro">One button. A complete half turn.</p>
        <section
          className="card input-card"
          aria-label="Trigger and sensitivity"
        >
          <Row icon={<Mouse size={18} />} label="Trigger">
            <button
              className={`capture-button ${state.recording ? 'recording' : ''}`}
              onClick={() =>
                void command(
                  state.recording ? 'cancel_capture' : 'record_trigger',
                )
              }
              aria-label={
                state.recording ? 'Cancel recording' : 'Click to record trigger'
              }
            >
              <span>
                {state.recording
                  ? 'Press a key or button…'
                  : profile.triggerLabel}
              </span>
              {!state.recording && <span className="capture-hint">Record</span>}
            </button>
          </Row>
          {profile.mode === 'game' ? (
            <>
              <Row icon={<Gamepad2 size={18} />} label="Game">
                <select
                  aria-label="Game"
                  value={profile.game}
                  onChange={(e) => updateProfile({ game: e.target.value })}
                >
                  {presets.map((p) => (
                    <option value={p.id} key={p.id}>
                      {p.name}
                    </option>
                  ))}
                </select>
              </Row>
              <Row label="Sensitivity" className="indented">
                <NumberField
                  label="Sensitivity"
                  step={0.01}
                  value={profile.sensitivity}
                  onChange={(sensitivity) => updateProfile({ sensitivity })}
                  stepper
                />
              </Row>
            </>
          ) : profile.mode === 'distance' ? (
            <>
              <Row icon={<Gamepad2 size={18} />} label="cm per 360°">
                <NumberField
                  label="Centimeters per 360 degrees"
                  value={profile.cm360}
                  onChange={(cm360) => updateProfile({ cm360 })}
                />
              </Row>
              <Row label="Mouse DPI" className="indented">
                <NumberField
                  label="Mouse DPI"
                  min={1}
                  value={profile.dpi}
                  onChange={(dpi) => updateProfile({ dpi })}
                  stepper
                  step={100}
                />
              </Row>
            </>
          ) : (
            <>
              <Row icon={<Gamepad2 size={18} />} label="Calibration">
                <button
                  className="text-button"
                  onClick={() => setSettingsOpen(true)}
                >
                  Raw counts
                  <ChevronRight size={13} />
                </button>
              </Row>
              <Row label="Counts per 180°" className="indented">
                <NumberField
                  label="Raw counts"
                  value={profile.rawCounts}
                  min={1}
                  step={1}
                  onChange={(rawCounts) => updateProfile({ rawCounts })}
                  stepper
                />
              </Row>
            </>
          )}
        </section>
        <section className="card motion-card" aria-label="Turn motion">
          <div className="speed-control">
            <div className="speed-heading">
              <label htmlFor="speed">Speed</label>
              <output htmlFor="speed">
                {profile.duration}
                <span> ms</span>
              </output>
            </div>
            <div
              className="range-track"
              style={
                { '--range': `${profile.duration}%` } as React.CSSProperties
              }
            >
              <div className="range-rail" aria-hidden="true" />
              <input
                id="speed"
                aria-label="Turn duration"
                type="range"
                min={0}
                max={100}
                step={1}
                value={profile.duration}
                onChange={(e) =>
                  updateProfile({ duration: Number(e.target.value) })
                }
              />
            </div>
            <div className="range-labels">
              <span>Instant</span>
              <span>Smooth</span>
            </div>
          </div>
          <Row label="Direction">
            <div className="segmented" role="group" aria-label="Direction">
              <button
                aria-pressed={profile.direction === 'left'}
                onClick={() => updateProfile({ direction: 'left' })}
              >
                <ArrowLeft size={15} />
                Left
              </button>
              <button
                aria-pressed={profile.direction === 'right'}
                onClick={() => updateProfile({ direction: 'right' })}
              >
                Right
                <ArrowRight size={15} />
              </button>
            </div>
          </Row>
        </section>
        <div className="readout" aria-live="polite">
          <span className="angle">180°</span>
          <span className="equals">=</span>
          <span className="count">
            {counts ? counts.toLocaleString('en-US') : '—'}
            <span> counts</span>
          </span>
          <button
            className="info-button"
            aria-label="About count accuracy"
            aria-expanded={tip}
            onClick={() => setTip(!tip)}
          >
            <Info size={14} />
          </button>
          {tip && (
            <div className="accuracy-tip" role="status">
              {counts
                ? 'Rounded for display. Fractional counts carry over between turns.'
                : 'These settings exceed the supported count range.'}
            </div>
          )}
        </div>
        <button
          className={`test-button ${testing ? 'testing' : ''}`}
          disabled={
            !ready ||
            !state.enabled ||
            !counts ||
            testing ||
            state.calibration !== 'idle'
          }
          onClick={() => void test()}
        >
          <CornerUpRight size={18} />
          {testing ? 'Turning…' : 'Test flick'}
          <ArrowUpRight size={15} className="test-arrow" />
        </button>
        <div className="test-note">
          {state.recording
            ? 'Listening · Esc to cancel'
            : state.calibration === 'armed'
              ? 'Calibration ready · Press F9 in game'
              : state.calibration === 'measuring'
                ? 'Measuring your turn · F9 to save'
                : state.enabled
                  ? 'Your trigger is ready, even with Flick in the tray.'
                  : 'Enable Flick when you’re ready to turn.'}
        </div>
        <footer>
          <button
            className="settings-button"
            onClick={() => setSettingsOpen(true)}
          >
            <Settings2 size={16} />
            Settings
          </button>
          <button className="risk-link" onClick={() => setNoticeOpen(true)}>
            Account risk
          </button>
          <span className="shortcut">
            <kbd>F8</kbd> {state.enabled ? 'to pause' : 'to enable'}
          </span>
        </footer>
      </main>
      <dialog
        ref={noticeDialog}
        className="risk-dialog"
        aria-labelledby="risk-title"
        onCancel={() => setNoticeOpen(false)}
      >
        <div className="risk-icon">
          <Info size={23} />
        </div>
        <h2 id="risk-title">Before you enable Flick</h2>
        <p>
          Flick turns your view with one press. Online games may treat this as
          prohibited input automation and suspend or permanently ban your
          account. Using standard Windows mouse input does not make it safe
          under a game’s rules.
        </p>
        <p>
          Check the current rules for each game and use Flick only where input
          automation is allowed. A game preset is a conversion aid, not approval
          from its publisher.
        </p>
        <div className="risk-actions">
          <button onClick={() => setNoticeOpen(false)}>Keep paused</button>
          <button
            className="risk-confirm"
            onClick={() => {
              if (!state.settings.riskAcknowledged)
                updateSettings({ riskAcknowledged: true });
              setNoticeOpen(false);
            }}
          >
            {state.settings.riskAcknowledged
              ? 'Got it'
              : 'I understand the risk'}
          </button>
        </div>
      </dialog>
      {error && !settingsOpen && (
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
      {!error &&
        state.environment.foregroundElevated &&
        !state.environment.elevated && (
          <div className="toast" role="status">
            <Info size={17} />
            <span>This game needs administrator access.</span>
            <button
              className="text-button"
              onClick={() => void command('run_as_admin')}
            >
              Restart
            </button>
          </div>
        )}
      {!error && state.environment.acceleration && !settingsOpen && (
        <button
          className="environment-note"
          onClick={() => setSettingsOpen(true)}
        >
          For consistent turns, turn off “Enhance pointer precision”.
        </button>
      )}
      {settingsOpen && (
        <SettingsSheet
          error={error}
          dismissError={dismissError}
          state={state}
          close={() => setSettingsOpen(false)}
          updateSettings={updateSettings}
          updateProfile={updateProfile}
          command={command}
        />
      )}
      {!native && (
        <span className="sr-only">
          Browser preview. Global input is available in the Windows app.
        </span>
      )}
    </div>
  );
}
