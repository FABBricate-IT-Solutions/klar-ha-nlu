// url=https://www.figma.com/design/9MMrHhjmSMA3rwL7oJ5lxv/Operator-v2?node-id=6-37
// source=web/src/components/Snackbar.tsx
// component=Snackbar
import figma from "figma";

const instance = figma.selectedInstance;
const message = instance.getString("Message");
const dismissLabel = instance.getString("DismissLabel");
const tone = instance.getEnum("Tone", {
  Default: "default",
  Danger: "danger",
});

export default {
  example: figma.code`
    <Snackbar
      message="${message}"
      dismissLabel="${dismissLabel}"
      tone="${tone}"
      onDismiss={onDismiss}
    />
  `,
  imports: ['import { Snackbar } from "../components/Snackbar"'],
  id: "klar-snackbar",
  metadata: { nestable: false },
};
