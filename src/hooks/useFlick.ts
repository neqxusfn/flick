import { useCallback, useEffect, useRef, useState } from 'react';
import { invoke, isTauri } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import {
  initialState,
  type Profile,
  type Settings,
  type Snapshot,
} from '../types';
export const native = isTauri();
const message = (error: unknown) =>
  error instanceof Error ? error.message : String(error);
export function useFlick() {
  const [state, setState] = useState<Snapshot>(initialState);
  const [ready, setReady] = useState(!native);
  const [error, setError] = useState<string | null>(null);
  const current = useRef(state);
  const pending = useRef(0);
  const queue = useRef(Promise.resolve());
  const accept = useCallback((snapshot: Snapshot) => {
    const next = pending.current
      ? { ...snapshot, settings: current.current.settings }
      : snapshot;
    current.current = next;
    setState(next);
  }, []);
  useEffect(() => {
    if (!native) return;
    let disposed = false;
    let unlisten: (() => void) | undefined;
    void (async () => {
      unlisten = await listen<Snapshot>('flick-state', (event) => {
        if (!disposed) accept(event.payload);
      });
      if (disposed) {
        unlisten();
        return;
      }
      const snapshot = await invoke<Snapshot>('get_state');
      if (!disposed) {
        accept(snapshot);
        setReady(true);
      }
    })().catch((e: unknown) => {
      if (!disposed) setError(message(e));
    });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [accept]);
  const save = useCallback(
    (settings: Settings) => {
      current.current = { ...current.current, settings };
      setState(current.current);
      if (!native) return;
      pending.current++;
      queue.current = queue.current.then(async () => {
        try {
          // Coalesce queued slider changes; the newest snapshot includes all edits.
          if (pending.current === 1)
            await invoke('save_settings', { settings });
        } catch (e) {
          setError(message(e));
        } finally {
          pending.current--;
          if (!pending.current) {
            try {
              accept(await invoke<Snapshot>('get_state'));
            } catch (e) {
              setError(message(e));
            }
          }
        }
      });
    },
    [accept],
  );
  const updateSettings = useCallback(
    (patch: Partial<Settings>) =>
      save({ ...current.current.settings, ...patch }),
    [save],
  );
  const updateProfile = useCallback(
    (patch: Partial<Profile>) => {
      const settings = current.current.settings;
      save({
        ...settings,
        profiles: settings.profiles.map((p) =>
          p.id === settings.activeId ? { ...p, ...patch } : p,
        ),
      });
    },
    [save],
  );
  const command = useCallback(
    async (name: string, args?: Record<string, unknown>) => {
      setError(null);
      if (!native) {
        if (name === 'set_enabled') {
          if (args?.enabled && !current.current.settings.riskAcknowledged)
            setError(
              'Read and acknowledge the account-risk warning before enabling Flick.',
            );
          else accept({ ...current.current, enabled: Boolean(args?.enabled) });
        } else if (name === 'clear_error')
          accept({ ...current.current, error: null });
        else
          setError('Open the Windows app to use global input and calibration.');
        return;
      }
      try {
        await queue.current;
        await invoke(name, args);
      } catch (e) {
        setError(message(e));
      }
    },
    [accept],
  );
  const dismissError = () => {
    setError(null);
    void command('clear_error');
  };
  return {
    state,
    ready,
    error: error ?? state.error,
    updateSettings,
    updateProfile,
    command,
    dismissError,
  };
}
