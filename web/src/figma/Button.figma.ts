// url=https://www.figma.com/design/9MMrHhjmSMA3rwL7oJ5lxv/Operator-v2?node-id=6-12
// source=web/src/theme.css
// component=button.primary | button.secondary | button.ghost
import figma from "figma";

const instance = figma.selectedInstance;
const label = instance.getString("Label");
const variant = instance.getEnum("Variant", {
  Primary: "primary",
  Secondary: "secondary",
  Ghost: "ghost",
});

export default {
  example: figma.code`<button className="${variant}" type="button">${label}</button>`,
  id: "klar-button",
  metadata: { nestable: true },
};
