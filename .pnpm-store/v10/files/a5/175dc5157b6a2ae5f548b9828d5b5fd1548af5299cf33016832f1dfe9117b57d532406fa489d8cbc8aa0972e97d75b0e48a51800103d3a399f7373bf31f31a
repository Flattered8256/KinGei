import type { FallbackLocale } from 'vue-i18n';
import type { NormalizedLocaleObject } from '#internal-i18n-types';
type LocaleConfig = {
    cacheable: boolean;
    fallbacks: string[];
};
export declare function createLocaleConfigs(fallbackLocale: FallbackLocale): Record<string, LocaleConfig>;
/**
 * Check if the loaders for the specified locale are all cacheable
 */
export declare function isLocaleCacheable(locale: string): any;
/**
 * Check if the loaders for the specified locale and fallback locales are all cacheable
 */
export declare function isLocaleWithFallbacksCacheable(locale: string, fallbackLocales: string[]): any;
/**
 * The locale configured as the default for `host`, undefined when no locale claims it
 */
export declare function getDefaultLocaleForDomain(host: string, locales?: NormalizedLocaleObject[]): string | undefined;
/**
 * The unprefixed locale for `host`: the locale that domain is the default for, falling back to the
 * configured `defaultLocale` where the host claims none. Route generation, the routing context and
 * the server all have to agree on this, so they resolve it here rather than each deriving it.
 */
export declare function resolveDefaultLocale(host: string, defaultLocale: string | undefined, locales?: NormalizedLocaleObject[]): string;
export declare const isSupportedLocale: (locale?: string) => boolean;
export declare const resolveSupportedLocale: (locale: string | undefined) => string | undefined;
export {};
