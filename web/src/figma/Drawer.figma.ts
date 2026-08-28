// url=https://www.figma.com/design/9MMrHhjmSMA3rwL7oJ5lxv/Operator-v2?node-id=7-7
// source=web/src/components/common.tsx
// component=Drawer
import figma from "figma";

const instance = figma.selectedInstance;
const title = instance.getString("Title");
const closeLabel = instance.getString("CloseLabel");
const body = instance.getString("Body");

export default {
  example: figma.code`
    <Drawer title="${title}" closeLabel="${closeLabel}" onClose={onClose}>
      <p>${body}</p>
    </Drawer>
  `,
  imports: ['import { Drawer } from "../components/common"'],
  id: "klar-drawer",
  metadata: { nestable: false },
};
