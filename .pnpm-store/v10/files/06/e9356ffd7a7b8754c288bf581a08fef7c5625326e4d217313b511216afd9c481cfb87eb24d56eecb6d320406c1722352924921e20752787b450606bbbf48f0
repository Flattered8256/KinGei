import { resolveTags } from "@unhead/vue/utils";
import { SafeInputPlugin } from "@unhead/vue/plugins";
import { createHead } from "@unhead/vue/server";
//#region src/runtime/utils/island-head.ts
const TAG_CONFIG_KEYS = [
	"key",
	"tagPosition",
	"tagPriority"
];
/**
* Apply the `useHeadSafe` allowlist to resolved head input, returning input that renders
* the same tags when pushed without it.
*/
function toSafeHeadInput(input) {
	const { titleTemplate, ...rest } = input;
	const head = createHead({
		disableDefaults: true,
		plugins: [SafeInputPlugin]
	});
	head.push(rest, { _safe: true });
	const safe = {};
	if (typeof titleTemplate === "string" || titleTemplate === null) safe.titleTemplate = titleTemplate;
	for (const tag of resolveTags(head)) {
		if (tag.tag === "title") {
			safe.title = tag.textContent;
			continue;
		}
		const value = toInputValue(tag);
		if (tag.tag === "htmlAttrs" || tag.tag === "bodyAttrs" || tag.tag === "base") safe[tag.tag] = value;
		else (safe[tag.tag] ||= []).push(value);
	}
	return safe;
}
function toInputValue(tag) {
	const value = {};
	for (const [name, prop] of Object.entries(tag.props)) value[name] = prop instanceof Set ? [...prop].join(" ") : prop instanceof Map ? [...prop].map(([property, propertyValue]) => `${property}:${propertyValue}`).join(";") : prop;
	if (tag.textContent !== void 0) value.textContent = tag.textContent;
	for (const key of TAG_CONFIG_KEYS) if (tag[key] !== void 0) value[key] = tag[key];
	return value;
}
//#endregion
export { toSafeHeadInput };
