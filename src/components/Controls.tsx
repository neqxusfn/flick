import { useEffect, useId, useState, type ReactNode } from 'react';
import { Minus, Plus } from 'lucide-react';
export function Toggle({
  checked,
  onChange,
  label,
  disabled = false,
}: {
  checked: boolean;
  onChange: (value: boolean) => void;
  label: string;
  disabled?: boolean;
}) {
  return (
    <button
      className="toggle"
      type="button"
      role="switch"
      aria-checked={checked}
      aria-label={label}
      disabled={disabled}
      onClick={() => onChange(!checked)}
    >
      <span />
    </button>
  );
}
export function Row({
  icon,
  label,
  children,
  className = '',
}: {
  icon?: ReactNode;
  label: string;
  children: ReactNode;
  className?: string;
}) {
  return (
    <div className={`row ${className}`}>
      <span className="row-label">
        {icon}
        {label}
      </span>
      <div className="row-control">{children}</div>
    </div>
  );
}
export function NumberField({
  label,
  value,
  onChange,
  step = 0.1,
  min = 0.001,
  max = 1_000_000,
  stepper = false,
}: {
  label: string;
  value: number;
  onChange: (value: number) => void;
  step?: number;
  min?: number;
  max?: number;
  stepper?: boolean;
}) {
  const [draft, setDraft] = useState(String(value));
  const [invalid, setInvalid] = useState(false);
  const id = useId();
  useEffect(() => {
    setDraft(String(value));
    setInvalid(false);
  }, [value]);
  const commit = () => {
    const parsed = Number(draft);
    if (
      !draft.trim() ||
      !Number.isFinite(parsed) ||
      parsed < min ||
      parsed > max
    ) {
      setInvalid(true);
      return;
    }
    setInvalid(false);
    if (parsed !== value) onChange(parsed);
  };
  const adjust = (direction: number) => {
    const next = Math.max(
      min,
      Math.min(max, Number((value + direction * step).toFixed(6))),
    );
    setDraft(String(next));
    setInvalid(false);
    onChange(next);
  };
  return (
    <div className="number-wrap">
      <div className={`number-field ${invalid ? 'invalid' : ''}`}>
        <input
          aria-label={label}
          aria-invalid={invalid}
          aria-describedby={invalid ? id : undefined}
          inputMode="decimal"
          type="text"
          value={draft}
          onChange={(e) => setDraft(e.target.value)}
          onBlur={commit}
          onKeyDown={(e) => {
            if (e.key === 'Enter') {
              commit();
              e.currentTarget.blur();
            }
            if (e.key === 'Escape') {
              setDraft(String(value));
              setInvalid(false);
            }
          }}
        />
        {stepper && (
          <div className="stepper">
            <button
              aria-label={`Decrease ${label.toLowerCase()}`}
              onClick={() => adjust(-1)}
            >
              <Minus size={12} />
            </button>
            <button
              aria-label={`Increase ${label.toLowerCase()}`}
              onClick={() => adjust(1)}
            >
              <Plus size={12} />
            </button>
          </div>
        )}
      </div>
      {invalid && (
        <span className="field-error" id={id}>
          Enter {min}–{max}.
        </span>
      )}
    </div>
  );
}
