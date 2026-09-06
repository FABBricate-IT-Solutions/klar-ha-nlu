export type GuideStep = {
  id: string;
  label: string;
  hint: string;
};

export function Guide({
  title,
  steps,
  current,
  onPick,
}: {
  title: string;
  steps: GuideStep[];
  current?: string;
  onPick?: (id: string) => void;
}) {
  return (
    <section className="guide" aria-label={title}>
      <p className="guide-title">{title}</p>
      <ol className="guide-steps">
        {steps.map((step, index) => {
          const active = current === step.id;
          return (
            <li key={step.id} className={active ? "active" : undefined}>
              {onPick ? (
                <button type="button" className="guide-step" onClick={() => onPick(step.id)}>
                  <span className="guide-num" aria-hidden="true">{index + 1}</span>
                  <span className="guide-copy">
                    <strong>{step.label}</strong>
                    <span className="muted">{step.hint}</span>
                  </span>
                </button>
              ) : (
                <div className="guide-step">
                  <span className="guide-num" aria-hidden="true">{index + 1}</span>
                  <span className="guide-copy">
                    <strong>{step.label}</strong>
                    <span className="muted">{step.hint}</span>
                  </span>
                </div>
              )}
            </li>
          );
        })}
      </ol>
    </section>
  );
}
