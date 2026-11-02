import { useStorage } from "nitropack/runtime";
const parsed = /* @__PURE__ */ new Map();
export function readI18nAsset(key) {
  if (!parsed.has(key)) {
    const promise = useStorage("assets/i18n").getItemRaw(key).then((raw) => {
      if (raw == null) {
        throw new Error(`Missing messages asset '${key}' - the server build may be stale, try rebuilding.`);
      }
      return JSON.parse(typeof raw === "string" ? raw : new TextDecoder().decode(raw));
    });
    parsed.set(key, promise);
    promise.catch(() => parsed.delete(key));
  }
  return parsed.get(key);
}
