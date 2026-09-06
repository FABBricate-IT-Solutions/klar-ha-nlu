import type { ReactNode } from "react";
import { fillWizard, type WizardMessages } from "../../i18n/wizard";
import { listReset } from "./styles";

function Line({ children }: { children: ReactNode }) {
  return <li style={{ marginBottom: 10, color: "var(--text)" }}>{children}</li>;
}

function Block({ title, body, hot, tag }: { title: string; body: string; hot?: boolean; tag?: string }) {
  return (
    <div className={`card${hot ? " hot" : ""}`} style={{ marginTop: 12 }}>
      <div className="row" style={{ justifyContent: "space-between" }}>
        <h2>{title}</h2>
        {tag ? <span className={`pill${hot ? " hot" : ""}`}>{tag}</span> : null}
      </div>
      <p className="muted" style={{ margin: "8px 0 0" }}>{body}</p>
    </div>
  );
}

export function TutorialStep({
  copy,
  leftover,
  pathTitle,
  pathBody,
}: {
  copy: WizardMessages;
  leftover: number;
  pathTitle: string;
  pathBody: string;
}) {
  return (
    <>
      <p>{copy.whatLead}</p>
      <ul style={listReset}>
        <Line>{copy.whatLocal}</Line>
        <Line>{copy.whatConsole}</Line>
        <Line>{copy.whatNoLlm}</Line>
      </ul>
      <Block title={pathTitle} body={pathBody} hot tag={copy.detected} />
      <p className="caption">{copy.pathShared}</p>
      <p>{copy.toolsLead}</p>
      <ul style={listReset}>
        <Line>{copy.toolsLab}</Line>
        <Line>{copy.toolsMapping}</Line>
        <Line>{copy.toolsPhrases}</Line>
        <Line>{copy.toolsRoutines}</Line>
        <Line>{copy.toolsPolicies}</Line>
      </ul>
      <p>{copy.phrasesLead}</p>
      <table className="wizard-phrases">
        <thead>
          <tr>
            <th>{copy.phraseSay}</th>
            <th>{copy.phraseExpect}</th>
          </tr>
        </thead>
        <tbody>
          {copy.phrases.map((row) => (
            <tr key={row.say}>
              <td>{row.say}</td>
              <td className="muted">{row.expect}</td>
            </tr>
          ))}
        </tbody>
      </table>
      <p className="caption">{copy.phrasesOther}</p>
      {leftover > 0 ? <p className="caption">{fillWizard(copy.phrasesMapping, { count: String(leftover) })}</p> : null}
      <p className="muted">{copy.phrasesReopen}</p>
    </>
  );
}
