// url=https://www.figma.com/design/9MMrHhjmSMA3rwL7oJ5lxv/Operator-v2?node-id=6-28
// source=web/src/App.tsx
// component=nav.rail a
import figma from "figma";

const instance = figma.selectedInstance;
const label = instance.getString("Label");
const active = instance.getEnum("Active", {
  True: true,
  False: false,
});

export default {
  example: figma.code`
    <a
      className="${active ? "active" : ""}"
      ${active ? 'aria-current="page"' : ""}
    >
      ${label}
    </a>
  `,
  id: "klar-rail-item",
  metadata: { nestable: true },
};
