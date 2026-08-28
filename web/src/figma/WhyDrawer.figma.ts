// url=https://www.figma.com/design/9MMrHhjmSMA3rwL7oJ5lxv/Operator-v2?node-id=7-7
// source=web/src/components/WhyDrawer.tsx
// component=WhyDrawer
import figma from "figma";

const instance = figma.selectedInstance;
instance.getString("Title");
instance.getString("CloseLabel");
instance.getString("Body");

export default {
  example: figma.code`
    <WhyDrawer turn={turn} t={t} onClose={onClose} />
  `,
  imports: ['import { WhyDrawer } from "../components/WhyDrawer"'],
  id: "klar-why-drawer",
  metadata: { nestable: false },
};
