import type { I18nPublicRuntimeConfig, NormalizedLocaleObject } from '#internal-i18n-types';
/**
 * Configured domains may include a protocol (used when generating URLs), comparisons
 * against the request host use the host part only. `ufo` helpers are unsuitable here
 * as they treat the `host:port` shape as a protocol.
 */
export declare const normalizeDomain: (domain?: string) => string;
/**
 * Whether the locale is served on the given host
 */
export declare function isLocaleOnHost(locale: NormalizedLocaleObject | undefined, host: string): boolean;
/**
 * How `host` can reach `locale`:
 * - `here` - served on this host, either because it is one of the locale's domains or because the
 *   locale configures none and is served on all of them
 * - `other-domain` - the locale belongs to domains this host is not one of
 * - `off-host` - same, but this host matches no configured domain at all (a staging domain, a health
 *   check by IP), where the site keeps serving every locale and never sends a visitor to a domain
 */
export declare function resolveLocaleReach(locales: NormalizedLocaleObject[], host: string, locale: string): 'here' | 'other-domain' | 'off-host';
/**
 * Whether the locale can be served on the given host - every {@link resolveLocaleReach} answer but
 * `other-domain`. Shares that resolution so the two cannot disagree.
 */
export declare function isLocaleServedOnHost(locales: NormalizedLocaleObject[], host: string, locale: string): boolean;
/** `defaultForDomains` leads so the domain agrees with the unprefixed shape derived from it */
export declare const canonicalDomain: (target: NormalizedLocaleObject | undefined) => string | undefined;
export declare function matchDomainLocale(locales: NormalizedLocaleObject[], host: string, pathLocale: string): string | undefined;
/**
 * Whether a cookie scoped to `cookieDomain` reaches every configured domain, i.e. each domain
 * equals the scope or is a subdomain of it. Ports are irrelevant to cookies.
 */
export declare function cookieSpansDomains(locales: NormalizedLocaleObject[], cookieDomain: string): boolean;
export declare function domainFromLocale(domainLocales: Record<string, {
    domain: string | undefined;
}>, url: {
    host: string;
    protocol: string;
}, locale: string, locales?: NormalizedLocaleObject[]): string | undefined;
/**
 * Resolves {@link canonicalDomain} for `locale`, unlike {@link domainFromLocale} without a
 * current-host preference - alternate links naming a different URL per host are non-reciprocal.
 * A locale configuring no domain is served on all of them, so it falls back to `fallbackLocale`'s
 * domain (the cluster's own) rather than to whichever host is answering.
 */
export declare function canonicalDomainFromLocale(domainLocales: Record<string, {
    domain: string | undefined;
}>, url: {
    host: string;
    protocol: string;
}, locale: string, fallbackLocale?: string, locales?: NormalizedLocaleObject[]): string | undefined;
/**
 * The configured domain matching the request host, keeping the protocol it was configured with.
 * This answers "where am I", unlike {@link domainFromLocale} which answers "where is this locale
 * served" and may resolve to another domain - deriving the current origin from a locale sends
 * hosts that serve no default locale to whichever domain `defaultLocale` happens to live on.
 */
export declare function domainForHost(domainLocales: I18nPublicRuntimeConfig['domainLocales'], url: {
    host: string;
    protocol: string;
}, locales?: NormalizedLocaleObject[]): string | undefined;
/**
 * Returns the locale object with the domain overridden by `domainLocales` runtime config (see also `getHostLocale`),
 * a no-op outside domain setups since the override entries are empty.
 */
export declare function withRuntimeDomain<T extends string | NormalizedLocaleObject>(locale: T, domainLocales: I18nPublicRuntimeConfig['domainLocales']): T;
