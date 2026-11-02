import type { NuxtApp } from '#app';
import type { Composer, I18n, Locale } from 'vue-i18n';
import type { ComposableContext } from './composable-context.js';
import type { BaseUrlResolveHandler, I18nPublicRuntimeConfig, NormalizedLocaleObject } from '#internal-i18n-types';
export declare const useLocaleConfigs: () => import("vue").Ref<Record<string, {
    cacheable: boolean;
    fallbacks: string[];
}> | undefined, Record<string, {
    cacheable: boolean;
    fallbacks: string[];
}> | undefined>;
export declare const useResolvedLocale: () => import("vue").Ref<string, string>;
/**
 * @internal
 */
export interface NuxtI18nContext {
    vueI18n: I18n;
    config: I18nPublicRuntimeConfig;
    /** Initial request/visit */
    initial: boolean;
    /** Locale messages attached during SSR and loaded during hydration */
    preloaded: boolean;
    usesRuntimeLoaders: (locale: Locale) => boolean;
    rootRedirect: {
        path: string;
        code: number;
    } | undefined;
    redirectStatusCode: number;
    /** Get default locale */
    getDefaultLocale: () => string;
    /** Get current locale */
    getLocale: () => string;
    /** Set locale directly  */
    setLocale: (locale: string) => Promise<void>;
    /** Set locale - suspend if `skipSettingLocaleOnNavigate` is enabled  */
    setLocaleSuspend: (locale: string) => Promise<void>;
    /** Get normalized runtime locales */
    getLocales: () => NormalizedLocaleObject[];
    /** Set locale to locale cookie */
    setCookieLocale: (locale: string) => void;
    /** Get current base URL */
    getBaseUrl: (locale?: string) => string;
    /** Base URL for a locale, ignoring the current host (alternate links) */
    getCanonicalBaseUrl: (locale: string) => string;
    /** Load locale messages */
    loadMessages: (locale: Locale) => Promise<void>;
    composableCtx: ComposableContext;
}
/**
 * `useRequestURL` reports the scheme the server was reached on: `http` behind a TLS-terminating
 * proxy, and nothing meaningful while prerendering. A configured `baseUrl` names the real one.
 * @internal exported for testing
 */
export declare function withConfiguredProtocol(url: {
    host: string;
    protocol: string;
}, baseUrl: string | BaseUrlResolveHandler<unknown> | undefined): {
    host: string;
    protocol: string;
};
/**
 * Returns a getter resolving the base URL for a locale, joined with `app.baseURL` exactly
 * once - locale domains and `baseUrl` are configured without it (#3628, #3887).
 */
export declare function createBaseUrlGetter(opts: {
    baseUrl: string | (() => string) | undefined;
    appBase: string;
    domains: boolean;
    getDomainForHost: () => string | undefined;
    getDomainFromLocale: (locale: string) => string | undefined;
}): (locale?: string) => string;
type MessageStore = Pick<Composer, 'getLocaleMessage' | 'setLocaleMessage' | 'mergeLocaleMessage'>;
export declare const isPrerenderable: (locale: Locale) => boolean;
/**
 * Installs loaded messages into `i18n` by reference, with whatever the vue-i18n config already put
 * there filled in underneath. Installed trees may be shared with the message cache, so
 * `mergeLocaleMessage` - which deep copies into its target - gets a private copy first.
 *
 * Patches the composer, not the VueI18n facade: legacy `useI18n()` and `<i18n>` blocks reach
 * composer methods directly, and the facade delegates to it at call time.
 * @internal exported for testing
 */
export declare function createMessageInstaller(i18n: MessageStore): (locale: string, message: object) => void;
export declare function createNuxtI18nContext(nuxt: NuxtApp, vueI18n: I18n, defaultLocale: string, flatJson?: boolean): NuxtI18nContext;
export declare function useNuxtI18nContext(nuxt: NuxtApp): NuxtI18nContext;
export {};
