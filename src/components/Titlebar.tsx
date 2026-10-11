import { CornerUpRight, Minus, X } from 'lucide-react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { native } from '../hooks/useFlick';
export function Titlebar() {
  return (
    <header className="titlebar" data-tauri-drag-region>
      <span className="wordmark" data-tauri-drag-region>
        <CornerUpRight size={15} /> flick{' '}
        <span className="edition">{native ? 'DESKTOP' : 'PREVIEW'}</span>
      </span>
      <div className="window-controls">
        <button
          className="window-button minimize"
          aria-label="Minimize window"
          disabled={!native}
          onClick={() => void getCurrentWindow().minimize()}
        >
          <Minus size={10} />
        </button>
        <button
          className="window-button close"
          aria-label="Close to tray"
          disabled={!native}
          onClick={() => void getCurrentWindow().hide()}
        >
          <X size={10} />
        </button>
      </div>
    </header>
  );
}
