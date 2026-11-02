import type { RouteLocationNormalizedLoadedGeneric, RouteLocationResolvedGeneric } from 'vue-router';
import type { HeadLocale } from './types.js';
import type { AlternateLanguageLink, CanonicalLink, HtmlAttributes, PropertyMeta } from 'unhead/types';
/**
 * I18n header meta info.
 * @internal
 */
export interface I18nHeadMetaInfo {
    htmlAttrs: Pick<HtmlAttributes, 'lang' | 'dir'>;
    meta: PropertyMeta[];
    link: (AlternateLanguageLink | CanonicalLink)[];
}
type SeoAttributesOptions = {
    /**
     * An array of strings corresponding to query params
     * to include in your canonical URL.
     * @default []
     */
    canonicalQueries?: string[];
};
/**
 * @internal
 */
export type HeadContext = {
    key: string;
    strictSeo: boolean;
    dir: boolean;
    lang: boolean;
    seo: boolean | SeoAttributesOptions | undefined;
    baseUrl: string;
    locales: HeadLocale[];
    currentLocale: string;
    defaultLocale: string | undefined;
    hreflangLinks: boolean;
    strictCanonicals: boolean;
    canonicalQueries: string[];
    getCurrentLanguage: () => string | undefined;
    getCurrentDirection: () => 'ltr' | 'rtl' | 'auto';
    getRouteBaseName: (route: RouteLocationResolvedGeneric | RouteLocationNormalizedLoadedGeneric) => string | undefined;
    getLocaleRoute: (route: RouteLocationResolvedGeneric) => RouteLocationResolvedGeneric | undefined;
    getCurrentRoute: () => RouteLocationNormalizedLoadedGeneric;
    getRouteWithoutQuery: () => RouteLocationResolvedGeneric | undefined;
    getLocalizedRoute: (locale: string, route: RouteLocationResolvedGeneric | undefined) => string;
};
/**
 * @internal
 */
export declare function localeHead(options: HeadContext, currentLanguage?: string | undefined, currentDirection?: "auto" | "ltr" | "rtl"): I18nHeadMetaInfo;
export {};
