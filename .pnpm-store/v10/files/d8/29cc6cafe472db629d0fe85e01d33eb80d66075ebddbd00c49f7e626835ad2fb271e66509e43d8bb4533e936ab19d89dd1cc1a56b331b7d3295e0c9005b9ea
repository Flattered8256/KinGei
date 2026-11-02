import { type useI18nDetection } from '../shared/utils.js';
import type { H3Event } from 'h3';
import type { CompatRoute } from '../types.js';
import { type NuxtApp } from '#app';
export declare const useDetectors: (event: H3Event | undefined, config: {
    cookieKey: string;
    cookieDomain?: string | null;
}, nuxtApp?: NuxtApp) => {
    cookie: () => string | undefined;
    header: () => string | undefined;
    navigator: () => string | undefined;
    host: (path: string) => string | undefined;
    route: (path: string | CompatRoute) => string | undefined;
    /** Passes the locale through when the current host serves it, `undefined` otherwise */
    onHost: (locale: string | null | undefined) => string | null | undefined;
    /** Whether the visitor arrived from one of the configured domains */
    fromOwnDomain: () => boolean;
    /** Whether a cookie scoped to the configured `cookieDomain` is readable on every domain */
    cookieSpans: () => boolean;
};
export type LocaleDetectorConfig = {
    detection: ReturnType<typeof useI18nDetection>;
    /** @default `isSupportedLocale` */
    isSupportedLocale?: (locale?: string) => boolean;
    /** Whether routes are localized (pages enabled and strategy is not `no_prefix`) */
    routing: boolean;
    /** Whether locales are resolved from domains */
    domains: boolean;
};
export declare function createLocaleDetector(config: LocaleDetectorConfig): (detectors: ReturnType<typeof useDetectors>, route: string | CompatRoute, initial: boolean) => string;
