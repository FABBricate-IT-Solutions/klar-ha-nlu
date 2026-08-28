// url=https://www.figma.com/design/9MMrHhjmSMA3rwL7oJ5lxv/Operator-v2?node-id=6-52
// source=web/src/components/SearchSelect.tsx
// component=SearchSelect
import figma from "figma";

const instance = figma.selectedInstance;
const value = instance.getString("Value");
const emptyLabel = instance.getString("EmptyLabel");
const placeholder = instance.getString("Placeholder");

export default {
  example: figma.code`
    <SearchSelect
      value="${value}"
      options={options}
      onChange={onChange}
      placeholder="${placeholder}"
      emptyLabel="${emptyLabel}"
    />
  `,
  imports: ['import { SearchSelect } from "../components/SearchSelect"'],
  id: "klar-search-select",
  metadata: { nestable: true },
};
