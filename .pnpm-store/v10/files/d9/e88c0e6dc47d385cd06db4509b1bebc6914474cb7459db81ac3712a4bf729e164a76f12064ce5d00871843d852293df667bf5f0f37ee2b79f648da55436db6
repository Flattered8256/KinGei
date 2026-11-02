import { defineCachedEventHandler, defineCachedFunction } from "nitropack/runtime";
import { createError, defineEventHandler, getRouterParam } from "h3";
import { initializeI18nContext, tryUseI18nContext, useI18nContext } from "../context.js";
import { warnMissedMessageFunctions } from "../../shared/messages.js";
const _messagesHandler = defineEventHandler(async (event) => {
  const locale = getRouterParam(event, "locale");
  if (!locale) {
    throw createError({ status: 400, message: "Locale not specified." });
  }
  const ctx = useI18nContext(event);
  if (ctx.localeConfigs && locale in ctx.localeConfigs === false) {
    throw createError({ status: 404, message: `Locale '${locale}' not found.` });
  }
  const messages = await ctx.loadMessages(locale);
  if (import.meta.dev || import.meta.prerender) {
    for (const k of Object.keys(messages)) {
      warnMissedMessageFunctions(k, messages[k]);
    }
  }
  return messages;
});
const getCacheKey = (event) => [getRouterParam(event, "locale") ?? "null", getRouterParam(event, "hash") ?? "null"].join("-");
async function shouldBypassCache(event) {
  const locale = getRouterParam(event, "locale");
  if (locale == null) {
    return false;
  }
  const ctx = tryUseI18nContext(event) || await initializeI18nContext(event);
  return !ctx.localeConfigs?.[locale]?.cacheable;
}
const _cachedMessageLoader = defineCachedFunction(_messagesHandler, {
  name: "i18n:messages-internal",
  maxAge: !__I18N_CACHE__ ? -1 : 60 * 60 * 24,
  getKey: getCacheKey,
  shouldBypassCache
});
const _messagesHandlerCached = defineCachedEventHandler(_cachedMessageLoader, {
  name: "i18n:messages",
  maxAge: !__I18N_CACHE__ ? -1 : __I18N_HTTP_CACHE_DURATION__,
  swr: false,
  getKey: getCacheKey,
  shouldBypassCache
});
export default import.meta.dev ? _messagesHandler : _messagesHandlerCached;
