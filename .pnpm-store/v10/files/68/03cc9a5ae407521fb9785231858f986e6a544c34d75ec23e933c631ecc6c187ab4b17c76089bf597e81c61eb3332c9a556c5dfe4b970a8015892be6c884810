import { assign, create, deepCopy, isFunction, toTypeString } from "@intlify/shared";
import { useNuxtApp } from "#app";
import { createDefu } from "defu";
const cacheMessages = /* @__PURE__ */ new Map();
export function cloneDeep(value) {
  if (value == null || typeof value !== "object") {
    return value;
  }
  if (Array.isArray(value)) {
    return value.map(cloneDeep);
  }
  const out = create(null);
  for (const key of Object.keys(value)) {
    out[key] = cloneDeep(value[key]);
  }
  return out;
}
export function hasMessageFunction(value, seen = /* @__PURE__ */ new WeakSet()) {
  if (isFunction(value)) {
    return true;
  }
  if (value == null || typeof value !== "object") {
    return false;
  }
  if (seen.has(value)) {
    return false;
  }
  seen.add(value);
  return Object.values(value).some((x) => hasMessageFunction(x, seen));
}
export function warnMissedMessageFunctions(locale, messages) {
  const undeliverable = __I18N_UNDELIVERABLE_LOCALES__;
  if (undeliverable.includes(locale) || !hasMessageFunction(messages)) {
    return;
  }
  console.warn(
    `[nuxt-i18n] Messages for locale "${locale}" contain message functions the build did not detect - they are dropped when messages are delivered as JSON. Write message functions literally in a locale file to make them detectable.`
  );
}
const isTree = (value) => value != null && typeof value === "object" && !Array.isArray(value);
export function fillMissing(over, base) {
  return isTree(over) && isTree(base) ? fillTree(over, base) : over;
}
function fillTree(over, base) {
  let out = over;
  for (const key of Object.keys(base)) {
    const has = Object.hasOwn(over, key);
    const current = over[key];
    const filled = !has ? base[key] : isTree(current) && isTree(base[key]) ? fillTree(current, base[key]) : current;
    if (has && filled === current) {
      continue;
    }
    if (out === over) {
      out = assign(create(null), over);
    }
    out[key] = filled;
  }
  return out;
}
const merger = createDefu((obj, key, value) => {
  if (key === "messages" || key === "datetimeFormats" || key === "numberFormats") {
    obj[key] ??= create(null);
    deepCopy(value, obj[key]);
    return true;
  }
});
export async function loadVueI18nOptions(vueI18nConfigs) {
  const nuxtApp = useNuxtApp();
  let vueI18nOptions = { messages: create(null) };
  for (const configFile of vueI18nConfigs) {
    const resolver = await configFile().then((x) => isModule(x) ? x.default : x);
    const resolved = isFunction(resolver) ? await nuxtApp.runWithContext(() => resolver()) : resolver;
    vueI18nOptions = merger(create(null), resolved, vueI18nOptions);
  }
  vueI18nOptions.fallbackLocale ??= false;
  return vueI18nOptions;
}
const isModule = (val) => toTypeString(val) === "[object Module]";
async function getLocaleMessages(locale, loader) {
  const nuxtApp = useNuxtApp();
  try {
    const getter = await nuxtApp.runWithContext(loader.load).then((x) => isModule(x) ? x.default : x);
    return isFunction(getter) ? await nuxtApp.runWithContext(() => getter(locale)) : getter;
  } catch (e) {
    throw new Error(`Failed loading locale (${locale}): ` + e.message, { cause: e });
  }
}
export async function getLocaleMessagesMerged(locale, loaders = []) {
  const nuxtApp = useNuxtApp();
  const messages = await Promise.all(
    loaders.map((loader) => nuxtApp.runWithContext(() => getLocaleMessages(locale, loader)))
  );
  const merged = {};
  for (const message of messages) {
    deepCopy(message, merged);
  }
  return merged;
}
export async function getLocaleMessagesMergedCached(locale, loaders = []) {
  const nuxtApp = useNuxtApp();
  const messages = await Promise.all(loaders.map(async (loader) => {
    const cached = getCachedMessages(loader);
    const messages2 = cached || await nuxtApp.runWithContext(() => getLocaleMessages(locale, loader));
    if (!cached && loader.cache !== false) {
      cacheMessages.set(loader.key, { ttl: Date.now() + __I18N_CACHE_LIFETIME__ * 1e3, value: messages2 });
    }
    return messages2;
  }));
  const merged = {};
  for (const message of messages) {
    deepCopy(message, merged);
  }
  return merged;
}
function getCachedMessages(loader) {
  if (!__I18N_CACHE__) {
    return;
  }
  if (loader.cache === false) {
    return;
  }
  const cache = cacheMessages.get(loader.key);
  if (cache == null) {
    return;
  }
  return __I18N_CACHE_LIFETIME__ === 0 || cache.ttl > Date.now() ? cache.value : void 0;
}
