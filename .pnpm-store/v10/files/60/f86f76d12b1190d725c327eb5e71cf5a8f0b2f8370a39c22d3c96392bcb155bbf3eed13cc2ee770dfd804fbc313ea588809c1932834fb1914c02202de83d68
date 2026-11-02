import { hasProtocol } from "ufo";
import { normalizedLocales } from "#build/i18n-options.mjs";
export const normalizeDomain = (domain = "") => domain.replace(/^https?:\/\//i, "").toLowerCase();
export function isLocaleOnHost(locale, host) {
  return !!locale?.domains.some((x) => normalizeDomain(x) === host);
}
export function resolveLocaleReach(locales, host, locale) {
  const target = locales.find((l) => l.code === locale);
  if (!target?.domains.length || isLocaleOnHost(target, host)) {
    return "here";
  }
  return locales.some((l) => isLocaleOnHost(l, host)) ? "other-domain" : "off-host";
}
export function isLocaleServedOnHost(locales, host, locale) {
  return resolveLocaleReach(locales, host, locale) !== "other-domain";
}
export const canonicalDomain = (target) => target?.defaultForDomains[0] || target?.domain || target?.domains[0];
function relocateHostForLocale(host, locale, locales) {
  if (isLocaleServedOnHost(locales, host, locale)) {
    return;
  }
  return canonicalDomain(locales.find((l) => l.code === locale));
}
export function matchDomainLocale(locales, host, pathLocale) {
  const matches = locales.filter((locale) => isLocaleOnHost(locale, host));
  return (
    // match by current path locale
    (matches.find((l) => l.code === pathLocale) || matches.find((l) => l.defaultForDomains.some((domain) => normalizeDomain(domain) === host)) || matches[0])?.code
  );
}
export function cookieSpansDomains(locales, cookieDomain) {
  const scope = cookieDomain.replace(/^\./, "").replace(/:\d+$/, "").toLowerCase();
  return locales.every(
    (l) => l.domains.concat(l.domain || []).every((domain) => {
      const host = normalizeDomain(domain).replace(/:\d+$/, "");
      return host === scope || host.endsWith("." + scope);
    })
  );
}
const withProtocol = (domain, url) => hasProtocol(domain, { strict: true }) ? domain : url.protocol + "//" + domain;
export function domainFromLocale(domainLocales, url, locale, locales = normalizedLocales) {
  const lang = locales.find((x) => x.code === locale);
  const domain = domainLocales?.[locale]?.domain || lang?.domain || lang?.domains.find((v) => normalizeDomain(v) === url.host) || relocateHostForLocale(url.host, locale, locales);
  if (!domain) {
    import.meta.dev && console.warn("[nuxt-i18n] Could not find domain name for locale " + locale);
    return;
  }
  return withProtocol(domain, url);
}
export function canonicalDomainFromLocale(domainLocales, url, locale, fallbackLocale, locales = normalizedLocales) {
  const forLocale = (code) => domainLocales?.[code]?.domain || canonicalDomain(locales.find((x) => x.code === code));
  const domain = forLocale(locale) || (fallbackLocale ? forLocale(fallbackLocale) : void 0);
  return domain ? withProtocol(domain, url) : void 0;
}
export function domainForHost(domainLocales, url, locales = normalizedLocales) {
  let match;
  for (const locale of locales) {
    const patched = withRuntimeDomain(locale, domainLocales);
    for (const candidate of [patched.domain, ...patched.domains]) {
      if (!candidate || normalizeDomain(candidate) !== url.host) {
        continue;
      }
      if (!match || !hasProtocol(match, { strict: true }) && hasProtocol(candidate, { strict: true })) {
        match = candidate;
      }
    }
  }
  return match && withProtocol(match, url);
}
export function withRuntimeDomain(locale, domainLocales) {
  if (typeof locale === "string") {
    return locale;
  }
  const properties = locale;
  const domain = domainLocales[properties.code]?.domain;
  if (!domain || domain === properties.domain) {
    return locale;
  }
  return {
    ...properties,
    domain,
    domains: [domain],
    defaultForDomains: properties.defaultForDomains.length ? [domain] : []
  };
}
