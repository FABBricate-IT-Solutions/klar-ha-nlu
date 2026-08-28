// url=https://www.figma.com/design/9MMrHhjmSMA3rwL7oJ5lxv/Operator-v2?node-id=7-13
// source=web/src/components/TeachFromMiss.tsx
// component=TeachFromMiss
import figma from "figma";

const instance = figma.selectedInstance;
instance.getString("TeachLabel");
instance.getString("ReplayLabel");

export default {
  example: figma.code`
    <TeachFromMiss heard={heard} t={t} onReplay={onReplay} />
  `,
  imports: ['import { TeachFromMiss } from "../components/TeachFromMiss"'],
  id: "klar-teach-from-miss",
  metadata: { nestable: true },
};
