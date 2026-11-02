import { withBase, withQuery } from "ufo";
export function localeHead(options, currentLanguage = options.getCurrentLanguage(), currentDirection = options.getCurrentDirection()) {
  const metaObject = {
    htmlAttrs: {},
    link: [],
    meta: []
  };
  if (options.dir) {
    metaObject.htmlAttrs.dir = currentDirection;
  }
  if (options.lang && currentLanguage) {
    metaObject.htmlAttrs.lang = currentLanguage;
  }
  if (options.seo) {
    const routeWithoutQuery = options.strictCanonicals ? options.getRouteWithoutQuery() : void 0;
    const alternateLinks = getHreflangLinks(options, routeWithoutQuery);
    const canonical = getCanonicalUrl(options, routeWithoutQuery);
    metaObject.link = metaObject.link.concat(
      alternateLinks,
      getCanonicalLink(options, canonical)
    );
    metaObject.meta = metaObject.meta.concat(
      getOgUrl(options, canonical),
      getCurrentOgLocale(options),
      getAlternateOgLocales(options, getAlternateOgLanguages(options, alternateLinks))
    );
  }
  return metaObject;
}
function createLocaleMap(locales) {
  const localeMap = /* @__PURE__ */ new Map();
  for (const locale of locales) {
    if (!locale.language) {
      console.warn("Locale `language` ISO code is required to generate alternate link");
      continue;
    }
    const [language, region] = locale.language.split("-");
    if (language && region && (locale.isCatchallLocale || !localeMap.has(language))) {
      localeMap.set(language, locale);
    }
    localeMap.set(locale.language, locale);
  }
  return localeMap;
}
function getHreflangLinks(options, routeWithoutQuery = options.strictCanonicals ? options.getRouteWithoutQuery() : void 0) {
  if (!options.hreflangLinks) {
    return [];
  }
  const links = [];
  const localeMap = createLocaleMap(options.locales);
  const hrefs = /* @__PURE__ */ new Map();
  for (const [language, locale] of localeMap.entries()) {
    const href = hrefs.get(locale.code) ?? resolveAlternateHref(locale.code, options, routeWithoutQuery);
    if (!href) {
      continue;
    }
    hrefs.set(locale.code, href);
    const link = options.strictSeo ? { rel: "alternate", href, hreflang: language } : { [options.key]: `i18n-alt-${language}`, rel: "alternate", href, hreflang: language };
    links.push(link);
    if (options.defaultLocale && options.defaultLocale === locale.code && links[0].hreflang !== "x-default") {
      links.unshift(
        options.strictSeo ? { rel: "alternate", href, hreflang: "x-default" } : { [options.key]: "i18n-xd", rel: "alternate", href, hreflang: "x-default" }
      );
    }
  }
  return links;
}
function resolveAlternateHref(locale, options, routeWithoutQuery = options.strictCanonicals ? options.getRouteWithoutQuery() : void 0) {
  const localePath = options.getLocalizedRoute(locale, routeWithoutQuery);
  if (!localePath) {
    return void 0;
  }
  return withQuery(
    withBase(localePath, options.baseUrl),
    options.strictCanonicals ? getCanonicalQueryParams(options) : {}
  );
}
function getCanonicalUrl(options, routeWithoutQuery = options.strictCanonicals ? options.getRouteWithoutQuery() : void 0) {
  const localePath = options.getLocalizedRoute(options.currentLocale, routeWithoutQuery);
  if (!localePath) {
    return "";
  }
  return withQuery(withBase(localePath, options.baseUrl), getCanonicalQueryParams(options));
}
function getCanonicalLink(options, href = getCanonicalUrl(options)) {
  if (!href) {
    return [];
  }
  return [options.strictSeo ? { rel: "canonical", href } : { [options.key]: "i18n-can", rel: "canonical", href }];
}
function getCanonicalQueryParams(options, route = options.getCurrentRoute()) {
  if (!options.canonicalQueries.length) {
    return {};
  }
  const currentRoute = options.getLocaleRoute(
    Object.assign({}, route, { path: void 0, name: options.getRouteBaseName(route) })
  );
  const currentRouteQuery = currentRoute?.query ?? {};
  const params = {};
  for (const param of options.canonicalQueries.filter((x) => x in currentRouteQuery)) {
    params[param] ??= [];
    for (const val of toArray(currentRouteQuery[param])) {
      params[param].push(val || "");
    }
  }
  return params;
}
function getOgUrl(options, href = getCanonicalUrl(options)) {
  if (!href) {
    return [];
  }
  return [
    options.strictSeo ? { property: "og:url", content: href } : { [options.key]: "i18n-og-url", property: "og:url", content: href }
  ];
}
function getCurrentOgLocale(options, currentLanguage = options.getCurrentLanguage()) {
  if (!currentLanguage) {
    return [];
  }
  return [
    options.strictSeo ? { property: "og:locale", content: formatOgLanguage(currentLanguage) } : { [options.key]: "i18n-og", property: "og:locale", content: formatOgLanguage(currentLanguage) }
  ];
}
function getAlternateOgLanguages(options, alternateLinks) {
  const languages = options.locales.map((x) => x.language || x.code);
  if (!options.strictSeo) {
    return languages;
  }
  const available = new Set(alternateLinks.map((x) => x.hreflang));
  return languages.filter((x) => available.has(x));
}
function getAlternateOgLocales(options, languages, currentLanguage = options.getCurrentLanguage()) {
  const alternateLocales = languages.filter((locale) => locale && locale !== currentLanguage);
  return alternateLocales.map(
    (locale) => options.strictSeo ? {
      property: "og:locale:alternate",
      content: formatOgLanguage(locale)
    } : {
      [options.key]: `i18n-og-alt-${locale}`,
      property: "og:locale:alternate",
      content: formatOgLanguage(locale)
    }
  );
}
function formatOgLanguage(val = "") {
  return val.replace(/-/g, "_");
}
function toArray(value) {
  return Array.isArray(value) ? value : [value];
}
