// url=https://www.figma.com/design/9MMrHhjmSMA3rwL7oJ5lxv/Operator-v2?node-id=6-12
// source=web/src/theme.css
// component=:root Klar chrome tokens
import figma from "figma";

const instance = figma.selectedInstance;
instance.getEnum("Variant", {
  Primary: "primary",
  Secondary: "secondary",
  Ghost: "ghost",
});

export default {
  example: figma.code`
    :root {
      --bg: #1c1c1c;
      --surface: #242424;
      --surface-2: #2a2a2a;
      --line: #3a3a3a;
      --text: #f4efe6;
      --muted: #9a948a;
      --accent: #c45c26;
      --cyan: #3ec1c8;
      --high: #7d9a6a;
      --medium: #d4a04a;
      --danger: #c76b56;
    }
  `,
  id: "klar-tokens",
  metadata: { nestable: true },
};
