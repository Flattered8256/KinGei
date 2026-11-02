import type { Strategies } from '#internal-i18n-types';
import type { useDetectors } from '../../shared/detection.js';
import type { useI18nDetection } from '../../shared/utils.js';
type Detectors = ReturnType<typeof useDetectors>;
export type RedirectResolverConfig = {
    detection: ReturnType<typeof useI18nDetection>;
    rootRedirect?: {
        path: string;
        code: number;
    };
    redirectStatusCode?: number;
    /** Resolves the localized variant of a path, returns `undefined` when it cannot be matched */
    matchLocalized: (path: string, locale: string, defaultLocale: string) => string | undefined;
    /** @default `isSupportedLocale` */
    isSupportedLocale?: (locale?: string) => boolean;
    strategy: Strategies;
    /** Whether routes are localized (pages enabled and strategy is not `no_prefix`) */
    routing: boolean;
    /** Whether locales are resolved from domains */
    domains: boolean;
};
/** `origin` is set when the redirect moves to another domain, a relative redirect otherwise */
export type ResolvedRedirect = {
    path: string | undefined;
    code: number;
    locale: string;
    origin?: string;
};
export declare function createRedirectResolver(config: RedirectResolverConfig): (fullPath: string, path: string | undefined, pathLocale: string | undefined, defaultLocale: string, detectors: Detectors, relocate?: (locale: string) => ResolvedRedirect | undefined) => ResolvedRedirect;
export {};
