interface Props {
  value: number;
  min?: number;
  max: number;
  onChange: (value: number) => void;
  label: string;
}

export function QuantityStepper({ value, min = 1, max, onChange, label }: Props) {
  return (
    <div className="stepper" role="group" aria-label={label}>
      <button
        type="button"
        className="stepper__btn"
        aria-label="数量を 1 減らす"
        disabled={value <= min}
        onClick={() => onChange(value - 1)}
      >
        −
      </button>
      <output className="stepper__value" aria-live="polite" aria-label={`数量 ${value}`}>
        {value}
      </output>
      <button
        type="button"
        className="stepper__btn"
        aria-label="数量を 1 増やす"
        disabled={value >= max}
        onClick={() => onChange(value + 1)}
      >
        ＋
      </button>
    </div>
  );
}
