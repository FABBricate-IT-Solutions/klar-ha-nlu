// url=https://www.figma.com/design/9MMrHhjmSMA3rwL7oJ5lxv/Operator-v2?node-id=6-89
// source=web/src/pages/Wizard.tsx
// component=Wizard
import figma from "figma";

const instance = figma.selectedInstance;
instance.getEnum("Step", {
  "1": "1",
  "2": "2",
  "3": "3",
  "4": "4",
  "5": "5",
  "6": "6",
});
instance.getString("Title");
instance.getString("Caption");
instance.getString("Body");

export default {
  example: figma.code`
    <Wizard
      open
      onClose={onClose}
      onDone={onDone}
    />
  `,
  imports: ['import { Wizard } from "../pages/Wizard"'],
  id: "klar-wizard",
  metadata: { nestable: false },
};
