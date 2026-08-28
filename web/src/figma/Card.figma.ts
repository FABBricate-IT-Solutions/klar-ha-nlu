// url=https://www.figma.com/design/9MMrHhjmSMA3rwL7oJ5lxv/Operator-v2?node-id=6-21
// source=web/src/theme.css
// component=div.card
import figma from "figma";

const instance = figma.selectedInstance;
const title = instance.getString("Title");
const body = instance.getString("Body");
const hot = instance.getEnum("Hot", {
  True: true,
  False: false,
});

export default {
  example: figma.code`
    <div className="card${hot ? " hot" : ""}">
      <h2>${title}</h2>
      <p className="muted">${body}</p>
    </div>
  `,
  id: "klar-card",
  metadata: { nestable: true },
};
