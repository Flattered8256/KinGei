import { parse } from "devalue";
import { unref } from "vue";
import { defineNuxtPlugin, useNuxtApp } from "#app";
import { localeCodes, localeLoaders } from "#build/i18n-options.mjs";
import { getLocaleMessagesMergedCached } from "../shared/messages.js";
import { useNuxtI18nContext } from "../context.js";
export default defineNuxtPlugin({
  name: "i18n:plugin:preload",
  dependsOn: ["i18n:plugin"],
  async setup(_nuxt) {
    if (!__I18N_PRELOAD__) {
      return;
    }
    const nuxt = useNuxtApp(_nuxt._id);
    const ctx = useNuxtI18nContext(nuxt);
    if (import.meta.server) {
      for (const locale of localeCodes) {
        await ctx.loadMessages(locale);
      }
      ctx.preloaded = true;
      const serverI18n = nuxt.ssrContext?.event.context.nuxtI18n;
      if (serverI18n) {
        const msg = unref(ctx.vueI18n.global.messages);
        serverI18n.messages ??= {};
        for (const k in msg) {
          serverI18n.messages[k] = __I18N_UNDELIVERABLE_LOCALES__.includes(k) ? {} : msg[k];
        }
      }
    }
    if (import.meta.client) {
      await mergePayloadMessages(ctx, ctx.vueI18n.global, nuxt);
      if (ctx.preloaded && __I18N_STRIP_UNUSED__) {
        const unsub = nuxt.$router.beforeResolve(async (to, from) => {
          if (to.path === from.path) {
            return;
          }
          await ctx.loadMessages(ctx.getLocale());
          unsub();
        });
      }
    }
  }
});
async function mergePayloadMessages(ctx, i18n, nuxt = useNuxtApp()) {
  const content = document.querySelector(`[data-nuxt-i18n="${nuxt._id}"]`)?.textContent;
  const preloadedMessages = content && parse(content);
  const preloadedKeys = Object.keys(preloadedMessages || {});
  if (!preloadedKeys.length) {
    return;
  }
  for (const locale of preloadedKeys) {
    if (ctx.usesRuntimeLoaders(locale)) {
      continue;
    }
    const messages = preloadedMessages[locale];
    if (messages) {
      i18n.mergeLocaleMessage(locale, messages);
    }
  }
  const reloaded = preloadedKeys.filter((locale) => ctx.usesRuntimeLoaders(locale));
  const results = await Promise.allSettled(reloaded.map((x) => getLocaleMessagesMergedCached(x, localeLoaders[x])));
  results.forEach((result, i) => {
    if (result.status === "fulfilled") {
      i18n.mergeLocaleMessage(reloaded[i], result.value);
    } else {
      console.warn(`Failed to load messages for locale "${reloaded[i]}"`, result.reason);
    }
  });
  ctx.preloaded = true;
}
