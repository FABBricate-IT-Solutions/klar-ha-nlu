// url=https://www.figma.com/design/9MMrHhjmSMA3rwL7oJ5lxv/Operator-v2?node-id=7-7
// source=web/src/components/InspectDrawer.tsx
// component=InspectDrawer
import figma from "figma";

const instance = figma.selectedInstance;
instance.getString("Title");
instance.getString("CloseLabel");
instance.getString("Body");

export default {
  example: figma.code`
    <InspectDrawer
      row={row}
      rooms={rooms}
      t={t}
      onClose={onClose}
      onSaved={onSaved}
      onDismiss={onDismiss}
    />
  `,
  imports: ['import { InspectDrawer } from "../components/InspectDrawer"'],
  id: "klar-inspect-drawer",
  metadata: { nestable: false },
};
